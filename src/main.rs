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
#[actix_web::main]
async fn main() -> Result<()> {
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
