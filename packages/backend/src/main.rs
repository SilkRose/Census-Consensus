use crate::endpoints::{form_page, user_feedback};
use crate::structs::UserType;

pub use self::auth::fimfic_auth;
pub use self::database::Db;
pub use self::fimfic_cfg::FimficCfg;
pub use self::http::HttpClient;

pub use actix_files::Files;
pub use actix_web::middleware::Compress;
pub use actix_web::web::Data;
pub use actix_web::{App as ActixApp, HttpServer};
pub use anyhow::Result;

mod auth;
mod database;
mod endpoints;
mod env_vars;
mod fimfic_cfg;
mod html_templates;
mod http;
mod rand;
mod structs;

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

	let admin_id = env_vars::admin_id().parse::<i32>()?;
	let bearer_token = env_vars::bearer_token();
	let admin = db.get_user(admin_id).await?;
	if let Some(admin) = admin {
		if admin.user_type != UserType::Admin {
			db.update_user_role(admin_id, UserType::Admin).await?;
		}
	} else {
		let admin = http_client.get_fimfic_user(admin_id, &bearer_token).await?;
		db.insert_user(admin_id, &admin.data, UserType::Admin)
			.await?;
	}

	println!("listening on 127.0.0.1:3000");

	let server = HttpServer::new(move || {
		ActixApp::new()
			.service(fimfic_auth)
			.service(form_page)
			.service(user_feedback)
			.service(Files::new("/", "./target/site").index_file("index.html"))
			.app_data(db.clone())
			.app_data(fimfic.clone())
			.app_data(http_client.clone())
			.wrap(Compress::default())
	});

	server.bind("127.0.0.1:3000")?.run().await?;

	Ok(())
}
