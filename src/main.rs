#![feature(impl_trait_in_assoc_type)]

use chrono::Utc;
use serde_json::Value;
use std::time::Duration;

use crate::endpoints::*;
use crate::structs::{ChapterRevision, UserType};
use crate::utility::parse_options;

pub use self::database::*;
pub use self::error::Result;
pub use self::fimfic_cfg::FimficCfg;
pub use self::http::HttpClient;
pub use self::json::{chapter_json, story_json, story_json_completed};

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
mod json;
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
		let db = db_clone.clone();
		event_control_loop(db, http_client, fimfic).await;
	});

	//                      mane
	server.bind(("0.0.0.0", 6263))?.run().await?;

	Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Tick {
	Continue,
	Skip,
}

async fn event_control_loop(
	mut db: Data<Db>, http_client: Data<HttpClient>, fimfic_cfg: Data<FimficCfg>,
) {
	let mut tick: Result<Tick> = Ok(Tick::Continue);
	loop {
		if matches!(tick, Err(_) | Ok(Tick::Continue)) {
			let time = Utc::now();
			let diff = 60_000 - (time.timestamp_millis() % 60_000) as u64;
			tokio::time::sleep(Duration::from_millis(diff)).await;
		}
		tick = event_control_tick(&mut db, &http_client, &fimfic_cfg).await;
		if let Err(e) = tick.as_ref() {
			eprintln!("Error occurred during event loop: {e}");
		}
	}
}

async fn event_control_tick(
	db: &mut Data<Db>, http_client: &Data<HttpClient>, fimfic_cfg: &Data<FimficCfg>,
) -> Result<Tick> {
	let settings = db.get_settings().await?;
	if settings.start_time.is_none_or(|time| time > Utc::now()) {
		return Ok(Tick::Continue);
	}
	let chapters = db.get_all_chapters().await?;
	let Some(chapter) = chapters
		.iter()
		.find(|c| c.chapter_order.is_some() && c.fimfic_ch_id.is_none())
	else {
		let story = http_client
			.get_story_update(fimfic_cfg, settings.story_id)
			.await?;
		db.insert_story_update(story.data).await?;
		return Ok(Tick::Continue);
	};
	let minutes_left = chapter
		.minutes_left
		.map_or(chapter.vote_duration, |m| m - 1);
	db.update_chapter_minutes_left(chapter.id, Some(minutes_left))
		.await?;
	let publish = minutes_left == 0;
	let final_chapter = db.get_question_count_by_chapter(chapter.id).await? == 0;
	if publish {
		let data = db.get_latest_chapter_revision(chapter.id).await?;
		let json = construct_chapter_json(db, data, final_chapter).await?;
		http_client
			.post_story_chapter(fimfic_cfg, settings.story_id, json)
			.await?;
		return Ok(Tick::Skip);
	}
	let final_update = final_chapter && minutes_left <= -1;
	let json = match (final_chapter, final_update) {
		// normal story updates during live surveys
		(false, false) => {
			todo!()
		}
		// final chapter countdown updates
		(true, false) => {
			let title = format!("{minutes_left} Minutes Until Consensus");
			story_json(settings.story_id, &title, "", "")
		}
		// final story update
		(true, true) => story_json_completed(
			settings.story_id,
			"Census Consensus",
			"The Equestrian Census, redefined.",
			"",
		),
		// should be impossible
		(false, true) => unreachable!(),
	};
	http_client
		.patch_story(fimfic_cfg, settings.story_id, json)
		.await?;
	Ok(Tick::Continue)
}

async fn construct_chapter_json(
	db: &mut Db, data: ChapterRevision, final_chapter: bool,
) -> Result<Value> {
	let json = match final_chapter {
		true => chapter_json(&data.title, &data.outro_text.ok_or("Missing outro!")?, None),
		false => {
			let mut texts = Vec::new();
			if let Some(ref intro) = data.intro_text {
				texts.push(intro.trim());
			}
			let questions = db.get_questions_by_chapter(data.chapter_id).await?;
			for question in questions {
				let data = db.get_latest_question_revision(question.id).await?;
				let options = data.option_writing.ok_or("Missing options!")?;
				let option_tuples = parse_options(&options, &data.question_type);
				let votes = db.get_all_votes_by_question(question.id).await?;
				// insert parsing results here
				texts.push("".trim());
			}
			if let Some(ref outro) = data.outro_text {
				texts.push(outro.trim());
			}
			chapter_json(&data.title, &texts.join("\n\n"), None)
		}
	};
	Ok(json)
}
