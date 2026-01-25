use crate::auth::SessionInfo;
use crate::error::ErrorWrapper;
use crate::utility::redirect;
use crate::{Db, html_templates::form_html_template};
use actix_web::web::ThinData;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post};
use std::collections::HashMap;

#[get("/user-feedback")]
pub async fn get_user_feedback(
	db: ThinData<Db>, session: SessionInfo,
) -> actix_web::Result<impl Responder> {
	let user = db
		.get_user(session.user_id)
		.await
		.map_err(ErrorWrapper)?
		.expect(
			"Database constraints mean a user will always be present if they have a session.",
		);
	let page = form_html_template(user.feedback_private, user.feedback_public);
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
