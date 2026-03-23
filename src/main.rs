#![feature(impl_trait_in_assoc_type)]

use chrono::Utc;
use pony::fimfiction_api::story::StoryApi;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::Duration;

use crate::endpoints::*;
use crate::structs::{Chapter, UserType};

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
mod result_formatter;
mod structs;
mod theme;
mod utility;

#[actix_web::main]
async fn main() -> Result<()> {
	env_vars::load_dotenv();
	env_vars::check();

	let db = Db::new(&env_vars::database_url()).await?;
	let mut db = Data(db);
	let db_clone = db.clone();

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
	let fimfic_clone = fimfic.clone();

	let http_client = HttpClient::new()?;
	let http_client = Data(http_client);
	let http_clone = http_client.clone();

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

	println!("listening at http://localhost:6263");

	if create_dev_session {
		println!();
		println!("You should unset the `CREATE_DEV_SESSION` environment variable in production.");
		println!(
			"to set a development session, open this link in your browser: http://localhost:6263/dev-session/{token}"
		);
	}

	let server = HttpServer::new(move || {
		ActixApp::new()
			.service(oembed)
			.service(get_css)
			.service(get_js)
			.service(get_home)
			.service(get_about)
			.service(get_feedback)
			.service(get_dashboard)
			.service(set_logo_submit)
			.service(auth::fimfic_auth)
			.service(auth::fimfic_auth_logout)
			.service(set_revoke_sessions)
			.service(set_update_user)
			.service(get_user)
			.service(set_update_user_role)
			.service(set_ban_user)
			.service(set_user_feedback)
			.service(get_chapters)
			.service(set_chapter_new)
			.service(get_chapter_edit)
			.service(set_chapter_edit)
			.service(set_chapter_order)
			.service(set_chapter_order_move)
			.service(set_chapter_vote_duration_move)
			.service(set_chapter_minutes_left_move)
			.service(get_chapter_revisions)
			.service(set_question_new)
			.service(get_question_edit)
			.service(set_question_edit)
			.service(get_question_revisions)
			.service(get_question_preview)
			.service(get_chapter_questions)
			.service(set_question_claim)
			.service(set_question_unclaim)
			.service(set_chapter_question_order)
			.service(set_chapter_question_order_move)
			.service(set_vote_duration)
			.service(set_population)
			.service(set_story_id)
			.service(set_reset)
			.service(set_start_time)
			.service(set_start_time_reset)
			.service(get_questions)
			.service(auth::dev_session)
			.service(Files::new("/assets", "./assets"))
			.app_data(db.clone())
			.app_data(fimfic.clone())
			.app_data(http_client.clone())
			.app_data(dev_session.clone())
			.wrap(Compress::default())
	});

	tokio::task::spawn_local(async move {
		let http_client = http_clone.clone();
		let fimfic = fimfic_clone.clone();
		let mut db = db_clone.clone();
		loop {
			let time = Utc::now();
			let diff = 60_000 - (time.timestamp_millis() % 60_000) as u64;
			tokio::time::sleep(Duration::from_millis(diff)).await;
			let settings = match db.get_settings().await {
				Ok(settings) => settings,
				Err(e) => {
					eprintln!("Error occurred during event loop: {e}");
					continue;
				}
			};
			if let Some(start_time) = settings.start_time
				&& start_time <= Utc::now()
			{
				let chapters = match db.get_all_chapters().await {
					Ok(chapters) => chapters,
					Err(e) => {
						eprintln!("Error occurred during event loop: {e}");
						continue;
					}
				};
				let active_chapter = chapters
					.iter()
					.find(|c| c.chapter_order.is_some() && c.fimfic_ch_id.is_none());
				if let Some(chapter) = active_chapter {
					let minutes_left = chapter
						.minutes_left
						.map_or(chapter.vote_duration, |m| m - 1);
					if let Err(e) = db
						.update_chapter_minutes_left(chapter.id, Some(minutes_left))
						.await
					{
						eprintln!("Error occurred during event loop: {e}");
						continue;
					};
					if minutes_left <= 0 {
						// publish chapter
					}
					// update story
				}
			} else {
				let endpoint = format!(
					"https://www.fimfiction.net/api/v2/stories/{}",
					settings.story_id
				);
				if let Ok(story) = get_story_update(&http_client, &fimfic, endpoint).await
					&& let Err(e) = db.insert_story_update(story.data).await
				{
					eprintln!("Error occurred during event loop: {e}");
				};
			}
		}
	});

	//                      mane
	server.bind(("0.0.0.0", 6263))?.run().await?;

	Ok(())
}

async fn get_story_update(
	client: &Data<HttpClient>, fimfic_cfg: &Data<FimficCfg>, endpoint: String,
) -> Result<StoryApi<i32>> {
	Ok(client
		.get(endpoint, Some(&fimfic_cfg.bearer_token))
		.send()
		.await?
		.json::<StoryApi<i32>>()
		.await?)
}

fn chapter_json(title: &str, content: &str, authors_note: Option<&str>) -> Value {
	// Construct the json for chapters.
	json!({
		 "data": {
			  "type": "chapter",
			  "attributes": {
					"title": title,
					"content": content,
					"authors_note": authors_note.unwrap_or_default(),
					"published": true
			  }
		 }
	})
}

fn story_json(id: u32, title: &str, short_description: &str, description: &str) -> Value {
	// Construct the json for story updates.
	json!({
		"data": {
			"id": id,
			"attributes": {
				"title": title,
				"description": description,
				"short_description": short_description
			}
		}
	})
}

fn story_json_optional(
	id: u32, title: &Option<String>, short_description: &Option<String>,
	description: &Option<String>, completion_status: &Option<String>,
) -> String {
	let mut attributes = HashMap::new();
	if let Some(name) = title {
		attributes.insert("title", name);
	}
	if let Some(short_desc) = short_description {
		attributes.insert("short_description", short_desc);
	}
	if let Some(desc) = description {
		attributes.insert("description", desc);
	}
	if let Some(status) = completion_status {
		attributes.insert("completion_status", status);
	}
	let json = json!({
		"data": {
			"id": id,
			"attributes": serde_json::to_value(attributes).unwrap()
		}
	});
	serde_json::to_string(&json).unwrap()
}
