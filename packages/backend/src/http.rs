use crate::error::Result;
use crate::fimfic_cfg::FimficCfg;
use pony::fimfiction_api::user::UserApi;
use reqwest::{
	Client as ReqwestClient, IntoUrl, RequestBuilder,
	header::{AUTHORIZATION, USER_AGENT},
};
use serde::Deserialize;
use std::borrow::Cow;

#[derive(Clone)]
pub struct HttpClient {
	inner: ReqwestClient,
}

pub struct FimficTokenExchangeResponse {
	pub user_id: i32,
	pub name: String,
	pub access_token: String,
}

impl HttpClient {
	pub fn new() -> Result<Self> {
		let inner = ReqwestClient::builder().https_only(true).build()?;

		Ok(Self { inner })
	}

	// if we ever need to fetch more user data than only a
	// pfp from fimfic, modify this function into that
	pub async fn get_fimfic_user(&self, user_id: i32, token: &str) -> Result<UserApi<i32>> {
		self.get(
			format!("https://www.fimfiction.net/api/v2/users/{user_id}"),
			Some(token),
		)
		.send()
		.await?
		.json()
		.await
		.map_err(Into::into)
	}

	pub async fn fimfic_token_exchange(
		&self, fimfic_cfg: &FimficCfg, code: &str,
	) -> Result<FimficTokenExchangeResponse> {
		// todo is there a better way to do this?
		// some kind of `path = "user.id"`?
		#[derive(Deserialize)]
		struct Res<'h> {
			access_token: String,
			user: ResUser<'h>,
		}

		#[derive(Deserialize)]
		struct ResUser<'h> {
			id: Cow<'h, str>,
			name: String,
		}

		let res = self
			.post("https://www.fimfiction.net/api/v2/token", None)
			.form::<[_]>(&[
				("client_id", &*fimfic_cfg.client_id),
				("client_secret", &*fimfic_cfg.client_secret),
				("grant_type", "authorization_code"),
				("redirect_uri", &*fimfic_cfg.oauth_redirect_url),
				("code", code),
			])
			.send()
			.await?
			.bytes()
			.await?;

		println!("{}", String::from_utf8_lossy(&res));
		let res = serde_json::from_slice::<Res>(&res)?;

		Ok(FimficTokenExchangeResponse {
			user_id: res.user.id.parse()?,
			name: res.user.name,
			access_token: res.access_token,
		})
	}
}

// internal only helper functions
fn common_setup(mut builder: RequestBuilder, token: Option<&str>) -> RequestBuilder {
	// todo need real header
	builder = builder.header(
		USER_AGENT,
		format!("Silk Rose Survey {}", env!("CARGO_PKG_VERSION")),
	);

	if let Some(token) = token {
		builder = builder.header(AUTHORIZATION, format!("Bearer {token}"));
	}

	builder
}

macro_rules! http_methods {
	($($method:ident)*) => {
		$(
			fn $method(&self, url: impl IntoUrl, token: Option<&str>) -> RequestBuilder {
				common_setup(self.inner.$method(url), token)
			}
		)*
	}
}

impl HttpClient {
	http_methods!(get post);
}
