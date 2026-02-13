use crate::auth::SessionInfo;
use crate::database::*;
use crate::html_templates::{
	ban_user_html, chapters_html, edit_chapter_html, edit_question_html, new_chapter_html,
	new_question_html, sessions_html, update_user_info_html, update_user_role_html,
};
use crate::html_templates::{chapter_history_html, user_feedback_html};
use crate::structs::{ChapterData, ChapterEdit, ChapterTable, Population, QuestionEdit, UserType};
use crate::utility::redirect;
use crate::{FimficCfg, HttpClient};
use actix_web::web::{Path, ThinData};
use actix_web::{HttpRequest, HttpResponse, Responder, get, post};
use chrono::Utc;
use pony::smart_map::SmartMap;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

pub const DATABASE_CONSTRAINT_EXPECT: &str =
	"Database constraints mean a user will always be present if they have a session.";

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
	let user = db
		.get_user(session.user_id)
		.await?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	let next_fetch_time = user.date_last_fetch + Duration::from_hours(1);
	if Utc::now() < next_fetch_time {
		return Ok(HttpResponse::BadRequest().finish());
	}
	let user_update = http_client
		.get_fimfic_user(user.id, &fimfic_cfg.bearer_token)
		.await?;
	db.insert_user(user.id, &user_update.data, user.user_type)
		.await?;
	Ok(HttpResponse::SeeOther()
		.append_header(("Location", redirect(req)))
		.finish())
}

#[get("/user-role")]
pub async fn get_update_user_role(
	mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let user = db
		.get_user(session.user_id)
		.await?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	if user.user_type != UserType::Admin {
		return Ok(HttpResponse::Unauthorized().finish());
	}
	let page = update_user_role_html();
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(page))
}

#[post("/user-role")]
pub async fn set_update_user_role(
	req: HttpRequest, body: String, mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let user = db
		.get_user(session.user_id)
		.await?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	if user.user_type != UserType::Admin {
		return Ok(HttpResponse::Unauthorized().finish());
	}
	let user = serde_urlencoded::from_str::<HashMap<String, String>>(&body)?;
	let user_id = user.get("id").and_then(|id| id.parse::<i32>().ok());
	let role = user.get("role").and_then(|role| UserType::from_str(role));
	if user_id.is_none() || role.is_none() {
		return Ok(HttpResponse::BadRequest().finish());
	}
	let user = db.get_user(user_id.unwrap()).await?;
	if user.is_none() {
		return Ok(HttpResponse::BadRequest().finish());
	}
	db.update_user_role(user_id.unwrap(), role.unwrap()).await?;
	Ok(HttpResponse::SeeOther()
		.append_header(("Location", redirect(req)))
		.finish())
}

#[get("/ban-user")]
pub async fn get_ban_user(
	mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let user = db
		.get_user(session.user_id)
		.await?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	if user.user_type != UserType::Admin {
		return Ok(HttpResponse::Unauthorized().finish());
	}
	let page = ban_user_html();
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(page))
}

#[post("/ban-user")]
pub async fn set_ban_user(
	req: HttpRequest, body: String, mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let user = db
		.get_user(session.user_id)
		.await?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	if user.user_type != UserType::Admin {
		return Ok(HttpResponse::Unauthorized().finish());
	}
	let user = serde_urlencoded::from_str::<HashMap<String, String>>(&body)?;
	let user_id = user.get("id").and_then(|id| id.parse::<i32>().ok());
	let reason = user
		.get("reason")
		.and_then(|msg| if msg.is_empty() { None } else { Some(msg) })
		.cloned();
	if user_id.is_none() || reason.is_none() {
		return Ok(HttpResponse::BadRequest().finish());
	}
	db.insert_banned_user(user_id.unwrap(), &reason.unwrap())
		.await?;
	Ok(HttpResponse::SeeOther()
		.append_header(("Location", redirect(req)))
		.finish())
}

#[get("/user-feedback")]
pub async fn get_user_feedback(
	mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let user = db
		.get_user(session.user_id)
		.await?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	let page = user_feedback_html(user.feedback_private, user.feedback_public);
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(page))
}

