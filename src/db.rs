use anyhow::Result;
use bon::bon;
use chrono::{ DateTime, Local };
use sqlx::{ Pool, Postgres };
use sqlx::postgres::PgPoolOptions;

pub struct Db {
	pool: Pool<Postgres>
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

pub struct User {
	id: i32,
	name: String,
	pfp_url: Option<String>,
	user_type: UserType,
	feedback_private: Option<String>,
	feedback_public: Option<String>,
	date_joined: DateTime<Local>
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
	pub async fn create_or_update_user(
		&self,
		id: i32,
		name: &str,
		pfp_url: Option<&str>,
		user_type: UserType
	) -> Result<User> {
		let query = sqlx::query_file_as!(
			User,
			"db/queries/create_or_update_user.sql",
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
