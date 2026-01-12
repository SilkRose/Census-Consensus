use crate::fimfic_cfg::{ FimficCfg, FIMFIC_TOKEN_EXCHANGE_URL };
use anyhow::Result;
use reqwest::Client as ReqwestClient;
use serde::Deserialize;

pub struct HttpClient {
	inner: ReqwestClient
}

pub struct FimficTokenExchangeResponse {
	pub id: u32,
	pub name: String
}

impl HttpClient {
	pub fn new() -> Result<Self> {
		let inner = ReqwestClient::builder()
			.https_only(true)
			.build()?;

		Ok(Self { inner })
	}

	pub async fn fimfic_token_exchange(
		&self,
		fimfic_cfg: &FimficCfg,
		code: &str
	) -> Result<FimficTokenExchangeResponse> {
		// todo is there a better way to do this?
		// some kind of `path = "user.id"`?
		#[derive(Deserialize)]
		struct Res {
			user: ResUser
		}

		#[derive(Deserialize)]
		struct ResUser {
			id: String,
			name: String
		}

		let res = self.inner.post(FIMFIC_TOKEN_EXCHANGE_URL)
			// todo need a proper user agent
			.header("user-agent", "fish")
			.form::<[_]>(&[
				("client_id", &*fimfic_cfg.client_id),
				("client_secret", &*fimfic_cfg.client_secret),
				("grant_type", "authorization_code"),
				("redirect_uri", &*fimfic_cfg.oauth_redirect_url),
				("code", code)
			])
			.send()
			.await?
			.json::<Res>()
			.await?;

		Ok(FimficTokenExchangeResponse {
			id: res.user.id.parse()?,
			name: res.user.name
		})
	}
}
