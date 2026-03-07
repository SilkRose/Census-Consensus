use chrono::{DateTime, Utc};
use pony::smart_map::SmartMap;
use pony::structs::{option_bool, option_string};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::fmt::{self, Display};

#[derive(Clone, Debug, Deserialize, Serialize, Type, Eq, Hash, PartialEq)]
#[sqlx(type_name = "user_type", rename_all = "snake_case")]
pub enum UserType {
	Admin,
	Writer,
	Voter,
}

impl fmt::Display for UserType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let text = match self {
			UserType::Admin => "Admin",
			UserType::Writer => "Writer",
			UserType::Voter => "Voter",
		};
		write!(f, "{text}")
	}
}

#[derive(Clone, Debug, Deserialize, Serialize, Type, Eq, Hash, PartialEq)]
#[sqlx(type_name = "question_type", rename_all = "snake_case")]
pub enum QuestionType {
	#[serde(rename = "Multiple Choice")]
	MultipleChoice,
	#[serde(rename = "Multi-Select")]
	Multiselect,
	#[serde(rename = "Scale")]
	Scale,
}

impl Display for QuestionType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			QuestionType::MultipleChoice => "Multiple Choice",
			QuestionType::Multiselect => "Multi-Select",
			QuestionType::Scale => "Scale",
		};
		write!(f, "{text}")
	}
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, Hash, PartialEq)]
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
pub struct ChapterTable {
	pub meta: Chapter,
	pub revisions: i64,
	pub questions: i64,
	pub newest_data: ChapterRevision,
	pub oldest_data: ChapterRevision,
	pub oldest_user: User,
	pub newest_user: User,
}

#[derive(Clone, Debug)]
pub struct ChapterData {
	pub meta: Chapter,
	pub data: Vec<ChapterRevision>,
	pub users: SmartMap<i32, User>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QuestionEdit {
	pub question_text: String,
	pub question_type: QuestionType,
	#[serde(default, deserialize_with = "option_bool")]
	pub claimed: bool,
	pub asked_by: String,
	pub response_percent: f64,
	#[serde(deserialize_with = "option_string")]
	pub option_writing: Option<String>,
	#[serde(deserialize_with = "option_string")]
	pub result_writing: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QuestionRevision {
	pub id: i32,
	pub question_text: String,
	pub question_type: QuestionType,
	pub asked_by: String,
	pub response_percent: f64,
	pub option_writing: Option<String>,
	pub result_writing: Option<String>,
	pub question_id: i32,
	pub created_by: i32,
	pub date_created: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Question {
	pub id: i32,
	pub claimed_by: Option<i32>,
	pub chapter_id: Option<i32>,
	pub chapter_order: Option<i32>,
	pub last_edit: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QuestionTable {
	pub meta: Question,
	pub revisions: i64,
	pub options: u32,
	pub outcomes: u32,
	pub claimant: Option<User>,
	pub oldest_data: QuestionRevision,
	pub newest_data: QuestionRevision,
	pub oldest_user: User,
	pub newest_user: User,
}

#[derive(Clone, Debug)]
pub struct QuestionData {
	pub meta: Question,
	pub data: Vec<QuestionRevision>,
	pub users: SmartMap<i32, User>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QuestionChapterId {
	pub chapter_id: Option<i32>,
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

#[repr(transparent)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Population {
	pub inner: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserRoleUpdate {
	pub id: i32,
	pub role: UserType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserBan {
	pub id: i32,
	pub reason: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserFeedback {
	#[serde(deserialize_with = "option_string")]
	pub feedback_private: Option<String>,
	#[serde(deserialize_with = "option_string")]
	pub feedback_public: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Pages {
	Home,
	User,
	About,
	Chapters,
	Questions,
	Feedback,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type, Eq, Hash, PartialEq)]
#[sqlx(type_name = "logo", rename_all = "snake_case")]
pub enum Logo {
	Census,
	Consensus,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LogoStat {
	pub id: i32,
	pub logo: Logo,
	pub user_id: Option<i32>,
	pub ip_addr: Option<String>,
	pub date_created: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OEmbed {
	pub r#type: String,
	pub version: f32,
	pub provider_name: String,
	pub provider_url: String,
	pub title: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub author_name: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub author_url: Option<String>,
	pub cache_age: u32,
	pub html: String,
}
