use chrono::{DateTime, Utc};
use pony::http::Request;
use sqlx::{Pool, Postgres};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Story {
	pub id: i32,
	pub title: String,
	pub short_description: String,
	pub description: String,
	pub published: bool,
	pub link: String,
	pub cover_url: Option<String>,
	pub color_hex: String,
	pub views: i32,
	pub total_views: i32,
	pub words: i32,
	pub chapters: i32,
	pub comments: i32,
	pub rating: i32,
	pub likes: i32,
	pub dislikes: i32,
	pub author_id: i32,
	pub date_modified: DateTime<Utc>,
	pub date_updated: DateTime<Utc>,
	pub date_published: DateTime<Utc>,
	pub date_cached: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AppState {
	pub api: Request,
	pub db: Pool<Postgres>,
	pub gc_interval: u64,
	pub cache_max_age: i64,
	pub cache_recache_age: i64,
}

#[derive(Debug, Clone)]
pub struct EmbedData {
	pub title: String,
	pub description: String,
	pub link: String,
	pub color: Option<String>,
	pub cover: Option<String>,
	pub site_name: String,
	pub site_url: String,
	pub errors: Vec<String>,
	pub user_name: Option<String>,
	pub user_link: Option<String>,
	pub html_comment: Option<String>,
	pub open_graph_type: String,
	pub open_graph_property: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Table {
	Users,
	Tokens,
	BannedUsers,
	Chapters,
	Writings,
	Questions,
	Options,
	Votes,
	StoryUpdates,
}

impl fmt::Display for Table {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let text = match self {
			Table::Users => "Users",
			Table::Tokens => "Tokens",
			Table::BannedUsers => "Banned_users",
			Table::Chapters => "Chapters",
			Table::Writings => "Writings",
			Table::Questions => "Questions",
			Table::Options => "Options",
			Table::Votes => "Votes",
			Table::StoryUpdates => "Story_updates",
		};
		write!(f, "{text}")
	}
}

#[derive(sqlx::Type)]
#[sqlx(type_name = "user_type", rename_all = "snake_case")]
pub enum UserType {
	Admin,
	Writer,
	Voter,
}

#[derive(sqlx::Type)]
#[sqlx(type_name = "question_type", rename_all = "snake_case")]
pub enum QuestionType {
	MultipleChoice,
	Multiselect,
	Scale,
}

pub struct Session {
	pub token: String,
	pub user_id: i32,
	pub date_created: DateTime<Utc>,
}

pub struct User {
	pub id: i32,
	pub name: String,
	pub pfp_url: Option<String>,
	pub user_type: UserType,
	pub feedback_private: Option<String>,
	pub feedback_public: Option<String>,
	pub date_joined: DateTime<Utc>,
}

pub struct BannedUser {
	pub id: i32,
	pub reason: String,
	pub date_banned: DateTime<Utc>,
}
