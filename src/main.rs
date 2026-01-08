#[cfg(feature = "ssr")]
mod ssr_imports {
	pub use actix_files::Files;
	pub use actix_web::{ App as ActixApp, HttpServer };
	pub use actix_web::middleware::Compress;
	pub use actix_web::web::Data;
	pub use anyhow::Result;
	pub use april_fools_2026::{ App, shell };
	pub use april_fools_2026::database::Db;
	pub use leptos::config::get_configuration;
	pub use leptos_actix::{ generate_route_list, LeptosRoutes };
	pub use std::env;
}
#[cfg(feature = "ssr")]
use ssr_imports::*;

#[cfg(feature = "ssr")]
fn main() -> Result<()> {
	// SAFETY: we do this before doing anything else in the program, including
	// before creating an actix runtime, so this should be fine :3
	unsafe {
		set_vars_if_not_present([
			("LEPTOS_SITE_ROOT", "site"),
			// this wasn't meant to be a vivid/stasis reference I swear
			("LEPTOS_SITE_PKG_DIR", "_"),
			("LEPTOS_SITE_ADDR", "127.0.0.1:3000")
		])
	}

	async_main()
}

#[cfg(feature = "ssr")]
#[actix_web::main]
async fn async_main() -> Result<()> {
	if let Err(err) = dotenvy::dotenv() {
		eprintln!("dotenv failed to load: {err:?}");
	}

	let conf = get_configuration(None).unwrap();
	let addr = conf.leptos_options.site_addr;

	let database_url = env::var("POSTGRES_URL").expect("POSTGRES_URL is not set");
	let db = Db::new(&database_url).await?;

	println!("listening on http://{}", &addr);

	HttpServer::new(move || {
		// Generate the list of routes in your Leptos App
		let routes = generate_route_list(App);
		let leptos_options = &conf.leptos_options;
		let site_root = leptos_options.site_root.clone().to_string();

		ActixApp::new()
			// serve JS/WASM/CSS from `pkg`
			.service(Files::new("/_", format!("{site_root}/_")))
			// serve other assets from the `assets` directory
			.service(Files::new("/assets", site_root))
			.leptos_routes(routes, {
				let leptos_options = leptos_options.clone();
				move || shell(&leptos_options)
			})
			.app_data(Data::new(leptos_options.clone()))
			.app_data(Data::new(db.clone()))
			.wrap(Compress::default())
	})
		.bind(&addr)?
		.run()
		.await?;

	Ok(())
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
	panic!("ssr feature is not enabled for server binary; exploding")
}

/// Sets the provided environment variables, if they are not already present and
/// valid UTF-8
///
/// # Safety
///
/// Follow the safety requirements of [`env::set_var`].
#[cfg(feature = "ssr")]
unsafe fn set_vars_if_not_present(vars: impl IntoIterator<Item = (&'static str, &'static str)>) {
	for (k, v) in vars {
		if env::var(k).is_err() {
			// SAFETY: caller of this function satisfies the thread safety requirement
			unsafe { env::set_var(k, v) }
		}
	}
}
