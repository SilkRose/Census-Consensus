use crate::structs::{BannedUser, Session, StoryUpdate, Table, User, UserType};
use anyhow::{Context as _, Result};
use chrono::{DateTime, Utc};
use pony::fimfiction_api::story::StoryData;
use pony::fimfiction_api::user::UserData;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

const INSERT_ERROR: &str = "database insertion error";
const SELECT_ERROR: &str = "database selection error";
const UPDATE_ERROR: &str = "database updating error";
const DELETE_ERROR: &str = "database deletion error";

// Going to order the database functions as followed:
// Tables: users, tokens, banned users, chapters, questions, writings, options, votes, story updates
// Methods: insert, select one, select many, update, delete one, delete all

pub struct Db {
	pool: Pool<Postgres>,
}

impl Db {
	pub async fn new(database_url: &str) -> Result<Self> {
		let pool = PgPoolOptions::new()
			.max_connections(16)
			.connect(database_url)
			.await?;

		sqlx::migrate!().run(&pool).await?;

		Ok(Self { pool })
	}

	pub async fn count_rows(&self, table: Table) -> Result<i64> {
		let query = format!("SELECT count(*) FROM {table};");
		let count: i64 = sqlx::query_scalar(&query).fetch_one(&self.pool).await?;
		Ok(count)
	}

	pub async fn insert_user(
		&self, id: i32, data: &UserData<i32>, user_type: UserType,
	) -> Result<User> {
		sqlx::query_as!(
			User,
			r#"INSERT INTO Users
				(id, name, pfp_url, type)
			VALUES
				($1, $2, $3, $4)
			ON CONFLICT(id) DO UPDATE SET
				name = EXCLUDED.name,
				pfp_url = EXCLUDED.pfp_url,
				type = EXCLUDED.type
			RETURNING
				id, name, pfp_url, type AS "user_type: UserType",
				feedback_private, feedback_public, date_joined;"#,
			id,
			data.attributes.name.clone(),
			(!data.attributes.avatar.r64.ends_with("none_64.png")).then_some(
				data.attributes
					.avatar
					.r256
					.trim_end_matches("-256")
					.to_string(),
			),
			user_type as _,
		)
		.fetch_one(&self.pool)
		.await
		.context(INSERT_ERROR)
	}

	pub async fn insert_session(&self, token: &str, user_id: i32) -> Result<Session> {
		sqlx::query_as!(
			Session,
			"INSERT INTO Tokens
				(token, user_id)
			VALUES
				($1, $2)
			RETURNING
				token, user_id, date_created;",
			token,
			user_id
		)
		.fetch_one(&self.pool)
		.await
		.context(INSERT_ERROR)
	}

	pub async fn get_session_by_token(&self, token: &str) -> Result<Option<Session>> {
		sqlx::query_as!(
			Session,
			"SELECT token, user_id, date_created FROM Tokens WHERE token = $1;",
			token
		)
		.fetch_optional(&self.pool)
		.await
		.context(SELECT_ERROR)
	}

	pub async fn get_all_user_sessions(&self, user_id: i32) -> Result<Vec<Session>> {
		sqlx::query_as!(
			Session,
			"SELECT
				token, user_id, date_created
			FROM Tokens
			WHERE user_id = $1;",
			user_id
		)
		.fetch_all(&self.pool)
		.await
		.context(SELECT_ERROR)
	}

	pub async fn get_all_sessions(&self) -> Result<Vec<Session>> {
		sqlx::query_as!(Session, "SELECT token, user_id, date_created FROM Tokens;",)
			.fetch_all(&self.pool)
			.await
			.context(SELECT_ERROR)
	}

	pub async fn insert_banned_user(&self, user_id: i32, reason: &str) -> Result<BannedUser> {
		sqlx::query_as!(
			BannedUser,
			"INSERT INTO Banned_users
				(id, reason)
			VALUES
				($1, $2)
			ON CONFLICT(id) DO UPDATE SET
			reason = EXCLUDED.reason
			RETURNING
				id, reason, date_banned;",
			user_id,
			reason
		)
		.fetch_one(&self.pool)
		.await
		.context(INSERT_ERROR)
	}

	pub async fn get_banned_user(&self, id: i32) -> Result<Option<BannedUser>> {
		sqlx::query_as!(
			BannedUser,
			"SELECT
				id, reason, date_banned
			FROM
				Banned_users
			WHERE id = $1
			LIMIT 1;",
			id
		)
		.fetch_optional(&self.pool)
		.await
		.context(SELECT_ERROR)
	}

	pub async fn get_all_banned_users(&self) -> Result<Vec<BannedUser>> {
		sqlx::query_as!(
			BannedUser,
			"SELECT id, reason, date_banned FROM Banned_users;",
		)
		.fetch_all(&self.pool)
		.await
		.context(SELECT_ERROR)
	}

	pub async fn insert_story_update(&self, data: StoryData<i32>) -> Result<StoryUpdate> {
		sqlx::query_as!(
			StoryUpdate,
			"INSERT INTO Story_updates
				(title, short_description, description, views, total_views,
				words, chapters, comments, rating, likes, dislikes)
			VALUES
				($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
			RETURNING
				title, short_description, description, views, total_views,
				words, chapters, comments, rating, likes, dislikes, date_cached;",
			data.attributes.title,
			data.attributes.short_description,
			data.attributes.description,
			data.attributes.num_views,
			data.attributes.total_num_views,
			data.attributes.num_words,
			data.attributes.num_chapters,
			data.attributes.num_comments,
			data.attributes.rating,
			data.attributes.num_likes,
			data.attributes.num_dislikes,
		)
		.fetch_one(&self.pool)
		.await
		.context(INSERT_ERROR)
	}

	pub async fn get_story_updates_in_range(
		&self, start: DateTime<Utc>, end: DateTime<Utc>,
	) -> Result<Vec<StoryUpdate>> {
		sqlx::query_as!(
			StoryUpdate,
			"SELECT
				title, short_description, description, views, total_views,
				words, chapters, comments, rating, likes, dislikes, date_cached
			FROM Story_updates
			WHERE date_cached > $1 AND date_cached < $2;",
			start,
			end
		)
		.fetch_all(&self.pool)
		.await
		.context(SELECT_ERROR)
	}

	pub async fn get_all_story_updates(&self) -> Result<Vec<StoryUpdate>> {
		sqlx::query_as!(
			StoryUpdate,
			"SELECT
				title, short_description, description, views, total_views,
				words, chapters, comments, rating, likes, dislikes, date_cached
			FROM Story_updates;",
		)
		.fetch_all(&self.pool)
		.await
		.context(SELECT_ERROR)
	}
}
