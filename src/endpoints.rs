use crate::auth::{AdminSessionInfo, MaybeSessionInfo, SessionInfo, WriterSessionInfo};
use crate::html_templates::*;
use crate::structs::*;
use crate::theme::Theme;
use crate::utility::{
	construct_chapter_data, construct_chapter_json, construct_question_data, parse_options,
	redirect,
};
use crate::{FimficCfg, HttpClient};
use crate::{database::*, result_formatter};
use actix_web::web::{Path, Query, ThinData};
use actix_web::{HttpRequest, HttpResponse, Responder, get, post};
use chrono::Utc;
use lightningcss::stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet};
use pony::smart_map::SmartMap;
use pony::time::format_milliseconds;
use rand::RngExt;
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
	let page = home_event_complete_html(user, theme);
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
	if db.get_chapter(chapter_id).await?.is_some() {
		let user = db.get_user(session.user_id).await?;
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
			db.delete_votes_complete_by_question_and_user(question.id, user.id)
				.await?;
			for answer in answers {
				if !options.contains_key(&answer) {
					continue;
				}
				db.insert_vote_complete(user.id, question.id, &answer)
					.await?;
			}
		}
		Ok(HttpResponse::SeeOther()
			.append_header(("Location", "/"))
			.finish())
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[get("/chapters/{chapter_id}/event-results")]
pub async fn get_chapter_preview_event(
	theme: Theme, path: Path<i32>, mut db: ThinData<Db>, session: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let chapter_id = path.into_inner();
	if let Some(chapter) = db.get_latest_chapter_revision_opt(chapter_id).await? {
		let question_count = db.get_question_count_by_chapter(chapter_id).await?;
		let text = match question_count > 0 {
			false => chapter.outro_text.clone().expect("Missing outro!"),
			true => {
				let settings = db.get_settings().await?;
				construct_chapter_data(&mut db, &settings, &chapter, true).await?
			}
		};
		let page = chapter_preview_event_html(session.user, theme, chapter, &text);
		Ok(HttpResponse::Ok()
			.content_type("text/html; charset=utf-8")
			.body(page))
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[get("/chapters/{chapter_id}/live-results")]
pub async fn get_chapter_preview_live(
	theme: Theme, path: Path<i32>, mut db: ThinData<Db>, session: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let chapter_id = path.into_inner();
	if let Some(chapter) = db.get_latest_chapter_revision_opt(chapter_id).await? {
		let question_count = db.get_question_count_by_chapter(chapter_id).await?;
		let text = match question_count > 0 {
			false => chapter.outro_text.clone().expect("Missing outro!"),
			true => {
				let settings = db.get_settings().await?;
				construct_chapter_data(&mut db, &settings, &chapter, true).await?
			}
		};
		let page = chapter_preview_live_html(session.user, theme, chapter, &text);
		Ok(HttpResponse::Ok()
			.content_type("text/html; charset=utf-8")
			.body(page))
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[get("/chapters/{chapter_id}/random-results")]
pub async fn get_chapter_preview_random(
	theme: Theme, path: Path<i32>, mut db: ThinData<Db>, session: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let chapter_id = path.into_inner();
	if let Some(chapter) = db.get_latest_chapter_revision_opt(chapter_id).await? {
		let settings = db.get_settings().await?;
		let mut texts = Vec::new();
		if let Some(ref intro) = chapter.intro_text {
			texts.push(intro.trim().to_string());
		}
		let questions = db.get_questions_by_chapter(chapter_id).await?;
		for question in questions {
			let data = db.get_latest_question_revision(question.id).await?;
			let options = data.option_writing.clone().unwrap_or_default();
			let option_tuples = parse_options(&options, &data.question_type);
			let max_count = 100_000;
			let mut current = 0;
			let mut results = HashMap::new();
			let mut rng = rand::rng();
			for option in option_tuples.iter().peekable() {
				let count = rng.random_range(0..=max_count - current);
				results.insert(option.0.clone(), count);
				current += count;
			}
			let options = OptionType::Count((results, current));
			let question_data = construct_question_data()
				.meta(question)
				.data(data)
				.option_texts(option_tuples)
				.option_data(options)
				.population(settings.population)
				.call();
			let (preview, errors) = result_formatter::format(&question_data);
			texts.push(preview.trim().to_string());
			for error in errors {
				eprintln!("Error in parsing question: {error}")
			}
		}
		if let Some(ref outro) = chapter.outro_text {
			texts.push(outro.trim().to_string());
		}
		let page = chapter_preview_random_html(session.user, theme, chapter, &texts.join("\n\n"));
		Ok(HttpResponse::Ok()
			.content_type("text/html; charset=utf-8")
			.body(page))
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[get("/chapters/{id}/update")]
pub async fn set_chapter_fimfic_update(
	path: Path<i32>, req: HttpRequest, mut db: ThinData<Db>, http_client: ThinData<HttpClient>,
	fimfic_cfg: ThinData<FimficCfg>, _: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let id = path.into_inner();
	let chapter = db.get_chapter(id).await?;
	let data = db.get_latest_chapter_revision(id).await?;
	let question_count = db.get_question_count_by_chapter(id).await?;
	let settings = db.get_settings().await?;
	if let Some(chapter) = chapter
		&& let Some(fimfic_id) = chapter.fimfic_ch_id
	{
		let json = construct_chapter_json()
			.db(&mut db)
			.settings(&settings)
			.data(data)
			.question_count(question_count)
			.call()
			.await?;
		http_client
			.patch_chapter(&fimfic_cfg, fimfic_id, json)
			.await?;
		Ok(HttpResponse::SeeOther()
			.append_header(("Location", redirect(req)))
			.finish())
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}
