use crate::fimfiction_api::user::UserData;
use crate::structs::{Session, Table, User, UserType};
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

		sqlx::migrate!()
			.run(&pool)
			.await?;

		Ok(Self { pool })
	}

	pub async fn delete_rows(&self, table: Table) -> Result<u64, Box<dyn Error>> {
		let query = format!("DELETE FROM {table};");
		let result = sqlx::query(&query).execute(&self.pool).await?;
		Ok(result.rows_affected())
	}

	pub async fn count_rows(&self, table: Table) -> Result<i64, Box<dyn Error>> {
		let query = format!("SELECT count(*) FROM {table};");
		let count: i64 = sqlx::query_scalar(&query).fetch_one(&self.pool).await?;
		Ok(count)
	}

	pub async fn get_session_by_token(&self, token: &str) -> Result<Option<Session>> {
		sqlx::query_as!(
			Session,
			"SELECT token, user_id, date_created FROM Tokens WHERE token = $1;",
			token
		)
		.fetch_optional(&self.pool)
		.await
		.map_err(|e| format!("database retrieval error.\n{e}").into())
	}

	pub async fn get_all_sessions(&self) -> Result<Vec<Session>> {
		sqlx::query_as!(Session, "SELECT token, user_id, date_created FROM Tokens;",)
			.fetch_all(&self.pool)
			.await
			.map_err(|e| format!("database retrieval error.\n{e}").into())
	}

	pub async fn insert_session(&self, token: &str, user_id: i32) -> Result<Option<Session>> {
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
		.fetch_optional(&self.pool)
		.await
		.map_err(|e| format!("database retrieval error.\n{e}").into())
	}

	pub async fn insert_user(
		&self, id: i32, data: &UserData<i32>, user_type: UserType,
	) -> Result<User, Box<dyn Error>> {
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
		.map_err(|e| format!("database insertion error.\n{e}").into())
	}

	#[builder]
	pub async fn create_session(
		&self,
		token: &str,
		id: i32
	) -> Result<Session> {
		let query = sqlx::query_file_as!(
			Session,
			"queries/insert/token.sql",
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
			"queries/select/token.sql",
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
			"queries/insert/user.sql",
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
