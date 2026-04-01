#![feature(impl_trait_in_assoc_type)]

use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

use crate::endpoints::*;
use crate::structs::{Chapter, ChapterRevision, OptionType, Settings, UserType};
use crate::utility::{construct_question_data, parse_options};

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

	let http_client = HttpClient::new().await?;
	let http_client = Data(http_client);
	let http_clone = http_client.clone();
	let http_clone_clone = http_clone.clone();

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
			.service(get_chapter_survey)
			.service(set_chapter_submit)
			.service(get_chapter_preview)
			.service(get_chapter_preview_random)
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

	tokio::task::spawn_local(async move {
		let mut http_client = http_clone_clone.clone();
		loop {
			let time = Utc::now();
			let diff = 60_000 - (time.timestamp_millis() % 60_000) as u64;
			tokio::time::sleep(Duration::from_millis(diff)).await;
			if http_client.cf_data.created + Duration::from_mins(30) <= time {
				let _ = http_client.refresh_cookie().await;
			}
		}
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
	let Some(chapter) = db.get_active_chapter().await? else {
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
	let question_count = db.get_question_count_by_chapter(chapter.id).await?;
	let publish = minutes_left == 0;
	let final_chapter = question_count == 0;
	let final_update = publish && final_chapter;
	if publish {
		let data = db.get_latest_chapter_revision(chapter.id).await?;
		let json = construct_chapter_json()
			.db(db)
			.settings(&settings)
			.data(data)
			.final_chapter(final_chapter)
			.call()
			.await?;
		let res = http_client
			.post_story_chapter(fimfic_cfg, settings.story_id, json)
			.await?;
		let fimfic_id = res.data.id.parse::<i32>().ok();
		db.update_chapter_fimfic_id(chapter.id, fimfic_id).await?;
		if !final_update {
			return Ok(Tick::Skip);
		}
	}
	let json = construct_story_json()
		.db(db)
		.settings(&settings)
		.chapter(&chapter)
		.final_chapter(final_chapter)
		.final_update(final_update)
		.minutes_left(minutes_left)
		.question_count(question_count)
		.call()
		.await?;
	http_client
		.patch_story(fimfic_cfg, settings.story_id, json)
		.await?;
	Ok(Tick::Continue)
}

#[bon::builder]
async fn construct_chapter_json(
	db: &mut Db, settings: &Settings, data: ChapterRevision, final_chapter: bool,
) -> Result<Value> {
	let json = match final_chapter {
		true => chapter_json(&data.title, &data.outro_text.ok_or("Missing outro!")?, None),
		false => {
			let mut texts = Vec::new();
			if let Some(ref intro) = data.intro_text {
				texts.push(intro.trim().to_string());
			}
			let questions = db.get_questions_by_chapter(data.chapter_id).await?;
			for question in questions {
				let data = db.get_latest_question_revision(question.id).await?;
				let options = data.option_writing.clone().ok_or("Missing options!")?;
				let option_tuples = parse_options(&options, &data.question_type);
				let votes = db.get_all_votes_by_question(question.id).await?;
				let buckets = votes.chunk_by(|a, b| a.option_id == b.option_id);
				let mut results = HashMap::new();
				let mut total_count = 0;
				for bucket in buckets {
					let mut count = 0;
					for vote in bucket {
						let banned = db.get_banned_user(vote.voter_id).await?;
						if banned.is_none() {
							count += 1;
						}
					}
					results.insert(bucket[0].option_id.clone(), count);
					total_count += count;
				}
				for (id, _) in &option_tuples {
					if !results.contains_key(id) {
						results.insert(id.clone(), 0);
					}
				}
				let options = OptionType::Count((results, total_count));
				let question_data = construct_question_data()
					.meta(question)
					.data(data)
					.option_texts(option_tuples)
					.option_data(options)
					.population(settings.population)
					.call();
				let (preview, errors) = result_formatter::format(&question_data);
				texts.push(preview.trim().to_string());
				for error in errors {
					eprintln!("Error in parsing question: {error}")
				}
			}
			if let Some(ref outro) = data.outro_text {
				texts.push(outro.trim().to_string());
			}
			let authors_note = "To participate in this event, please visit our [url=https://census.silkrose.dev/]custom survey site[/url].";
			chapter_json(&data.title, &texts.join("\n\n"), Some(authors_note))
		}
	};
	Ok(json)
}

#[bon::builder]
async fn construct_story_json(
	db: &mut Db, settings: &Settings, chapter: &Chapter, final_chapter: bool, final_update: bool,
	minutes_left: i32, question_count: i64,
) -> Result<Value> {
	let json = match (final_chapter, final_update) {
		// normal story updates during live surveys
		(false, false) => {
			let title = format!("Survey ends in {minutes_left} Minutes!");
			let elapsed = (chapter.vote_duration - minutes_left + 1) as f64;
			let fraction = elapsed / chapter.vote_duration as f64;
			let order = (question_count as f64 * fraction).ceil() as i32;
			let question = db
				.get_question_by_chapter_and_order(chapter.id, order)
				.await?;
			let Some(question) = question else {
				return Err(
					"Chapter question order missing from database! Check math again.".into(),
				);
			};
			let data = db.get_latest_question_revision(question.id).await?;
			let mut short_desc = format!("{} asked, \"{}\"", data.asked_by, data.question_text);
			if short_desc.len() > 250 {
				short_desc = format!("{}…", &short_desc[..=249])
			}
			story_json(settings.story_id, &title, &short_desc)
		}
		// final chapter countdown updates
		(true, false) => {
			let title = format!("{minutes_left} Minutes Until Consensus");
			story_json(
				settings.story_id,
				&title,
				"The Equestrian Census, redefined.",
			)
		}
		// final story update
		(true, true) => story_json_completed(
			settings.story_id,
			"Census Consensus",
			"The Equestrian Census, redefined.",
		),
		// should be impossible
		(false, true) => unreachable!(),
	};
	Ok(json)
}
