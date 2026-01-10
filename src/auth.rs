use crate::fimfic_cfg::FimficCfg;
use crate::rand::gen_auth_state;
use actix_web::get;
use actix_web::{ HttpRequest, HttpResponse };
use actix_web::cookie::{ Cookie, SameSite };
use actix_web::cookie::time::Duration;
use actix_web::web::{ Data, Query };
use serde::Deserialize;

const STATE_COOKIE_NAME: &str = "fimfic-auth-state";
const STATE_COOKIE_MAX_AGE: Duration = Duration::hours(1);
const STATE_COOKIE_PATH: &str = "/login/fimfic";

const SESSION_COOKIE_NAME: &str = "fimfic-auth-session";
const SESSION_COOKIE_MAX_AGE: Duration = Duration::days(14);
const SESSION_COOKIE_PATH: &str = "/";

#[get("/login/fimfic")]
pub async fn fimfic_auth(
	req: HttpRequest,
	Query(form): Query<FimficAuthParams>,
	fimfic_data: Data<FimficCfg>
) -> HttpResponse {
	if let Some(code) = form.code && let Some(state) = form.state {
		fimfic_auth_return(req, code, state).await
	} else {
		fimfic_auth_redirect(fimfic_data).await
	}
}

#[derive(Deserialize)]
struct FimficAuthParams {
	code: Option<String>,
	state: Option<String>
}

async fn fimfic_auth_redirect(fimfic_data: Data<FimficCfg>) -> HttpResponse {
	let state = gen_auth_state();

	let login_url = format!("{login_url}&state={state}", login_url = &*fimfic_data.login_url);
	let cookie = Cookie::build(STATE_COOKIE_NAME, state)
		.max_age(STATE_COOKIE_MAX_AGE)
		.path(STATE_COOKIE_PATH)
		.same_site(SameSite::Lax)
		.http_only(true)
		.secure(true)
		.finish();

	HttpResponse::Found()
		.append_header(("location", login_url))
		.cookie(cookie)
		.finish()
}

async fn fimfic_auth_return(
	req: HttpRequest,
	code: String,
	state: String
) -> HttpResponse {
	let Some(state_cookie) = req.cookie(STATE_COOKIE_NAME) else {
		// todo present an actual error
		return HttpResponse::Ok()
			.content_type("text/plain")
			.body("no cooki found")
	};

	let state_cookie = state_cookie.value();
	if state_cookie != &*state {
		// todo present an actual error
		return HttpResponse::Ok()
			.content_type("text/plain")
			.body(format!("state mismatch\n\nstate param: {state}\nstate cookie: {state_cookie}"))
	}

	let state_cookie = Cookie::build(STATE_COOKIE_NAME, "3c")
		.path(STATE_COOKIE_PATH)
		.max_age(Duration::ZERO)
		.finish();
	let session_cookie = Cookie::build(SESSION_COOKIE_NAME, "pretend-this-is-a-token-todo-fix-this")
		.max_age(SESSION_COOKIE_MAX_AGE)
		.path(SESSION_COOKIE_PATH)
		.same_site(SameSite::Lax)
		.http_only(true)
		.secure(true)
		.finish();

	// todo redirect to home page or something
	HttpResponse::Ok()
		.cookie(state_cookie)
		.cookie(session_cookie)
		.content_type("text/plain")
		.body(format!(r#"the return!! code is "{code}" and state (verified) is "{state}""#))
}