#[post("/user-feedback")]
pub async fn set_user_feedback(
	req: HttpRequest, body: String, mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let feedback = serde_urlencoded::from_str::<HashMap<String, String>>(&body)?;
	let private_feedback = feedback
		.get("feedback_private")
		.and_then(|msg| if msg.is_empty() { None } else { Some(msg) })
		.cloned();
	let public_feedback = feedback
		.get("feedback_public")
		.and_then(|msg| if msg.is_empty() { None } else { Some(msg) })
		.cloned();

	db.update_user_feedback(session.user_id, private_feedback, public_feedback)
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
	let sessions = serde_urlencoded::from_str::<HashMap<u32, String>>(&body)?
		.into_values()
		.collect::<Vec<_>>();
	let logout = sessions.contains(&session.token);
	for session_del in sessions {
		let check = db.get_session_by_token(&session_del).await?;
		if let Some(check) = check
			&& check.user_id == session.user_id
		{
			db.delete_session(&session_del).await?;
		}
	}
	if logout {
		Ok(HttpResponse::SeeOther()
			.append_header(("Location", "/logout"))
			.finish())
	} else {
		Ok(HttpResponse::SeeOther()
			.append_header(("Location", redirect(req)))
			.finish())
	}
}

#[get("/chapters")]
pub async fn get_chapters(
	mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let user = db
		.get_user(session.user_id)
		.await?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	if user.user_type == UserType::Voter {
		return Ok(HttpResponse::Unauthorized().finish());
	}
	let admin = user.user_type == UserType::Admin;
	let chapters = db
		.get_all_chapters()
		.await
		.expect(DATABASE_CONSTRAINT_EXPECT);
	let mut data = vec![];
	for chapter in chapters {
		let last_data = db
			.get_latest_chapter_revision(chapter.id)
			.await?
			.expect(DATABASE_CONSTRAINT_EXPECT);
		let first_data = db
			.get_latest_chapter_revision(chapter.id)
			.await?
			.expect(DATABASE_CONSTRAINT_EXPECT);
		let last_user = db
			.get_user(last_data.created_by)
			.await?
			.expect(DATABASE_CONSTRAINT_EXPECT);
		let first_user = db
			.get_user(first_data.created_by)
			.await?
			.expect(DATABASE_CONSTRAINT_EXPECT);
		let revisions = db.get_chapter_revisions_count_by_id(chapter.id).await?;
		let questions = db.get_question_count_by_chapter(chapter.id).await?;
		let table_data = ChapterTable {
			meta: chapter,
			revisions,
			questions,
			first_data,
			last_data,
			first_user,
			last_user,
		};
		data.push(table_data);
	}
	let page = chapters_html(data, admin);
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(page))
}

#[get("/chapters/new")]
pub async fn get_chapter_new(
	mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let user = db
		.get_user(session.user_id)
		.await?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	if user.user_type == UserType::Voter {
		return Ok(HttpResponse::Unauthorized().finish());
	}
	let page = new_chapter_html();
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(page))
}

#[post("/chapters/new")]
pub async fn set_chapter_new(
	body: String, mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let user = db
		.get_user(session.user_id)
		.await?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	if user.user_type == UserType::Voter {
		return Ok(HttpResponse::Unauthorized().finish());
	}
	let chapter_data = serde_urlencoded::from_str::<ChapterEdit>(&body)?;
	let chapter = db.insert_chapter(chapter_data, user).await?;
	Ok(HttpResponse::SeeOther()
		.append_header(("Location", format!("/chapters/{}", chapter.id)))
		.finish())
}

