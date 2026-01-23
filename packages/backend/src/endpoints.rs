use actix_web::{HttpRequest, HttpResponse, Responder, get, post};
use std::{collections::HashMap, error::Error};

use crate::html_templates::form_html_template;

#[get("/form-page")]
pub async fn form_page() -> Result<impl Responder, Box<dyn Error>> {
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(form_html_template()))
}

#[post("/form-endpoint")]
pub async fn form_endpoint(
	_req: HttpRequest, body: String,
) -> Result<impl Responder, Box<dyn Error>> {
	let params = serde_urlencoded::from_str::<HashMap<String, String>>(&body)?;
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(format!("{params:#?}")))
}
