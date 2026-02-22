use crate::error::{Error, Result};

use actix_web::{FromRequest, HttpRequest};
use actix_web::dev::Payload;
use core::future::{Ready, ready};
use core::str::FromStr;

pub enum Theme {
	Light,
	Dark,
	None
}

impl FromStr for Theme {
	type Err = ();

	fn from_str(s: &str) -> Result<Theme, ()> {
		match s {
			"light" => { Ok(Theme::Light) }
			"dark" => { Ok(Theme::Dark) }
			"none" => { Ok(Theme::None) }
			_ => { Err(()) }
		}
	}
}

impl FromRequest for Theme {
	type Error = Error;
	type Future = Ready<Result<Theme>>;

	fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Ready<Result<Self>> {
		let Some(cookie) = req.cookie("theme") else {
			return ready(Ok(Theme::None))
		};

		let theme = cookie.value()
			.parse()
			.ok()
			.unwrap_or(Theme::None);

		ready(Ok(theme))
	}
}
