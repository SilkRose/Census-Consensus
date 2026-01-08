use crate::env_vars;
use actix_web::web::Data;
use std::ops::Deref;
use std::sync::Arc;

pub type FimficData = Data<FimficArc>;

#[derive(Clone)]
pub struct FimficArc {
	inner: Arc<Fimfic>
}

pub struct Fimfic {
	pub client_id: String,
	pub client_secret: String,
	pub oauth_redirect_url: String
}

impl Fimfic {
	pub fn wrap(self) -> FimficArc {
		FimficArc { inner: Arc::new(self) }
	}
}

impl Deref for FimficArc {
	type Target = Fimfic;

	fn deref(&self) -> &Fimfic {
		&self.inner
	}
}
