use crate::{Db, html_templates::form_html_template};
use crate::error::ErrorWrapper;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web::Data};
use std::collections::HashMap;

type Result<T, E = Box<dyn ::std::error::Error>> = ::std::result::Result<T, E>;

#[get("/form-page")]
pub async fn form_page() -> Result<impl Responder> {
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(form_html_template()))
}

#[post("/user-feedback")]
pub async fn user_feedback(
	req: HttpRequest, body: String, db: Data<Db>,
) -> actix_web::Result<impl Responder> {
	let feedback = serde_urlencoded::from_str::<HashMap<String, String>>(&body)?;
	let private_feedback = feedback
		.get("feedback_private")
		.and_then(|msg| if msg.is_empty() { None } else { Some(msg) });
	let public_feedback = feedback
		.get("feedback_public")
		.and_then(|msg| if msg.is_empty() { None } else { Some(msg) });

	if let Some(token) = req.cookie("fimfic-auth-session-info") {
		let session = db.get_session_by_token(token.value()).await.map_err(ErrorWrapper)?;
		// if let Ok(session) = session {
		// 	if let Some(session) = session {}
		// }
	}

	let redirect_to = req
		.headers()
		.get("Referer")
		.and_then(|v| v.to_str().ok())
		.unwrap_or("/");

	Ok(HttpResponse::SeeOther()
		.append_header(("Location", redirect_to))
		.finish())
}
