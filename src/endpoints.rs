use crate::auth::{AdminSessionInfo, MaybeSessionInfo, SessionInfo, WriterSessionInfo};
use crate::database::*;
use crate::html_templates::*;
use crate::structs::*;
use crate::theme::Theme;
use crate::utility::{parse_options, redirect};
use crate::{FimficCfg, HttpClient};
use actix_web::web::{Path, Query, ThinData};
use actix_web::{HttpRequest, HttpResponse, Responder, get, post};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};
use lightningcss::stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet};
use pony::smart_map::SmartMap;
use pony::time::format_milliseconds;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::time::Duration;

pub const MIN_USER_UPDATE_TIME: Duration = Duration::from_hours(1);

#[get("/style.css")]
pub async fn get_css() -> actix_web::Result<impl Responder> {
	if let Ok(ref css_file) = fs::read_to_string("./src/style.css")
		&& let Ok(mut stylesheet) = StyleSheet::parse(css_file, ParserOptions::default())
	{
		if stylesheet.minify(MinifyOptions::default()).is_err() {
			return Ok(HttpResponse::Ok()
				.content_type("text/css; charset=utf-8")
				.body(css_file.clone()));
		}
		let opts = PrinterOptions {
			minify: true,
			..Default::default()
		};
		if let Ok(styles) = stylesheet.to_css(opts) {
			Ok(HttpResponse::Ok()
				.content_type("text/css; charset=utf-8")
				.body(styles.code))
		} else {
			Ok(HttpResponse::Ok()
				.content_type("text/css; charset=utf-8")
				.body(css_file.clone()))
		}
	} else {
		Ok(HttpResponse::InternalServerError().finish())
	}
}

#[get("/mane.js")]
pub async fn get_js() -> actix_web::Result<impl Responder> {
	Ok(HttpResponse::Ok()
		.content_type("text/javascript; charset=utf-8")
		.body(fs::read_to_string("./src/mane.js")?))
}

#[post("/logo/{opt}")]
pub async fn set_logo_submit(
	path: Path<String>, mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let opt = path.into_inner();
	let logo = match opt.as_str() {
		"census" => Logo::Census,
		"consensus" => Logo::Consensus,
		_ => return Ok(HttpResponse::BadRequest().finish()),
	};
	db.insert_logo_stat(logo, session.user_id).await?;
	Ok(HttpResponse::Ok().finish())
}

#[get("/user")]
pub async fn get_user(
	theme: Theme, mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let user = db.get_user(session.user_id).await?;
	let mut sessions = db.get_all_user_sessions(session.user_id).await?;
	sessions.sort_by_key(|k| k.last_seen);
	sessions.reverse();
	let page = user_settings_html(user, theme, sessions);
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(page))
}

#[get("/user/update")]
pub async fn set_update_user(
	req: HttpRequest, mut db: ThinData<Db>, session: SessionInfo,
	http_client: ThinData<HttpClient>, fimfic_cfg: ThinData<FimficCfg>,
) -> actix_web::Result<impl Responder> {
	let user = db.get_user(session.user_id).await?;
	let next_fetch_time = user.date_last_fetch + MIN_USER_UPDATE_TIME;
	if Utc::now() > next_fetch_time || user.user_type == UserType::Admin {
		let user_update = http_client
			.get_fimfic_user(user.id, &fimfic_cfg.bearer_token)
			.await?;
		db.insert_user(user.id, &user_update.data, user.user_type)
			.await?;
		Ok(HttpResponse::SeeOther()
			.append_header(("Location", redirect(req)))
			.finish())
	} else {
		let remaining = format_milliseconds(
			(next_fetch_time - Utc::now()).num_milliseconds() as u128,
			Some(2),
		)?;
		let msg = format!("Please wait {remaining} before trying again.");
		Ok(HttpResponse::TooManyRequests().body(msg))
	}
}

