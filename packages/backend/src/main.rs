#![feature(impl_trait_in_assoc_type)]

use crate::endpoints::{get_user_feedback, set_user_feedback};
use crate::structs::UserType;

pub use self::auth::{DevSession, dev_session, fimfic_auth};
pub use self::database::Db;
pub use self::fimfic_cfg::FimficCfg;
pub use self::http::HttpClient;

pub use actix_files::Files;
pub use actix_web::middleware::Compress;
pub use actix_web::web::ThinData as Data;
pub use actix_web::{App as ActixApp, HttpServer};
pub use anyhow::Result;

mod auth;
mod database;
mod endpoints;
mod env_vars;
mod error;
mod fimfic_cfg;
mod html_templates;
mod http;
mod rand;
mod structs;
mod utility;

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
	let db = Data(db);

	let client_id = env_vars::fimfic_client_id();
	let oauth_redirect_url = env_vars::fimfic_oauth_redirect_url();
	let login_url = fimfic_cfg::make_login_url(&client_id, &oauth_redirect_url);
	let fimfic_cfg = FimficCfg::builder()
		.client_id(client_id)
		.client_secret(env_vars::fimfic_client_secret())
		.oauth_redirect_url(oauth_redirect_url)
		.login_url(login_url)
		.build();
	let fimfic = Data(fimfic_cfg);

	let http_client = HttpClient::new()?;
	let http_client = Data(http_client);

	let admin_id = env_vars::admin_id().parse::<i32>()?;
	let bearer_token = env_vars::bearer_token();
	let admin = db.get_user(admin_id).await?;
	let admin_fimfic_user = http_client.get_fimfic_user(admin_id, &bearer_token).await?;

	if let Some(admin) = admin {
		if admin.user_type != UserType::Admin {
			db.update_user_role(admin_id, UserType::Admin).await?;
		}
	} else {
		db.insert_user(admin_id, &admin_fimfic_user.data, UserType::Admin)
			.await?;
	}

	println!("listening on localhost:3000");

	let create_dev_session = env_vars::create_dev_session().is_some();
	let token = rand::gen_auth_token();

	if create_dev_session {
		println!();
		println!("You should unset the `CREATE_DEV_SESSION` environment variable in production.");
		println!("to set a development session, open this link in your browser: http://localhost:3000/dev-session/{token}");
	}

	let server = HttpServer::new(move || {
		ActixApp::new()
			.service(fimfic_auth)
			.service(get_user_feedback)
			.service(set_user_feedback)
			.service(dev_session)
			.service(Files::new("/", "./target/site").index_file("index.html"))
			.app_data(db.clone())
			.app_data(fimfic.clone())
			.app_data(http_client.clone())
			.app_data(Data(create_dev_session.then(|| DevSession::new(
				token.clone(),
				admin_id,
				admin_fimfic_user.data.attributes.avatar.r256.trim_end_matches("-256").into()
			))))
			.wrap(Compress::default())
	});

	server.bind("localhost:3000")?.run().await?;

	Ok(())
}
