use crate::env_vars;
use crate::error::ErrorWrapper;
use crate::utility::redirect;
use crate::{Db, html_templates::form_html_template};
use actix_web::http::header::SET_COOKIE;
use actix_web::web::ThinData;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post};
use std::collections::HashMap;

#[get("/user-feedback")]
pub async fn get_user_feedback(
	req: HttpRequest, db: ThinData<Db>,
) -> actix_web::Result<impl Responder> {
	if let Some(token) = req.cookie("fimfic-auth-session") {
		let session = db
			.get_session_by_token(token.value())
			.await
			.map_err(ErrorWrapper)?;
		if let Some(session) = session {
			let user = db
				.get_user(session.user_id)
				.await
				.map_err(ErrorWrapper)?
				.expect(
					"Database constraints mean a user will always be present if they have a session.",
				);
			let page = form_html_template(user.feedback_private, user.feedback_public);
			return Ok(HttpResponse::Ok()
				.content_type("text/html; charset=utf-8")
				.body(page));
		};
	}
	Ok(HttpResponse::Unauthorized().finish())
}

#[post("/user-feedback")]
pub async fn set_user_feedback(
	req: HttpRequest, body: String, db: ThinData<Db>,
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

	if let Some(token) = req.cookie("fimfic-auth-session") {
		let session = db
			.get_session_by_token(token.value())
			.await
			.map_err(ErrorWrapper)?;
		if let Some(session) = session {
			// If the database constraints mean a user is always present,
			// do we even need to get them before updating their feedback?
			//
			// let user = db
			// 	.get_user(session.user_id)
			// 	.await
			// 	.map_err(ErrorWrapper)?
			// 	.expect(
			// 		"Database constraints mean a user will always be present if they have a session.",
			// 	);
			db.update_user_feedback(session.user_id, private_feedback, public_feedback)
				.await
				.map_err(ErrorWrapper)?;
		}
	}

	Ok(HttpResponse::SeeOther()
		.append_header(("Location", redirect(req)))
		.finish())
}

#[get("/dev-session")]
pub async fn dev_session() -> actix_web::Result<impl Responder> {
	let mut res = HttpResponse::SeeOther();
	if let Some(token) = env_vars::create_dev_session() {
		let cookie = format!(
			"fimfic-auth-session={token}; Domain=127.0.0.1; Max-Age=2592000; HttpOnly; Secure; Path=/; SameSite=Lax;"
		);
		res.append_header((SET_COOKIE, cookie));
	}
	res.append_header(("Location", "/"));
	Ok(res.finish())
}