#[get("/chapters/{id}")]
pub async fn get_chapter_edit(
	path: Path<i32>, mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let id = path.into_inner();
	let user = db
		.get_user(session.user_id)
		.await?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	if user.user_type == UserType::Voter {
		return Ok(HttpResponse::Unauthorized().finish());
	}
	let chapter = db.get_chapter(id).await?;
	if let Some(chapter) = chapter {
		let data = db
			.get_latest_chapter_revision(chapter.id)
			.await?
			.expect(DATABASE_CONSTRAINT_EXPECT);
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
	path: Path<i32>, body: String, mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let id = path.into_inner();
	let user = db
		.get_user(session.user_id)
		.await?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	if user.user_type == UserType::Voter {
		return Ok(HttpResponse::Unauthorized().finish());
	}
	let chapter_rev = serde_urlencoded::from_str::<ChapterEdit>(&body)?;
	let chapter = db.get_chapter(id).await?;
	if let Some(chapter) = chapter {
		db.insert_chapter_revision(chapter_rev, user.id, chapter.id)
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
	path: Path<i32>, mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let id = path.into_inner();
	let user = db
		.get_user(session.user_id)
		.await?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	if user.user_type != UserType::Admin {
		return Ok(HttpResponse::Unauthorized().finish());
	}
	let chapters = db.get_all_chapters().await?;
	let max = chapters.iter().filter_map(|c| c.chapter_order).max();
	db.update_chapter_order(id, max.map_or(1, |i| i + 1))
		.await?;
	Ok(HttpResponse::SeeOther()
		.append_header(("Location", "/chapters"))
		.finish())
}

#[get("/chapters/{id}/ordered/{movement}")]
pub async fn set_chapter_order_move(
	path: Path<(i32, i32)>, mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let (id, movement) = path.into_inner();
	if movement.abs() != 1 {
		return Ok(HttpResponse::BadRequest().finish());
	}
	let user = db
		.get_user(session.user_id)
		.await?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	if user.user_type != UserType::Admin {
		return Ok(HttpResponse::Unauthorized().finish());
	}
	let chapter = db.get_chapter(id).await?;
	if let Some(chapter) = chapter
		&& let Some(order) = chapter.chapter_order
	{
		if order + movement == 0 {
			return Ok(HttpResponse::BadRequest().finish());
		}
		let chapter_above = db.get_chapter_by_order(order + movement).await?;
		if let Some(above) = chapter_above {
			db.swap_chapters_by_order(id, above.id, order, movement)
				.await?;
		} else {
			db.update_chapter_order_none(id).await?;
		}
	}
	Ok(HttpResponse::SeeOther()
		.append_header(("Location", "/chapters"))
		.finish())
}

#[get("/chapters/{id}/vote-duration/{movement}")]
pub async fn set_chapter_vote_duration_move(
	path: Path<(i32, i32)>, mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let (id, movement) = path.into_inner();
	if movement.abs() != 1 {
		return Ok(HttpResponse::BadRequest().finish());
	}
	let user = db
		.get_user(session.user_id)
		.await?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	if user.user_type != UserType::Admin {
		return Ok(HttpResponse::Unauthorized().finish());
	}
	let chapter = db.get_chapter(id).await?;
	if let Some(chapter) = chapter {
		let new_duratrion = chapter.vote_duration + movement;
		if new_duratrion == 0 {
			return Ok(HttpResponse::BadRequest().finish());
		}
		db.update_chapter_vote_duration(id, new_duratrion).await?;
	}
	Ok(HttpResponse::SeeOther()
		.append_header(("Location", "/chapters"))
		.finish())
}

#[get("/chapters/{id}/minutes-left/{movement}")]
pub async fn set_chapter_minutes_left_move(
	path: Path<(i32, i32)>, mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let (id, movement) = path.into_inner();
	if movement.abs() != 1 {
		return Ok(HttpResponse::BadRequest().finish());
	}
	let user = db
		.get_user(session.user_id)
		.await?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	if user.user_type != UserType::Admin {
		return Ok(HttpResponse::Unauthorized().finish());
	}
	let chapter = db.get_chapter(id).await?;
	if let Some(chapter) = chapter
		&& let Some(minutes_left) = chapter.minutes_left
	{
		let new_left = minutes_left + movement;
		if new_left < 0 {
			return Ok(HttpResponse::BadRequest().finish());
		}
		db.update_chapter_minutes_left(id, new_left).await?;
	}
	Ok(HttpResponse::SeeOther()
		.append_header(("Location", "/chapters"))
		.finish())
}

#[get("/chapters/{id}/revisions")]
pub async fn get_chapter_revisions(
	path: Path<i32>, mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let id = path.into_inner();
	let user = db
		.get_user(session.user_id)
		.await?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	if user.user_type == UserType::Voter {
		return Ok(HttpResponse::Unauthorized().finish());
	}
	let chapter = db.get_chapter(id).await?;
	if chapter.is_none() {
		return Ok(HttpResponse::BadRequest().finish());
	}
	let revisions = db.get_all_chapter_revisions_by_chapter(id).await?;
	let mut users = SmartMap::default();
	for revison in &revisions {
		let user = db
			.get_user(revison.created_by)
			.await?
			.expect(DATABASE_CONSTRAINT_EXPECT);
		users.insert(revison.id, user);
	}
	let chapter_data = ChapterData {
		meta: chapter.expect("Earlier check means this is always present."),
		data: revisions,
		users,
	};
	let page = chapter_history_html(chapter_data);
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(page))
}

#[get("/population")]
pub async fn get_population(
	mut db: ThinData<Db>, session: SessionInfo, population: ThinData<Arc<RwLock<Population>>>,
) -> actix_web::Result<impl Responder> {
	let user = db
		.get_user(session.user_id)
		.await?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	if user.user_type == UserType::Voter {
		return Ok(HttpResponse::Unauthorized().finish());
	}
	if let Ok(pop) = population.read() {
		Ok(HttpResponse::Ok()
			.content_type("text/html; charset=utf-8")
			.body(pop.inner.to_string()))
	} else {
		Ok(HttpResponse::InternalServerError().finish())
	}
}

#[get("/population/{pop}")]
pub async fn set_population(
	path: Path<u32>, mut db: ThinData<Db>, session: SessionInfo,
	population: ThinData<Arc<RwLock<Population>>>,
) -> actix_web::Result<impl Responder> {
	let new_pop = path.into_inner();
	let user = db
		.get_user(session.user_id)
		.await?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	if user.user_type == UserType::Voter {
		return Ok(HttpResponse::Unauthorized().finish());
	}
	if let Ok(mut pop) = population.write() {
		pop.inner = new_pop;
	} else {
		return Ok(HttpResponse::InternalServerError().finish());
	}
	Ok(HttpResponse::SeeOther()
		.append_header(("Location", "/population"))
		.finish())
}

#[get("/questions/new")]
pub async fn get_question_new(
	mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let user = db
		.get_user(session.user_id)
		.await?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	if user.user_type == UserType::Voter {
		return Ok(HttpResponse::Unauthorized().finish());
	}
	let page = new_question_html();
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(page))
}

