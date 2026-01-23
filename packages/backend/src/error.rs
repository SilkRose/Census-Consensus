use actix_web::{ HttpResponse, ResponseError };
use std::fmt;

pub struct ErrorWrapper(pub anyhow::Error);

impl From<anyhow::Error> for ErrorWrapper {
	fn from(error: anyhow::Error) -> Self {
		Self(error)
	}
}

impl ResponseError for ErrorWrapper {
	fn error_response(&self) -> HttpResponse {
		// todo: should we send the error details? or is that a vulnerability
		HttpResponse::InternalServerError()
			.insert_header(("content-type", "text/plain"))
			.body("internal server error occured")
	}
}

impl fmt::Debug for ErrorWrapper {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(&self.0, f)
	}
}

impl fmt::Display for ErrorWrapper {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Display::fmt(&self.0, f)
	}
}
