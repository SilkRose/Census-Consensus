#[cfg(feature = "ssr")]
mod ssr_imports {
	pub use actix_files::Files;
	pub use actix_web::{ App as ActixApp, HttpServer };
	pub use actix_web::middleware::Compress;
	pub use actix_web::web::Data;
	pub use anyhow::Result;
	pub use april_fools_2026::{ App, env_vars, server_config, shell };
	pub use april_fools_2026::database::Db;
	pub use leptos::config::get_configuration;
	pub use leptos_actix::{ generate_route_list, LeptosRoutes };
}
#[cfg(feature = "ssr")]
use ssr_imports::*;

#[cfg(feature = "ssr")]
fn main() -> Result<()> {
	// SAFETY: we do this before doing anything else in the program, including
	// before creating an actix runtime, so this should be fine :3
	unsafe { env_vars::set_required_vars() }

	async_main()
}

#[cfg(feature = "ssr")]
#[actix_web::main]
async fn async_main() -> Result<()> {
	env_vars::load_dotenv();
	env_vars::check();

	let conf = get_configuration(None).unwrap();
	let addr = conf.leptos_options.site_addr;

	let db = Db::new(&env_vars::postgres_url()).await?;
	let fimfic = server_config::Fimfic {
		client_id: env_vars::fimfic_client_id(),
		client_secret: env_vars::fimfic_client_secret(),
		oauth_redirect_url: env_vars::fimfic_oauth_redirect_url()
	}.wrap();

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
			.app_data(Data::new(fimfic.clone()))
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