#[post("/questions/new")]
pub async fn set_question_new(
	body: String, mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let user = db
		.get_user(session.user_id)
		.await?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	if user.user_type == UserType::Voter {
		return Ok(HttpResponse::Unauthorized().finish());
	}
	let question_data = serde_urlencoded::from_str::<QuestionEdit>(&body)?;
	let question = db.insert_question(None).await?;
	db.insert_question_revision(question_data, question.id, user.id)
		.await?;
	Ok(HttpResponse::SeeOther()
		.append_header(("Location", format!("/questions/{}", question.id)))
		.finish())
}

#[get("/questions/{id}")]
pub async fn get_question_edit(
	path: Path<i32>, mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let id = path.into_inner();
	let user = db
		.get_user(session.user_id)
		.await?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	if user.user_type == UserType::Voter {
		return Ok(HttpResponse::Unauthorized().finish());
	}
	let question = db.get_question(id).await?;
	if let Some(question) = question {
		let data = db
			.get_latest_question_revision(question.id)
			.await?
			.expect(DATABASE_CONSTRAINT_EXPECT);
		let page = edit_question_html(question, data);
		Ok(HttpResponse::Ok()
			.content_type("text/html; charset=utf-8")
			.body(page))
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}

#[post("/questions/{id}")]
pub async fn set_question_edit(
	path: Path<i32>, body: String, mut db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let id = path.into_inner();
	let user = db
		.get_user(session.user_id)
		.await?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	if user.user_type == UserType::Voter {
		return Ok(HttpResponse::Unauthorized().finish());
	}
	let question_rev = serde_urlencoded::from_str::<QuestionEdit>(&body)?;
	let question = db.get_question(id).await?;
	if let Some(question) = question {
		db.insert_question_revision(question_rev, user.id, question.id)
			.await?;
		Ok(HttpResponse::SeeOther()
			.append_header(("Location", format!("/questions/{id}")))
			.finish())
	} else {
		Ok(HttpResponse::BadRequest().finish())
	}
}
