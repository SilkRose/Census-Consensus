use anyhow::Result;
use bon::bon;
use chrono::{ DateTime, Local };
use sqlx::{ Pool, Postgres };
use sqlx::postgres::PgPoolOptions;

pub struct Db {
	pool: Pool<Postgres>
}

#[bon]
impl Db {
	pub async fn new(database_url: &str) -> Result<Self> {
		let pool = PgPoolOptions::new()
			.max_connections(16)
			.connect(database_url)
			.await?;

		sqlx::migrate!("db/migrations")
			.run(&pool)
			.await?;

		Ok(Self { pool })
	}

	#[builder]
	pub async fn create_session(
		&self,
		token: &str,
		id: i32
	) -> Result<Session> {
		let query = sqlx::query_file_as!(
			Session,
			"db/queries/insert/token.sql",
			token,
			id
		);

		query
			.fetch_one(&self.pool)
			.await
			.map_err(Into::into)
	}

	pub async fn get_session_by_token(&self, token: &str) -> Result<Option<Session>> {
		let query = sqlx::query_file_as!(
			Session,
			"db/queries/select/token.sql",
			token
		);

		query
			.fetch_optional(&self.pool)
			.await
			.map_err(Into::into)
	}

	#[builder]
	pub async fn create_or_update_user(
		&self,
		id: i32,
		name: &str,
		pfp_url: Option<&str>,
		user_type: UserType
	) -> Result<User> {
		let query = sqlx::query_file_as!(
			User,
			"db/queries/insert/user.sql",
			id,
			name,
			pfp_url,
			user_type as _
		);

		query
			.fetch_one(&self.pool)
			.await
			.map_err(Into::into)
	}
}

#[derive(sqlx::Type)]
#[sqlx(type_name = "user_type", rename_all = "snake_case")]
pub enum UserType {
	Admin,
	Writer,
	Voter
}

#[derive(sqlx::Type)]
#[sqlx(type_name = "question_type", rename_all = "snake_case")]
pub enum QuestionType {
	MultipleChoice,
	Multiselect,
	Scale
}

#[derive(sqlx::Type)]
#[sqlx(type_name = "question_status", rename_all = "snake_case")]
pub enum QuestionStatus {
	Unclaimed,
	Claimed,
	InProgress,
	Written
}

pub struct Session {
	token: String,
	user_id: i32,
	date_created: DateTime<Local>
}

pub struct User {
	id: i32,
	name: String,
	pfp_url: Option<String>,
	user_type: UserType,
	feedback_private: Option<String>,
	feedback_public: Option<String>,
	date_joined: DateTime<Local>
}
