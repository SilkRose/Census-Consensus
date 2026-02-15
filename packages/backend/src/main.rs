#![feature(impl_trait_in_assoc_type)]

use std::sync::{Arc, RwLock};

use crate::endpoints::{
	get_ban_user, get_chapter_edit, get_chapter_new, get_chapter_questions, get_chapter_revisions,
	get_chapters, get_population, get_question_edit, get_question_new, get_question_revisions,
	get_questions, get_sessions, get_update_user, get_update_user_role, get_user_feedback,
	set_ban_user, set_chapter_edit, set_chapter_minutes_left_move, set_chapter_new,
	set_chapter_order, set_chapter_order_move, set_chapter_question_order,
	set_chapter_question_order_move, set_chapter_vote_duration_move, set_population,
	set_question_claim, set_question_edit, set_question_new, set_question_unclaim,
	set_revoke_sessions, set_update_user, set_update_user_role, set_user_feedback,
};
use crate::structs::{Population, UserType};

pub use self::database::*;
pub use self::error::Result;
pub use self::fimfic_cfg::FimficCfg;
pub use self::http::HttpClient;

pub use actix_files::Files;
pub use actix_web::middleware::Compress;
pub use actix_web::web::ThinData as Data;
pub use actix_web::{App as ActixApp, HttpServer};

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

#[actix_web::main]
async fn main() -> Result<()> {
	env_vars::load_dotenv();
	env_vars::check();

	let db = Db::new(&env_vars::database_url()).await?;
	let mut db = Data(db);

	let admin_id = env_vars::admin_id().parse::<i32>()?;
	let bearer_token = env_vars::bearer_token();

	let client_id = env_vars::fimfic_client_id();
	let oauth_redirect_url = env_vars::fimfic_oauth_redirect_url();
	let login_url = fimfic_cfg::make_login_url(&client_id, &oauth_redirect_url);
	let fimfic_cfg = FimficCfg::builder()
		.client_id(client_id)
		.client_secret(env_vars::fimfic_client_secret())
		.oauth_redirect_url(oauth_redirect_url)
		.login_url(login_url)
		.bearer_token(bearer_token.clone())
		.build();
	let fimfic = Data(fimfic_cfg);

	let http_client = HttpClient::new()?;
	let http_client = Data(http_client);

	let admin = match db.get_user_opt(admin_id).await? {
		Some(admin) => {
			if admin.user_type != UserType::Admin {
				db.update_user_role(admin_id, UserType::Admin).await?;
			}

			admin
		}
		None => {
			let admin = http_client.get_fimfic_user(admin_id, &bearer_token).await?;
			db.insert_user(admin_id, &admin.data, UserType::Admin)
				.await?
		}
	};

	let create_dev_session = env_vars::create_dev_session().is_some();
	let token = rand::gen_auth_token();

	let dev_session = create_dev_session.then(|| {
		auth::DevSession::new(
			token.clone(),
			admin_id,
			admin
				.pfp_url
				.unwrap_or_else(|| "https://static.fimfiction.net/images/none_64.png".into()),
		)
	});
	let dev_session = Data(dev_session);

	println!("listening on localhost:3000");

	if create_dev_session {
		println!();
		println!("You should unset the `CREATE_DEV_SESSION` environment variable in production.");
		println!(
			"to set a development session, open this link in your browser: http://localhost:3000/dev-session/{token}"
		);
	}

	let population = env_vars::population()
		.and_then(|pop| pop.parse().ok())
		.unwrap_or(50_240_000);
	let population = Population { inner: population };
	let population = Data(Arc::new(RwLock::new(population)));

	let server = HttpServer::new(move || {
		ActixApp::new()
			.service(auth::fimfic_auth)
			.service(auth::fimfic_auth_logout)
			.service(get_sessions)
			.service(set_revoke_sessions)
			.service(set_update_user)
			.service(get_update_user)
			.service(get_update_user_role)
			.service(set_update_user_role)
			.service(get_ban_user)
			.service(set_ban_user)
			.service(get_user_feedback)
			.service(set_user_feedback)
			.service(get_chapters)
			.service(set_chapter_new)
			.service(get_chapter_new)
			.service(get_chapter_edit)
			.service(set_chapter_edit)
			.service(set_chapter_order)
			.service(set_chapter_order_move)
			.service(set_chapter_vote_duration_move)
			.service(set_chapter_minutes_left_move)
			.service(get_chapter_revisions)
			.service(get_question_new)
			.service(set_question_new)
			.service(get_question_edit)
			.service(set_question_edit)
			.service(get_question_revisions)
			.service(get_population)
			.service(set_population)
			.service(get_chapter_questions)
			.service(set_question_claim)
			.service(set_question_unclaim)
			.service(set_chapter_question_order)
			.service(set_chapter_question_order_move)
			.service(get_questions)
			.service(auth::dev_session)
			.service(Files::new("/", "./target/site").index_file("index.html"))
			.app_data(db.clone())
			.app_data(fimfic.clone())
			.app_data(http_client.clone())
			.app_data(dev_session.clone())
			.app_data(population.clone())
			.wrap(Compress::default())
	});

	server.bind("localhost:3000")?.run().await?;

	Ok(())
}
