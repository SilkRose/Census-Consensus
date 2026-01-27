use std::cmp::Ordering;

use chrono::{DateTime, Utc};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::Type;

fn option_number<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
	D: Deserializer<'de>,
{
	let s: Option<&str> = Option::deserialize(deserializer)?;
	s.filter(|s| !s.is_empty())
		.map(|s| s.parse::<i32>().map_err(D::Error::custom))
		.transpose()
}

fn option_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
	D: Deserializer<'de>,
{
	let s: Option<&str> = Option::deserialize(deserializer)?;
	Ok(s.filter(|s| !s.is_empty()).map(str::to_owned))
}

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
pub struct NewChapter {
	pub title: String,
	pub vote_duration: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChapterEdit {
	pub title: String,
	pub vote_duration: i32,
	#[serde(deserialize_with = "option_string")]
	pub intro_text: Option<String>,
	#[serde(deserialize_with = "option_string")]
	pub outro_text: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Chapter {
	pub id: i32,
	pub title: String,
	pub vote_duration: i32,
	pub minutes_left: Option<i32>,
	pub fimfic_ch_id: Option<i32>,
	pub intro_text: Option<String>,
	pub outro_text: Option<String>,
	pub chapter_order: Option<i32>,
	pub last_edit: DateTime<Utc>,
	pub date_created: DateTime<Utc>,
}

impl Ord for Chapter {
	fn cmp(&self, other: &Self) -> Ordering {
		(self.chapter_order.is_none(), self.chapter_order, self.id).cmp(&(
			other.chapter_order.is_none(),
			other.chapter_order,
			other.id,
		))
	}
}

impl PartialOrd for Chapter {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl PartialEq for Chapter {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

impl Eq for Chapter {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Writing {
	pub id: i32,
	pub writing: String,
	pub created_by: i32,
	pub previous_revision: Option<i32>,
	pub date_created: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Question {
	pub id: i32,
	pub text: String,
	pub r#type: QuestionType,
	pub response_percent: f64,
	pub asked_by: String,
	pub created_by: i32,
	pub claimed_by: Option<i32>,
	pub chapter_id: Option<i32>,
	pub chapter_order: Option<i32>,
	pub latest_writing: Option<i32>,
	pub date_created: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QuestionOption {
	pub id: i32,
	pub question_id: i32,
	pub option_number: i32,
	pub text: String,
	pub writing_id: Option<i32>,
	pub order_rank: i32,
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