#[post("/user/role")]
pub async fn set_update_user_role(
	req: HttpRequest, body: String, mut db: ThinData<Db>, _: AdminSessionInfo,
) -> actix_web::Result<impl Responder> {
	let user_update = serde_urlencoded::from_str::<UserRoleUpdate>(&body)?;
	if let Some(user) = db.get_user_opt(user_update.id).await? {
		db.update_user_role(user.id, user_update.role).await?;
		Ok(HttpResponse::SeeOther()
			.append_header(("Location", redirect(req)))
			.finish())
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[post("/user/ban")]
pub async fn set_ban_user(
	req: HttpRequest, body: String, mut db: ThinData<Db>, _: AdminSessionInfo,
) -> actix_web::Result<impl Responder> {
	let user = serde_urlencoded::from_str::<UserBan>(&body)?;
	db.insert_banned_user(user.id, &user.reason).await?;
	if let Some(user) = db.get_user_opt(user.id).await? {
		db.update_user_role(user.id, UserType::Voter).await?;
	}
	Ok(HttpResponse::SeeOther()
		.append_header(("Location", redirect(req)))
		.finish())
}

#[post("/user/feedback")]
pub async fn set_user_feedback(
	req: HttpRequest, body: String, mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let feedback = serde_urlencoded::from_str::<UserFeedback>(&body)?;
	db.update_user_feedback(
		session.user_id,
		feedback.feedback_private,
		feedback.feedback_public,
	)
	.await?;
	Ok(HttpResponse::SeeOther()
		.append_header(("Location", redirect(req)))
		.finish())
}

#[post("/user/revoke-sessions")]
pub async fn set_revoke_sessions(
	req: HttpRequest, body: String, mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let sessions: HashSet<String> = serde_urlencoded::from_str::<HashMap<u32, String>>(&body)?
		.into_values()
		.collect();
	for session_del in &sessions {
		let check = db.get_session_by_token(session_del).await?;
		if let Some(check) = check
			&& check.user_id == session.user_id
		{
			db.delete_session(session_del).await?;
		}
	}
	let url = match sessions.contains(&session.token) {
		true => "/logout",
		false => &redirect(req),
	};
	Ok(HttpResponse::SeeOther()
		.append_header(("Location", url))
		.finish())
}

#[get("/chapters")]
pub async fn get_chapters(
	theme: Theme, mut db: ThinData<Db>, session: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let chapters = db.get_chapters_table().await?;
	let page = chapters_html(session.user, theme, chapters);
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(page))
}

#[post("/chapters")]
pub async fn set_chapter_new(
	body: String, mut db: ThinData<Db>, session: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let chapter_data = serde_urlencoded::from_str::<ChapterEdit>(&body)?;
	let chapter = db.insert_chapter(chapter_data, session.user).await?;
	Ok(HttpResponse::SeeOther()
		.append_header(("Location", format!("/chapters/{}", chapter.id)))
		.finish())
}

#[get("/chapters/{id}")]
pub async fn get_chapter_edit(
	path: Path<i32>, theme: Theme, mut db: ThinData<Db>, session: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let id = path.into_inner();
	let chapter = db.get_chapter(id).await?;
	if let Some(chapter) = chapter {
		let data = db.get_latest_chapter_revision(chapter.id).await?;
		let page = edit_chapter_html(session.user, theme, chapter, data);
		Ok(HttpResponse::Ok()
			.content_type("text/html; charset=utf-8")
			.body(page))
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[post("/chapters/{id}")]
pub async fn set_chapter_edit(
	path: Path<i32>, body: String, mut db: ThinData<Db>, session: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let id = path.into_inner();
	let chapter_rev = serde_urlencoded::from_str::<ChapterEdit>(&body)?;
	let chapter = db.get_chapter(id).await?;
	if let Some(chapter) = chapter
		&& chapter.fimfic_ch_id.is_none()
	{
		db.insert_chapter_revision(chapter_rev, session.user.id, chapter.id)
			.await?;
		Ok(HttpResponse::SeeOther()
			.append_header(("Location", format!("/chapters/{id}")))
			.finish())
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[get("/chapters/{id}/ordered")]
pub async fn set_chapter_order(
	path: Path<i32>, mut db: ThinData<Db>, _: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let id = path.into_inner();
	if let Some(chapter) = db.get_chapter(id).await?
		&& chapter.chapter_order.is_none()
		&& chapter.minutes_left.is_none()
		&& chapter.fimfic_ch_id.is_none()
	{
		let chapters = db.get_all_chapters().await?;
		let max = chapters.iter().filter_map(|c| c.chapter_order).max();
		db.update_chapter_order(id, max.map_or(1, |i| i + 1))
			.await?;
		Ok(HttpResponse::SeeOther()
			.append_header(("Location", "/chapters"))
			.finish())
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[get("/chapters/{id}/ordered/{movement}")]
pub async fn set_chapter_order_move(
	path: Path<(i32, i32)>, mut db: ThinData<Db>, _: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let (id, movement) = path.into_inner();
	if movement.abs() == 1
		&& let Some(chapter) = db.get_chapter(id).await?
		&& let Some(order) = chapter.chapter_order
		&& chapter.minutes_left.is_none()
		&& chapter.fimfic_ch_id.is_none()
		&& order + movement > 0
	{
		let chapter_above = db.get_chapter_by_order(order + movement).await?;
		if let Some(above) = chapter_above {
			db.swap_chapters_by_order(id, above.id, order, movement)
				.await?;
		} else {
			db.update_chapter_order_none(id).await?;
		}
		Ok(HttpResponse::SeeOther()
			.append_header(("Location", "/chapters"))
			.finish())
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[get("/chapters/{id}/vote-duration/{movement}")]
pub async fn set_chapter_vote_duration_move(
	path: Path<(i32, i32)>, mut db: ThinData<Db>, _: AdminSessionInfo,
) -> actix_web::Result<impl Responder> {
	let (id, movement) = path.into_inner();
	if movement.abs() == 1
		&& let Some(chapter) = db.get_chapter(id).await?
		&& chapter.minutes_left.is_none()
		&& chapter.fimfic_ch_id.is_none()
		&& chapter.vote_duration + movement > 0
	{
		let new_duration = chapter.vote_duration + movement;
		db.update_chapter_vote_duration(id, new_duration).await?;
		Ok(HttpResponse::SeeOther()
			.append_header(("Location", "/chapters"))
			.finish())
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[get("/chapters/{id}/minutes-left/{movement}")]
pub async fn set_chapter_minutes_left_move(
	path: Path<(i32, i32)>, mut db: ThinData<Db>, _: AdminSessionInfo,
) -> actix_web::Result<impl Responder> {
	let (id, movement) = path.into_inner();
	if movement.abs() == 1
		&& let Some(chapter) = db.get_chapter(id).await?
		&& let Some(minutes_left) = chapter.minutes_left
		&& chapter.fimfic_ch_id.is_none()
		&& minutes_left + movement > 0
	{
		let new_left = minutes_left + movement;
		db.update_chapter_minutes_left(id, Some(new_left)).await?;
		Ok(HttpResponse::SeeOther()
			.append_header(("Location", "/chapters"))
			.finish())
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[get("/chapters/{id}/revisions")]
pub async fn get_chapter_revisions(
	path: Path<i32>, theme: Theme, mut db: ThinData<Db>, session: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let id = path.into_inner();
	if let Some(chapter) = db.get_chapter(id).await? {
		let revisions = db.get_all_chapter_revisions_by_chapter(id).await?;
		let mut users = SmartMap::default();
		for revision in &revisions {
			let user = db.get_user(revision.created_by).await?;
			users.insert(revision.id, user);
		}
		let chapter_data = ChapterData {
			meta: chapter,
			data: revisions,
			users,
		};
		let page = chapter_history_html(session.user, theme, chapter_data);
		Ok(HttpResponse::Ok()
			.content_type("text/html; charset=utf-8")
			.body(page))
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[get("/feedback")]
pub async fn get_feedback(
	theme: Theme, mut db: ThinData<Db>, session: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	if let Ok(users) = db.get_all_users().await {
		let mut user_data = Vec::with_capacity(users.len());
		for user in users {
			let census = db.get_logo_stats_census_count_by_user(user.id).await?;
			let consensus = db.get_logo_stats_consensus_count_by_user(user.id).await?;
			let data = UserData {
				meta: user,
				logo_census: census,
				logo_consensus: consensus,
			};
			user_data.push(data);
		}
		let page = feedback_html(session.user, theme, user_data);
		Ok(HttpResponse::Ok()
			.content_type("text/html; charset=utf-8")
			.body(page))
	} else {
		Ok(HttpResponse::InternalServerError().finish())
	}
}

#[post("/questions")]
pub async fn set_question_new(
	params: Query<QuestionChapterId>, body: String, mut db: ThinData<Db>,
	session: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let chapter_id = params.chapter_id;
	let question_data = serde_urlencoded::from_str::<QuestionEdit>(&body)?;
	let question = db
		.insert_question(question_data, session.user, chapter_id)
		.await?;
	Ok(HttpResponse::SeeOther()
		.append_header(("Location", format!("/questions/{}", question.id)))
		.finish())
}

#[get("/questions/{id}")]
pub async fn get_question_edit(
	path: Path<i32>, theme: Theme, mut db: ThinData<Db>, session: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let id = path.into_inner();
	if let Some(question) = db.get_question(id).await?
		&& let Ok(settings) = db.get_settings().await
	{
		let population = settings.population;
		let data = db.get_latest_question_revision(question.id).await?;
		let page = edit_question_html(session.user, theme, question, data, population);
		Ok(HttpResponse::Ok()
			.content_type("text/html; charset=utf-8")
			.body(page))
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[post("/questions/{id}")]
pub async fn set_question_edit(
	path: Path<i32>, body: String, mut db: ThinData<Db>, session: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let id = path.into_inner();
	let question_rev = serde_urlencoded::from_str::<QuestionEdit>(&body)?;
	let question = db.get_question(id).await?;
	if let Some(question) = question {
		db.insert_question_revision(question_rev, question.id, session.user.id)
			.await?;
		Ok(HttpResponse::SeeOther()
			.append_header(("Location", format!("/questions/{id}")))
			.finish())
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[get("/questions/{id}/revisions")]
pub async fn get_question_revisions(
	path: Path<i32>, theme: Theme, mut db: ThinData<Db>, session: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let id = path.into_inner();
	if let Some(question) = db.get_question(id).await?
		&& let Ok(settings) = db.get_settings().await
	{
		let population = settings.population;
		let revisions = db.get_all_question_revisions_by_question(id).await?;
		let mut users = SmartMap::default();
		for revision in &revisions {
			let user = db.get_user(revision.created_by).await?;
			users.insert(revision.id, user);
		}
		let question_data = QuestionData {
			meta: question,
			data: revisions,
			users,
		};
		let page = question_history_html(session.user, theme, question_data, population);
		Ok(HttpResponse::Ok()
			.content_type("text/html; charset=utf-8")
			.body(page))
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[get("/chapters/{chapter_id}/questions")]
pub async fn get_chapter_questions(
	theme: Theme, path: Path<i32>, mut db: ThinData<Db>, session: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let chapter_id = path.into_inner();
	if let Some(chapter) = db.get_chapter(chapter_id).await?
		&& let Ok(settings) = db.get_settings().await
	{
		let population = settings.population;
		let (data, _) = db.get_questions_table(Some(chapter_id)).await?;
		let page = chapter_questions_html(session.user, theme, chapter, data, population);
		Ok(HttpResponse::Ok()
			.content_type("text/html; charset=utf-8")
			.body(page))
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[get("/questions/{id}/claim")]
pub async fn set_question_claim(
	path: Path<i32>, req: HttpRequest, mut db: ThinData<Db>, session: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let id = path.into_inner();
	if let Some(question) = db.get_question(id).await?
		&& question.claimed_by.is_none()
	{
		db.update_question_claimed_by(id, Some(session.user.id))
			.await?;
		Ok(HttpResponse::SeeOther()
			.append_header(("Location", redirect(req)))
			.finish())
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[get("/questions/{id}/unclaim")]
pub async fn set_question_unclaim(
	path: Path<i32>, req: HttpRequest, mut db: ThinData<Db>, session: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let id = path.into_inner();
	if let Some(question) = db.get_question(id).await?
		&& let Some(claimant) = question.claimed_by
		&& (claimant == session.user.id || session.user.user_type == UserType::Admin)
	{
		db.update_question_claimed_by(id, None).await?;
		Ok(HttpResponse::SeeOther()
			.append_header(("Location", redirect(req)))
			.finish())
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[get("/chapters/{chapter_id}/questions/{question_id}/ordered")]
pub async fn set_chapter_question_order(
	path: Path<(i32, i32)>, req: HttpRequest, mut db: ThinData<Db>, _: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let (chapter_id, question_id) = path.into_inner();
	db.add_question_to_chapter(question_id, chapter_id).await?;
	Ok(HttpResponse::SeeOther()
		.append_header(("Location", redirect(req)))
		.finish())
}

#[get("/chapters/{chapter_id}/questions/{question_id}/ordered/{movement}")]
pub async fn set_chapter_question_order_move(
	path: Path<(i32, i32, i32)>, req: HttpRequest, mut db: ThinData<Db>, _: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let (chapter_id, question_id, movement) = path.into_inner();
	if movement.abs() == 1
		&& let Some(question) = db.get_question(question_id).await?
		&& let Some(ch_id) = question.chapter_id
		&& let Some(order) = question.chapter_order
		&& chapter_id == ch_id
		&& order + movement != 0
	{
		let question_above = db
			.get_question_by_chapter_and_order(chapter_id, order + movement)
			.await?;
		if let Some(above) = question_above {
			db.swap_questions_by_order(question_id, above.id, order, movement)
				.await?;
		} else {
			db.update_question_chapter_id_order_none(question_id)
				.await?;
		}
		Ok(HttpResponse::SeeOther()
			.append_header(("Location", redirect(req)))
			.finish())
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[get("/questions")]
pub async fn get_questions(
	theme: Theme, mut db: ThinData<Db>, session: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	if let Ok((data, chapters)) = db.get_questions_table(None).await
		&& let Ok(settings) = db.get_settings().await
	{
		let population = settings.population;
		let page = questions_html(session.user, theme, data, chapters, population);
		Ok(HttpResponse::Ok()
			.content_type("text/html; charset=utf-8")
			.body(page))
	} else {
		Ok(HttpResponse::InternalServerError().finish())
	}
}

#[get("/about")]
pub async fn get_about(
	theme: Theme, mut db: ThinData<Db>, session: MaybeSessionInfo,
) -> actix_web::Result<impl Responder> {
	let contributors = db.get_all_contributors().await?;
	let user = match session.session_info {
		Some(user) => db.get_user_opt(user.user_id).await?,
		None => None,
	};
	let page = about_html(user, theme, contributors);
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(page))
}

#[get("/")]
pub async fn get_home(
	theme: Theme, mut db: ThinData<Db>, session: MaybeSessionInfo,
) -> actix_web::Result<impl Responder> {
	let user = match session.session_info {
		Some(user) => db.get_user(user.user_id).await?,
		None => {
			return Ok(HttpResponse::Ok()
				.content_type("text/html; charset=utf-8")
				.body(home_html(None, theme)));
		}
	};
	let setting = db.get_settings().await?;
	let page = if let Some(chapter) = db.get_active_chapter().await? {
		// event live
		let question_count = db.get_question_count_by_chapter(chapter.id).await?;
		if question_count > 0 {
			// survey chapter
			let questions = db.get_questions_by_chapter(chapter.id).await?;
			for question in questions {
				let votes = db
					.get_all_votes_by_question_and_user(question.id, user.id)
					.await?;
				if !votes.is_empty() {
					// previously voted
					let page = home_survey_complete_html(user, theme, chapter, question_count);
					return Ok(HttpResponse::Ok()
						.content_type("text/html; charset=utf-8")
						.body(page));
				}
			}
			// new voter
			todo!()
		} else {
			// final chapter
			let page = home_survey_complete_html(user, theme, chapter, question_count);
			return Ok(HttpResponse::Ok()
				.content_type("text/html; charset=utf-8")
				.body(page));
		}
	} else if let Some(start_time) = setting.start_time
		&& start_time < Utc::now()
	{
		// event over
		todo!()
	} else {
		// event hasn't started
		home_html(Some(user), theme)
	};
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(page))
}

#[get("/oembed")]
async fn oembed(query: Query<OEmbed>) -> actix_web::Result<impl Responder> {
	let embed = query.into_inner();
	Ok(HttpResponse::Ok()
		.content_type("application/json+oembed")
		.json(embed))
}

#[get("/questions/{id}/preview")]
pub async fn get_question_preview(
	theme: Theme, path: Path<i32>, query: Query<HashMap<String, f64>>, mut db: ThinData<Db>,
	session: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let id = path.into_inner();
	let options = query.into_inner();
	if let Some(question) = db.get_question(id).await?
		&& let Ok(settings) = db.get_settings().await
	{
		let data = db.get_latest_question_revision(question.id).await?;
		let population = settings.population;
		let page = question_preview_html(session.user, theme, question, data, options, population);
		Ok(HttpResponse::Ok()
			.content_type("text/html; charset=utf-8")
			.body(page))
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[get("/dashboard")]
pub async fn get_dashboard(
	theme: Theme, mut db: ThinData<Db>, session: AdminSessionInfo,
) -> actix_web::Result<impl Responder> {
	if let Ok(settings) = db.get_settings().await {
		let page = dashboard_html(session.user, theme, settings);
		Ok(HttpResponse::Ok()
			.content_type("text/html; charset=utf-8")
			.body(page))
	} else {
		Ok(HttpResponse::InternalServerError().finish())
	}
}

#[post("/story-id")]
pub async fn set_story_id(
	body: String, mut db: ThinData<Db>, _: AdminSessionInfo,
) -> actix_web::Result<impl Responder> {
	let data = serde_urlencoded::from_str::<HashMap<String, i32>>(&body)?;
	if let Some(story_id) = data.get("story-id") {
		db.update_story_id(*story_id).await?;
		Ok(HttpResponse::SeeOther()
			.append_header(("Location", "/dashboard"))
			.finish())
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[post("/population")]
pub async fn set_population(
	body: String, mut db: ThinData<Db>, _: AdminSessionInfo,
) -> actix_web::Result<impl Responder> {
	let data = serde_urlencoded::from_str::<HashMap<String, i32>>(&body)?;
	if let Some(population) = data.get("population") {
		db.update_population(*population).await?;
		Ok(HttpResponse::SeeOther()
			.append_header(("Location", "/dashboard"))
			.finish())
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[post("/vote-duration")]
pub async fn set_vote_duration(
	body: String, mut db: ThinData<Db>, _: AdminSessionInfo,
) -> actix_web::Result<impl Responder> {
	let data = serde_urlencoded::from_str::<HashMap<String, i32>>(&body)?;
	if let Some(vote_duration) = data.get("vote-duration") {
		db.update_chapter_vote_durations(*vote_duration).await?;
		Ok(HttpResponse::SeeOther()
			.append_header(("Location", "/dashboard"))
			.finish())
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[post("/reset")]
pub async fn set_reset(
	body: String, mut db: ThinData<Db>, _: AdminSessionInfo,
) -> actix_web::Result<impl Responder> {
	let data = serde_urlencoded::from_str::<HashMap<String, String>>(&body)?;
	if data.contains_key("reset-1") && data.contains_key("reset-2") && data.contains_key("reset-3")
	{
		if db.update_start_time(None).await.is_ok()
			&& db.delete_all_votes().await.is_ok()
			&& let Ok(chapters) = db.get_all_chapters().await
		{
			for chapter in chapters {
				db.update_chapter_minutes_left(chapter.id, None).await?;
				// Todo: Unpublish or delete fimfic chapters using id from chapter.
				db.update_chapter_fimfic_id(chapter.id, None).await?;
			}
			Ok(HttpResponse::SeeOther()
				.append_header(("Location", "/dashboard"))
				.finish())
		} else {
			Ok(HttpResponse::InternalServerError().finish())
		}
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[post("/start-time")]
pub async fn set_start_time(
	body: String, mut db: ThinData<Db>, _: AdminSessionInfo,
) -> actix_web::Result<impl Responder> {
	let data = serde_urlencoded::from_str::<HashMap<String, String>>(&body)?;
	if let Some(date) = data.get("date")
		&& let Some(time) = data.get("time")
		&& let Ok(date) = NaiveDate::parse_from_str(date, "%Y-%m-%d")
		&& let Ok(time) = NaiveTime::parse_from_str(time, "%H:%M")
	{
		let date_time = NaiveDateTime::new(date, time).and_utc();
		if db.update_start_time(Some(date_time)).await.is_ok() {
			Ok(HttpResponse::SeeOther()
				.append_header(("Location", "/dashboard"))
				.finish())
		} else {
			Ok(HttpResponse::InternalServerError().finish())
		}
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[get("/start-time/reset")]
pub async fn set_start_time_reset(
	mut db: ThinData<Db>, _: AdminSessionInfo,
) -> actix_web::Result<impl Responder> {
	if db.update_start_time(None).await.is_ok() {
		Ok(HttpResponse::SeeOther()
			.append_header(("Location", "/dashboard"))
			.finish())
	} else {
		Ok(HttpResponse::InternalServerError().finish())
	}
}

#[get("/chapters/{chapter_id}/survey")]
pub async fn get_chapter_survey(
	theme: Theme, path: Path<i32>, mut db: ThinData<Db>, session: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let chapter_id = path.into_inner();
	if let Some(chapter) = db.get_latest_chapter_revision_opt(chapter_id).await?
		&& let Ok(questions) = db.get_questions_by_chapter(chapter_id).await
	{
		let mut data = Vec::with_capacity(questions.len());
		for question in questions {
			let question_data = db.get_latest_question_revision(question.id).await?;
			data.push((question, question_data));
		}
		let page = chapter_survey_html(session.user, theme, chapter, data);
		Ok(HttpResponse::Ok()
			.content_type("text/html; charset=utf-8")
			.body(page))
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[post("/chapters/{chapter_id}/submit")]
pub async fn set_chapter_submit(
	path: Path<i32>, body: String, mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let chapter_id = path.into_inner();
	let votes = serde_urlencoded::from_str::<HashMap<String, String>>(&body)?;
	if let Some(chapter) = db.get_chapter(chapter_id).await?
		&& chapter.chapter_order.is_some()
		&& chapter.fimfic_ch_id.is_none()
	{
		let user = db.get_user(session.user_id).await?;
		let active = db
			.get_active_chapter()
			.await?
			.is_some_and(|ch| ch.id == chapter_id);
		if active || user.user_type != UserType::Voter {
			let questions = db.get_questions_by_chapter(chapter_id).await?;
			for question in questions {
				let Some(vote) = votes.get(&question.id.to_string()) else {
					continue;
				};
				let data = db.get_latest_question_revision(question.id).await?;
				let text = data.option_writing.unwrap_or_default();
				let options = parse_options(&text, &data.question_type)
					.into_iter()
					.collect::<HashMap<_, _>>();
				let answers: Vec<String> = match data.question_type {
					QuestionType::Multiselect => vote.split("+").map(|v| v.to_string()).collect(),
					_ => vec![vote.clone()],
				};
				db.delete_votes_by_question_and_user(question.id, user.id)
					.await?;
				for answer in answers {
					if !options.contains_key(&answer) {
						continue;
					}
					db.insert_vote(user.id, question.id, &answer).await?;
				}
			}
		}
		Ok(HttpResponse::SeeOther()
			.append_header(("Location", "/"))
			.finish())
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}
