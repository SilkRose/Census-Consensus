use crate::fimfic_cfg::{ FIMFIC_USER_AGENT, FimficCfg };
use anyhow::Result;
use reqwest::Client as ReqwestClient;
use serde::Deserialize;
use std::borrow::Cow;

pub struct HttpClient {
	inner: ReqwestClient
}

pub struct FimficTokenExchangeResponse {
	pub id: i32,
	pub name: String,
	pub access_token: String
}

impl HttpClient {
	pub fn new() -> Result<Self> {
		let inner = ReqwestClient::builder()
			.https_only(true)
			.build()?;

		Ok(Self { inner })
	}

	// if we ever need to fetch more user data than only a
	// pfp from fimfic, modify this function into that
	pub async fn get_fimfic_pfp(&self, id: i32, token: &str) -> Result<Option<String>> {
		// todo is there a better way to do this?
		#[derive(Deserialize)]
		struct Res {
			data: Data
		}

		#[derive(Deserialize)]
		struct Data {
			attributes: Attributes
		}

		#[derive(Deserialize)]
		struct Attributes {
			avatar: Avatar
		}

		// help me
		#[derive(Deserialize)]
		struct Avatar {
			#[serde(rename = "512")]
			size_512: String
		}

		let res = self.inner.get(format!("https://www.fimfiction.net/api/v2/users/{id}"))
			.header("user-agent", FIMFIC_USER_AGENT)
			.header("authorization", format!("Bearer {token}"))
			.send()
			.await?
			.json::<Res>()
			.await?;

		let res = res.data.attributes.avatar.size_512;
		if let Some((link, _)) = res.rsplit_once('-') {
			Ok(Some(link.into()))
		} else if &*res == "https://static.fimfiction.net/images/none_64.png" {
			Ok(None)
		} else {
			Err(anyhow::anyhow!("invalid pfp_url format"))
		}
	}

	pub async fn fimfic_token_exchange(
		&self,
		fimfic_cfg: &FimficCfg,
		code: &str
	) -> Result<FimficTokenExchangeResponse> {
		// todo is there a better way to do this?
		// some kind of `path = "user.id"`?
		#[derive(Deserialize)]
		struct Res<'h> {
			access_token: String,
			user: ResUser<'h>
		}

		#[derive(Deserialize)]
		struct ResUser<'h> {
			id: Cow<'h, str>,
			name: String
		}

		let res = self.inner.post("https://www.fimfiction.net/api/v2/token")
			.header("user-agent", FIMFIC_USER_AGENT)
			.form::<[_]>(&[
				("client_id", &*fimfic_cfg.client_id),
				("client_secret", &*fimfic_cfg.client_secret),
				("grant_type", "authorization_code"),
				("redirect_uri", &*fimfic_cfg.oauth_redirect_url),
				("code", code)
			])
			.send()
			.await?
			.bytes()
			.await?;

		let res = serde_json::from_slice::<Res>(&res)?;

		Ok(FimficTokenExchangeResponse {
			id: res.user.id.parse()?,
			name: res.user.name,
			access_token: res.access_token
		})
	}
}
