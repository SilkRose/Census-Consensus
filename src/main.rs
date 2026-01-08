#[cfg(feature = "ssr")]
mod ssr_imports {
	pub use actix_files::Files;
	pub use actix_web::{ App as ActixApp, HttpServer };
	pub use actix_web::middleware::Compress;
	pub use actix_web::web::Data;
	pub use anyhow::Result;
	pub use april_fools_2026::{ App, env_vars, shell };
	pub use april_fools_2026::database::{ Db, DbData };
	pub use april_fools_2026::server_config::{ Fimfic, FimficData };
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
	let leptos_options = Data::new(conf.leptos_options.clone());
	let site_root = String::from(&*leptos_options.site_root);
	let site_addr = leptos_options.site_addr;
	let routes = generate_route_list(App);

	let db = Db::new(&env_vars::postgres_url()).await?;
	let db = DbData::new(db);

	let fimfic = Fimfic {
		client_id: env_vars::fimfic_client_id(),
		client_secret: env_vars::fimfic_client_secret(),
		oauth_redirect_url: env_vars::fimfic_oauth_redirect_url()
	};
	let fimfic = FimficData::new(fimfic);

	println!("listening on http://{}", &site_addr);

	HttpServer::new(move || {
		ActixApp::new()
			.service(Files::new("/_", format!("{site_root}/_")))
			.service(Files::new("/assets", site_root.clone()))
			.leptos_routes(routes.clone(), {
				let leptos_options = leptos_options.clone();
				move || shell(&leptos_options)
			})
			.app_data(leptos_options.clone())
			.app_data(db.clone())
			.app_data(fimfic.clone())
			.wrap(Compress::default())
	})
		.bind(site_addr)?
		.run()
		.await?;

	Ok(())
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
	panic!("ssr feature is not enabled for server binary; exploding")
}
