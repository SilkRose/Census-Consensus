use crate::auth::SessionInfo;
use crate::error::ErrorWrapper;
use crate::html_templates::{ban_user_html, update_user_role_html};
use crate::structs::UserType;
use crate::utility::redirect;
use crate::{Db, html_templates::user_feedback_html};
use actix_web::web::ThinData;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post};
use std::collections::HashMap;

#[get("/user-role")]
pub async fn get_update_user_role(
	db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let user = db
		.get_user(session.user_id)
		.await
		.map_err(ErrorWrapper)?
		.expect("Database constraints mean a user will always be present if they have a session.");
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
		.expect("Database constraints mean a user will always be present if they have a session.");
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
		.expect("Database constraints mean a user will always be present if they have a session.");
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
		.expect("Database constraints mean a user will always be present if they have a session.");
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
		.expect("Database constraints mean a user will always be present if they have a session.");
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
