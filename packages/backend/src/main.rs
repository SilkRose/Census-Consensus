use crate::html_templates::form_html_template;

pub use self::auth::fimfic_auth;
pub use self::database::Db;
pub use self::fimfic_cfg::FimficCfg;
pub use self::http::HttpClient;

pub use actix_files::Files;
pub use actix_web::middleware::Compress;
pub use actix_web::web::Data;
pub use actix_web::{App as ActixApp, HttpServer};
pub use anyhow::Result;

use actix_web::{HttpRequest, HttpResponse, Responder, get, post};
use std::error::Error;

mod auth;
mod database;
mod env_vars;
mod fimfic_cfg;
mod html_templates;
mod http;
mod rand;
mod structs;

#[get("/form-page")]
async fn form_page() -> Result<impl Responder, Box<dyn Error>> {
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(form_html_template()))
}

#[post("/form-endpoint")]
async fn form_endpoint(_req: HttpRequest, body: String) -> Result<impl Responder, Box<dyn Error>> {
	Ok(HttpResponse::Ok()
		.content_type("text/html; charset=utf-8")
		.body(body))
}

fn main() -> Result<()> {
	// SAFETY: we do this before doing anything else in the program, including
	// before creating an actix runtime, so this should be fine :3
	unsafe { env_vars::set_required_vars() }

	async_main()
}

#[actix_web::main]
async fn async_main() -> Result<()> {
	env_vars::load_dotenv();
	env_vars::check();

	let db = Db::new(&env_vars::database_url()).await?;
	let db = Data::new(db);

	let client_id = env_vars::fimfic_client_id();
	let oauth_redirect_url = env_vars::fimfic_oauth_redirect_url();
	let login_url = fimfic_cfg::make_login_url(&client_id, &oauth_redirect_url);
	let fimfic_cfg = FimficCfg {
		client_id,
		client_secret: env_vars::fimfic_client_secret(),
		oauth_redirect_url,
		login_url,
	};
	let fimfic = Data::new(fimfic_cfg);

	let http_client = HttpClient::new()?;
	let http_client = Data::new(http_client);

	println!("listening on 127.0.0.1:3000");

	let server = HttpServer::new(move || {
		ActixApp::new()
			.service(fimfic_auth)
			.service(form_page)
			.service(form_endpoint)
			.service(Files::new("/", "./target/site").index_file("index.html"))
			.app_data(db.clone())
			.app_data(fimfic.clone())
			.app_data(http_client.clone())
			.wrap(Compress::default())
	});

	server.bind("127.0.0.1:3000")?.run().await?;

	Ok(())
}
