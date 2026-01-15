use crate::fimfic_cfg::FimficCfg;
use crate::db::{ Db, UserType };
use crate::http::{ FimficTokenExchangeResponse, HttpClient };
use crate::rand::{ gen_auth_state, gen_auth_token };
use actix_web::get;
use actix_web::{ HttpRequest, HttpResponse };
use actix_web::cookie::{ Cookie, SameSite };
use actix_web::cookie::time::Duration;
use actix_web::web::{ Data, Query };
use bon::builder;
use serde::Deserialize;

const STATE_COOKIE_NAME: &str = "fimfic-auth-state";
const STATE_COOKIE_MAX_AGE: Duration = Duration::hours(1);
const STATE_COOKIE_PATH: &str = "/login/fimfic";

const SESSION_COOKIE_NAME: &str = "fimfic-auth-session";
const SESSION_COOKIE_MAX_AGE: Duration = Duration::days(14);
const SESSION_COOKIE_PATH: &str = "/";

#[derive(Deserialize)]
struct AuthQueryParams {
	code: Option<String>,
	state: Option<String>
}

#[get("/login/fimfic")]
pub async fn fimfic_auth(
	req: HttpRequest,
	Query(form): Query<AuthQueryParams>,
	db: Data<Db>,
	fimfic_cfg: Data<FimficCfg>,
	http_client: Data<HttpClient>
) -> HttpResponse {
	// todo also check that the auth cookie isn't already set, if it is already set
	// then skip this whole flow and redirect back immediately
	if let Some(code) = form.code && let Some(state) = form.state {
		fimfic_auth_return()
			.code(code)
			.state(state)
			.req(req)
			.db(db)
			.fimfic_cfg(fimfic_cfg)
			.http_client(http_client)
			.call()
			.await
	} else {
		fimfic_auth_redirect(fimfic_cfg).await
	}
}

async fn fimfic_auth_redirect(fimfic_cfg: Data<FimficCfg>) -> HttpResponse {
	let state = gen_auth_state();

	let login_url = format!("{login_url}&state={state}", login_url = &*fimfic_cfg.login_url);
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

#[builder]
async fn fimfic_auth_return(
	req: HttpRequest,
	db: Data<Db>,
	fimfic_cfg: Data<FimficCfg>,
	http_client: Data<HttpClient>,
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

	let FimficTokenExchangeResponse {
		id,
		name,
		access_token
	} = match http_client.fimfic_token_exchange(&fimfic_cfg, &code).await {
		Ok(res) => { res }
		Err(err) => {
			eprintln!("error in fimfic token exchange: {err}");
			// todo present an actual error
			return HttpResponse::Ok()
				.content_type("text/plain")
				.body("fimfic didn't like your code for some reason")
		}
	};

	let fimfic_pfp = match http_client.get_fimfic_pfp(id, &access_token).await {
		Ok(res) => { res }
		Err(err) => {
			eprintln!("error in pfp fetching: {err}");
			// todo present an actual error
			return HttpResponse::Ok()
				.content_type("text/plain")
				.body("fimfic didn't like the request for pfp")
		}
	};

	let db_result = db.create_or_update_user()
		.id(id)
		.name(&name)
		.maybe_pfp_url(fimfic_pfp.as_deref())
		.user_type(UserType::Voter)
		.call()
		.await;

	if let Err(err) = db_result {
		eprintln!("error in db storing: {err}");
		// todo present an actual error
		return HttpResponse::Ok()
			.content_type("text/plain")
			.body("db broke")
	}

	let token = gen_auth_token();

	let db_result = db.create_session()
		.token(&token)
		.id(id)
		.call()
		.await;

	if let Err(err) = db_result {
		eprintln!("error in token storing {err}");
		return HttpResponse::Ok()
			.content_type("text/plain")
			.body("storing token in db broke")
	}

	let state_cookie = Cookie::build(STATE_COOKIE_NAME, "3c")
		.path(STATE_COOKIE_PATH)
		.max_age(Duration::ZERO)
		.finish();
	let session_cookie = Cookie::build(SESSION_COOKIE_NAME, &*token)
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
		.body(format!(r#"the return!! code is "{code}" and state (verified) is "{state}" and token is "{token}""#))
}
