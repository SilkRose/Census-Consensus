#[cfg(feature = "ssr")]
mod ssr_imports {
	pub use actix_files::Files;
	pub use actix_web::{ App as ActixApp, HttpServer };
	pub use actix_web::middleware::Compress;
	pub use actix_web::web::Data;
	pub use leptos::config::get_configuration;
	pub use leptos_actix::{ generate_route_list, LeptosRoutes };
	pub use april_fools_2026::{ App, shell };
}
#[cfg(feature = "ssr")]
use ssr_imports::*;

#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let conf = get_configuration(None).unwrap();
	let addr = conf.leptos_options.site_addr;

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
			.app_data(Data::new(leptos_options.to_owned()))
			.wrap(Compress::default())
	})
	.bind(&addr)?
	.run()
	.await
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
	panic!("ssr feature is not enabled for server binary; exploding")

}
