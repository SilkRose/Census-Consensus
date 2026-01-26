use crate::auth::SessionInfo;
use crate::error::ErrorWrapper;
use crate::html_templates::{
	ban_user_html, chapters_html, new_chapter_html, sessions_html, update_user_info_html,
	update_user_role_html,
};
use crate::structs::{NewChapter, UserType};
use crate::utility::redirect;
use crate::{Db, html_templates::user_feedback_html};
use crate::{FimficCfg, HttpClient};
use actix_web::web::ThinData;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post};
use chrono::Utc;
use std::collections::HashMap;
use std::time::Duration;

const DATABASE_CONSTRAINT_EXPECT: &str =
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
	req: HttpRequest, db: ThinData<Db>, session: SessionInfo, http_client: ThinData<HttpClient>,
	fimfic_cfg: ThinData<FimficCfg>,
) -> actix_web::Result<impl Responder> {
	let user = db
		.get_user(session.user_id)
		.await
		.map_err(ErrorWrapper)?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	let next_fetch_time = user.date_last_fetch + Duration::from_hours(1);
	if Utc::now() < next_fetch_time {
		return Ok(HttpResponse::BadRequest().finish());
	}
	let user_update = http_client
		.get_fimfic_user(user.id, &fimfic_cfg.bearer_token)
		.await
		.map_err(ErrorWrapper)?;
	db.insert_user(user.id, &user_update.data, user.user_type)
		.await
		.map_err(ErrorWrapper)?;
	Ok(HttpResponse::SeeOther()
		.append_header(("Location", redirect(req)))
		.finish())
}

#[get("/user-role")]
pub async fn get_update_user_role(
	db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let user = db
		.get_user(session.user_id)
		.await
		.map_err(ErrorWrapper)?
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
	req: HttpRequest, body: String, db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let user = db
		.get_user(session.user_id)
		.await
		.map_err(ErrorWrapper)?
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
	let user = db.get_user(user_id.unwrap()).await.map_err(ErrorWrapper)?;
	if user.is_none() {
		return Ok(HttpResponse::BadRequest().finish());
	}
	db.update_user_role(user_id.unwrap(), role.unwrap())
		.await
		.map_err(ErrorWrapper)?;
	Ok(HttpResponse::SeeOther()
		.append_header(("Location", redirect(req)))
		.finish())
}

#[get("/ban-user")]
pub async fn get_ban_user(
	db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let user = db
		.get_user(session.user_id)
		.await
		.map_err(ErrorWrapper)?
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
	req: HttpRequest, body: String, db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let user = db
		.get_user(session.user_id)
		.await
		.map_err(ErrorWrapper)?
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
		.await
		.map_err(ErrorWrapper)?;
	Ok(HttpResponse::SeeOther()
		.append_header(("Location", redirect(req)))
		.finish())
}

#[get("/user-feedback")]
pub async fn get_user_feedback(
	db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let user = db
		.get_user(session.user_id)
		.await
		.map_err(ErrorWrapper)?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	let page = user_feedback_html(user.feedback_private, user.feedback_public);
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(page))
}

#[post("/user-feedback")]
pub async fn set_user_feedback(
	req: HttpRequest, body: String, db: ThinData<Db>, session: SessionInfo,
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
		.await
		.map_err(ErrorWrapper)?;

	Ok(HttpResponse::SeeOther()
		.append_header(("Location", redirect(req)))
		.finish())
}

#[get("/sessions")]
pub async fn get_sessions(
	db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let mut sessions = db
		.get_all_user_sessions(session.user_id)
		.await
		.map_err(ErrorWrapper)?;
	sessions.sort_by_key(|k| k.last_seen);
	sessions.reverse();
	let page = sessions_html(sessions);
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(page))
}

#[post("/revoke-sessions")]
pub async fn set_revoke_sessions(
	req: HttpRequest, body: String, db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let sessions = serde_urlencoded::from_str::<HashMap<u32, String>>(&body)?
		.into_values()
		.collect::<Vec<_>>();
	let logout = sessions.contains(&session.token);
	for session_del in sessions {
		let check = db
			.get_session_by_token(&session_del)
			.await
			.map_err(ErrorWrapper)?;
		if let Some(check) = check
			&& check.user_id == session.user_id
		{
			db.delete_session(&session_del)
				.await
				.map_err(ErrorWrapper)?;
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
	db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let user = db
		.get_user(session.user_id)
		.await
		.map_err(ErrorWrapper)?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	if user.user_type == UserType::Voter {
		return Ok(HttpResponse::Unauthorized().finish());
	}
	let chapters = db
		.get_all_chapters()
		.await
		.map_err(ErrorWrapper)
		.expect(DATABASE_CONSTRAINT_EXPECT);
	let page = chapters_html(chapters);
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(page))
}

#[get("/chapters/new")]
pub async fn get_chapter_new(
	db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let user = db
		.get_user(session.user_id)
		.await
		.map_err(ErrorWrapper)?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	if user.user_type != UserType::Admin {
		return Ok(HttpResponse::Unauthorized().finish());
	}
	let page = new_chapter_html();
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(page))
}

#[post("/chapters/new")]
pub async fn set_chapter_new(
	body: String, db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let user = db
		.get_user(session.user_id)
		.await
		.map_err(ErrorWrapper)?
		.expect(DATABASE_CONSTRAINT_EXPECT);
	if user.user_type != UserType::Admin {
		return Ok(HttpResponse::Unauthorized().finish());
	}
	let chapter = serde_urlencoded::from_str::<NewChapter>(&body)?;
	let chapter = db
		.insert_chapter(&chapter.title, chapter.vote_duration)
		.await
		.map_err(ErrorWrapper)?;
	Ok(HttpResponse::SeeOther()
		.append_header(("Location", format!("/chapters/{}", chapter.id)))
		.finish())
}
