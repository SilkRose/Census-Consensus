use crate::auth::{AdminSessionInfo, SessionInfo, WriterSessionInfo};
use crate::database::*;
use crate::html_templates::*;
use crate::structs::*;
use crate::utility::redirect;
use crate::{FimficCfg, HttpClient};
use actix_web::web::{Path, ThinData};
use actix_web::{HttpRequest, HttpResponse, Responder, get, post};
use chrono::Utc;
use pony::number_format::format_number_u128;
use pony::smart_map::SmartMap;
use pony::time::format_milliseconds;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::sync::{Arc, RwLock};
use std::time::Duration;

#[get("/style.css")]
pub async fn get_css() -> actix_web::Result<impl Responder> {
	Ok(HttpResponse::Ok()
		.content_type("text/css; charset=utf-8")
		.body(fs::read_to_string("./src/style.css")?))
}

#[get("/update-user")]
pub async fn get_update_user() -> actix_web::Result<impl Responder> {
	let page = update_user_info_html();
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(page))
}

#[post("/update-user")]
pub async fn set_update_user(
	req: HttpRequest, mut db: ThinData<Db>, session: SessionInfo,
	http_client: ThinData<HttpClient>, fimfic_cfg: ThinData<FimficCfg>,
) -> actix_web::Result<impl Responder> {
	let user = db.get_user(session.user_id).await?;
	let next_fetch_time = user.date_last_fetch + Duration::from_hours(1);
	if Utc::now() > next_fetch_time {
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
			None,
		)?;
		let msg = format!("Please wait {remaining} before trying again.");
		Ok(HttpResponse::TooManyRequests().body(msg))
	}
}

#[get("/user-role")]
pub async fn get_update_user_role(_: AdminSessionInfo) -> actix_web::Result<impl Responder> {
	let page = update_user_role_html();
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(page))
}

#[post("/user-role")]
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

#[get("/ban-user")]
pub async fn get_ban_user(_: AdminSessionInfo) -> actix_web::Result<impl Responder> {
	let page = ban_user_html();
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(page))
}

#[post("/ban-user")]
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

#[get("/user-feedback")]
pub async fn get_user_feedback(
	mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let user = db.get_user(session.user_id).await?;
	let page = user_feedback_html(user.feedback_private, user.feedback_public);
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(page))
}

#[post("/user-feedback")]
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

#[get("/sessions")]
pub async fn get_sessions(
	mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let mut sessions = db.get_all_user_sessions(session.user_id).await?;
	sessions.sort_by_key(|k| k.last_seen);
	sessions.reverse();
	let page = sessions_html(sessions);
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(page))
}

#[post("/revoke-sessions")]
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
	mut db: ThinData<Db>, session: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let admin = session.user.user_type == UserType::Admin;
	let data = db.get_chapters_table().await?;
	let page = chapters_html(data, admin);
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(page))
}

#[get("/chapters/new")]
pub async fn get_chapter_new(_: WriterSessionInfo) -> actix_web::Result<impl Responder> {
	let page = new_chapter_html();
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(page))
}

