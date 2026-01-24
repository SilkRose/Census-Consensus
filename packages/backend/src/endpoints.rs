use crate::env_vars;
use crate::error::ErrorWrapper;
use crate::{Db, html_templates::form_html_template};
use actix_web::http::header::SET_COOKIE;
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
		let session = db
			.get_session_by_token(token.value())
			.await
			.map_err(ErrorWrapper)?;
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

#[get("/dev-session")]
pub async fn dev_session() -> actix_web::Result<impl Responder> {
	let mut res = HttpResponse::SeeOther();
	if let Some(token) = env_vars::create_dev_session() {
		let cookie = format!(
			"fimfic-auth-session={token}; Domain=127.0.0.1; Max-Age=2592000; HttpOnly; Secure; Path=/; SameSite=Lax;"
		);
		res.append_header((SET_COOKIE, cookie));
	}
	res.append_header(("Location", "/form-page"));
	Ok(res.finish())
}
