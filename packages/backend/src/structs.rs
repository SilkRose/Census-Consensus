use chrono::{DateTime, Utc};
use pony::structs::option_string;
use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Clone, Debug, Deserialize, Serialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "user_type", rename_all = "snake_case")]
pub enum UserType {
	Admin,
	Writer,
	Voter,
}

impl UserType {
	pub fn from_str(value: &str) -> Option<Self> {
		match value {
			"admin" => Some(UserType::Admin),
			"writer" => Some(UserType::Writer),
			"voter" => Some(UserType::Voter),
			_ => None,
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[sqlx(type_name = "question_type", rename_all = "snake_case")]
pub enum QuestionType {
	MultipleChoice,
	Multiselect,
	Scale,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
	pub id: i32,
	pub name: String,
	pub pfp_url: Option<String>,
	pub user_type: UserType,
	pub feedback_private: Option<String>,
	pub feedback_public: Option<String>,
	pub date_last_fetch: DateTime<Utc>,
	pub date_joined: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Session {
	pub token: String,
	pub user_id: i32,
	pub user_agent: String,
	pub last_seen: DateTime<Utc>,
	pub date_created: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BannedUser {
	pub id: i32,
	pub reason: String,
	pub date_banned: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChapterEdit {
	pub title: String,
	#[serde(deserialize_with = "option_string")]
	pub intro_text: Option<String>,
	#[serde(deserialize_with = "option_string")]
	pub outro_text: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Chapter {
	pub id: i32,
	pub vote_duration: i32,
	pub minutes_left: Option<i32>,
	pub fimfic_ch_id: Option<i32>,
	pub chapter_order: Option<i32>,
	pub last_edit: DateTime<Utc>,
	pub date_created: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChapterRevision {
	pub id: i32,
	pub title: String,
	pub intro_text: Option<String>,
	pub outro_text: Option<String>,
	pub chapter_id: i32,
	pub created_by: i32,
	pub date_created: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChapterDatum {
	pub meta: Chapter,
	pub data: ChapterRevision,
	pub user: User,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChapterDatums {
	pub meta: Chapter,
	pub data: Vec<ChapterRevision>,
	pub user: Vec<User>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WritingEdit {
	pub question_text: String,
	#[serde(deserialize_with = "option_string")]
	pub option_writing: Option<String>,
	#[serde(deserialize_with = "option_string")]
	pub result_writing: Option<String>,
	pub asked_by: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QuestionEdit {
	pub r#type: QuestionType,
	pub response_percent: f64,
	pub latest_writing: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Question {
	pub id: i32,
	pub r#type: QuestionType,
	pub response_percent: f64,
	pub created_by: i32,
	pub claimed_by: Option<i32>,
	pub chapter_id: Option<i32>,
	pub chapter_order: Option<i32>,
	pub date_created: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QuestionWriting {
	pub id: i32,
	pub question_text: String,
	pub option_writing: Option<String>,
	pub result_writing: Option<String>,
	pub asked_by: String,
	pub created_by: i32,
	pub question_id: i32,
	pub date_created: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Vote {
	pub voter_id: i32,
	pub question_id: i32,
	pub option_id: i32,
	pub date_created: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StoryUpdate {
	pub title: String,
	pub short_description: String,
	pub description: String,
	pub views: i32,
	pub total_views: i32,
	pub words: i32,
	pub chapters: i32,
	pub comments: i32,
	pub rating: i32,
	pub likes: i32,
	pub dislikes: i32,
	pub date_cached: DateTime<Utc>,
}