#[post("/chapters/new")]
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
	path: Path<i32>, mut db: ThinData<Db>, _: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let id = path.into_inner();
	let chapter = db.get_chapter(id).await?;
	if let Some(chapter) = chapter {
		let data = db.get_latest_chapter_revision(chapter.id).await?;
		let page = edit_chapter_html(chapter, data);
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
	path: Path<i32>, mut db: ThinData<Db>, _: AdminSessionInfo,
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
	path: Path<(i32, i32)>, mut db: ThinData<Db>, _: AdminSessionInfo,
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
		let new_duratrion = chapter.vote_duration + movement;
		db.update_chapter_vote_duration(id, new_duratrion).await?;
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
		db.update_chapter_minutes_left(id, new_left).await?;
		Ok(HttpResponse::SeeOther()
			.append_header(("Location", "/chapters"))
			.finish())
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[get("/chapters/{id}/revisions")]
pub async fn get_chapter_revisions(
	path: Path<i32>, mut db: ThinData<Db>, _: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let id = path.into_inner();
	if let Some(chapter) = db.get_chapter(id).await? {
		let revisions = db.get_all_chapter_revisions_by_chapter(id).await?;
		let mut users = SmartMap::default();
		for revison in &revisions {
			let user = db.get_user(revison.created_by).await?;
			users.insert(revison.id, user);
		}
		let chapter_data = ChapterData {
			meta: chapter,
			data: revisions,
			users,
		};
		let page = chapter_history_html(chapter_data);
		Ok(HttpResponse::Ok()
			.content_type("text/html; charset=utf-8")
			.body(page))
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[get("/population")]
pub async fn get_population(
	_: WriterSessionInfo, population: ThinData<Arc<RwLock<Population>>>,
) -> actix_web::Result<impl Responder> {
	if let Ok(pop) = population.read() {
		Ok(HttpResponse::Ok()
			.content_type("text/html; charset=utf-8")
			.body(format_number_u128(pop.inner as u128)?))
	} else {
		Ok(HttpResponse::InternalServerError().finish())
	}
}

#[get("/population/{pop}")]
pub async fn set_population(
	path: Path<u32>, _: AdminSessionInfo, population: ThinData<Arc<RwLock<Population>>>,
) -> actix_web::Result<impl Responder> {
	let new_pop = path.into_inner();
	if let Ok(mut pop) = population.write() {
		pop.inner = new_pop;
		Ok(HttpResponse::SeeOther()
			.append_header(("Location", "/population"))
			.finish())
	} else {
		Ok(HttpResponse::InternalServerError().finish())
	}
}

#[get("/questions/new")]
pub async fn get_question_new(_: WriterSessionInfo) -> actix_web::Result<impl Responder> {
	let page = new_question_html();
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(page))
}

#[post("/questions/new")]
pub async fn set_question_new(
	body: String, mut db: ThinData<Db>, session: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let question_data = serde_urlencoded::from_str::<QuestionEdit>(&body)?;
	let question = db.insert_question(question_data, session.user).await?;
	Ok(HttpResponse::SeeOther()
		.append_header(("Location", format!("/questions/{}", question.id)))
		.finish())
}

#[get("/questions/{id}")]
pub async fn get_question_edit(
	path: Path<i32>, population: ThinData<Arc<RwLock<Population>>>, mut db: ThinData<Db>,
	_: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let id = path.into_inner();
	let population = population.0.read().unwrap().inner;
	let question = db.get_question(id).await?;
	if let Some(question) = question {
		let data = db.get_latest_question_revision(question.id).await?;
		let page = edit_question_html(question, data, population);
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
	path: Path<i32>, population: ThinData<Arc<RwLock<Population>>>, mut db: ThinData<Db>,
	_: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let id = path.into_inner();
	let Ok(ref pop) = population.0.read() else {
		return Ok(HttpResponse::InternalServerError().finish());
	};
	let population = pop.inner;
	if let Some(question) = db.get_question(id).await? {
		let revisions = db.get_all_question_revisions_by_question(id).await?;
		let mut users = SmartMap::default();
		for revison in &revisions {
			let user = db.get_user(revison.created_by).await?;
			users.insert(revison.id, user);
		}
		let question_data = QuestionData {
			meta: question,
			data: revisions,
			users,
		};
		let page = question_history_html(question_data, population);
		Ok(HttpResponse::Ok()
			.content_type("text/html; charset=utf-8")
			.body(page))
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[get("/chapters/{chapter_id}/questions")]
pub async fn get_chapter_questions(
	path: Path<i32>, population: ThinData<Arc<RwLock<Population>>>, mut db: ThinData<Db>,
	session: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let chapter_id = path.into_inner();
	let Ok(ref pop) = population.0.read() else {
		return Ok(HttpResponse::InternalServerError().finish());
	};
	let population = pop.inner;
	if db.get_chapter_exists(chapter_id).await? {
		let data = db.get_chapter_questions_table(chapter_id).await?;
		let page = chapter_questions_html(data, chapter_id, population, session.user);
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
		&& let Some(claiment) = question.claimed_by
		&& (claiment == session.user.id || session.user.user_type == UserType::Admin)
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
	path: Path<(i32, i32)>, req: HttpRequest, mut db: ThinData<Db>, _: AdminSessionInfo,
) -> actix_web::Result<impl Responder> {
	let (chapter_id, question_id) = path.into_inner();
	let questions = db.get_questions_by_chapter(chapter_id).await?;
	let max = questions.iter().filter_map(|q| q.chapter_order).max();
	db.update_question_chapter_id_order(question_id, chapter_id, max.map_or(1, |i| i + 1))
		.await?;
	Ok(HttpResponse::SeeOther()
		.append_header(("Location", redirect(req)))
		.finish())
}

#[get("/chapters/{chapter_id}/questions/{question_id}/ordered/{movement}")]
pub async fn set_chapter_question_order_move(
	path: Path<(i32, i32, i32)>, req: HttpRequest, mut db: ThinData<Db>, _: AdminSessionInfo,
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
	population: ThinData<Arc<RwLock<Population>>>, mut db: ThinData<Db>, session: WriterSessionInfo,
) -> actix_web::Result<impl Responder> {
	let Ok(ref pop) = population.0.read() else {
		return Ok(HttpResponse::InternalServerError().finish());
	};
	let population = pop.inner;
	let data = db.get_questions_table().await?;
	let page = questions_html(data, population, session.user);
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(page))
}
