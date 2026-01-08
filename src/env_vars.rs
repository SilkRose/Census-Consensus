use std::env;

pub fn load_dotenv() {
	if let Err(err) = dotenvy::dotenv() {
		eprintln!("dotenv failed to load: {err:?}")
	}
}

/// Sets required environment variables with defaults, if they are not already
/// present and valid UTF-8
///
/// # Safety
///
/// Follow the safety requirements of [`env::set_var`].
pub unsafe fn set_required_vars() {
	let vars = [
		("LEPTOS_SITE_ROOT", "site"),
		// this wasn't meant to be a vivid/stasis reference I swear
		("LEPTOS_SITE_PKG_DIR", "_"),
		("LEPTOS_SITE_ADDR", "127.0.0.1:3000")
	];

	for (k, v) in vars {
		if env::var(k).is_err() {
			// SAFETY: caller of this function satisfies the thread safety requirement
			unsafe { env::set_var(k, v) }
		}
	}
}

macro_rules! declare_env_fn {
	{
		$(
			$(#[$meta:meta])*
			$fn_name:ident() -> $env_name:literal
		)*
	} => {
		pub fn check() {
			$(
				let _ = $fn_name();
			)*
		}

		$(
			$(#[$meta])*
			pub fn $fn_name() -> String {
				std::env::var($env_name)
					.expect(concat!("environment variable `", $env_name, "` is not set"))
			}
		)*
	}
}

declare_env_fn! {
	/// fimfic oauth2 client id
	fimfic_client_id() -> "FIMFIC_CLIENT_ID"

	/// fimfic oauth2 client secret
	fimfic_client_secret() -> "FIMFIC_CLIENT_SECRET"

	/// fimfic oauth2 redirect url
	fimfic_oauth_redirect_url() -> "FIMFIC_OAUTH_REDIRECT_URL"

	/// URL to use to connect to postgres
	postgres_url() -> "POSTGRES_URL"
}
