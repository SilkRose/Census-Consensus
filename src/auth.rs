use leptos::prelude::*;

#[server]
pub async fn login_url() -> Result<String, ServerFnError> {
	use crate::fimfic_config::FimficData;
	use leptos_actix::extract;

	Ok(String::from(&*extract::<FimficData>().await?.login_url))
}
