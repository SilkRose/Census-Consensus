use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::fimfic_cfg::FimficCfg;
#[cfg(feature = "ssr")]
use actix_web::get;
#[cfg(feature = "ssr")]
use actix_web::HttpResponse;
#[cfg(feature = "ssr")]
use actix_web::web::{ Data, Query };
#[cfg(feature = "ssr")]
use serde::Deserialize;

#[cfg(feature = "ssr")]
#[get("/login/fimfic")]
pub async fn fimfic_auth(
	Query(form): Query<FimficAuthParams>,
	fimfic_data: Data<FimficCfg>
) -> String {
	if let Some(code) = &form.code && let Some(state) = &form.state {
		fimfic_auth_return(&code, &state).await
	} else {
		fimfic_auth_redirect().await
	}
}

#[cfg(feature = "ssr")]
#[derive(Deserialize)]
struct FimficAuthParams {
	code: Option<String>,
	state: Option<String>
}

#[cfg(feature = "ssr")]
async fn fimfic_auth_redirect() -> String {
	"hi :3".into()
}

#[cfg(feature = "ssr")]
async fn fimfic_auth_return(code: &str, state: &str) -> String {
	format!(r#"the return!! code is "{code}" and state is "{state}""#)
}
