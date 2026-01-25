use crate::structs::{
	BannedUser, Chapter, Question, QuestionOption, QuestionType, Session, StoryUpdate, Table, User,
	UserType, Vote, Writing,
};
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

#[derive(Clone)]
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
				type = EXCLUDED.type,
				date_last_fetch = now()
			RETURNING
				id, name, pfp_url, type AS "user_type: UserType", feedback_private,
				feedback_public, date_last_fetch, date_joined;"#,
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

	pub async fn get_user(&self, id: i32) -> Result<Option<User>> {
		sqlx::query_as!(
			User,
			r#"SELECT
				id, name, pfp_url, type AS "user_type: UserType", feedback_private,
				feedback_public, date_last_fetch, date_joined
			FROM Users WHERE id = $1 LIMIT 1;"#,
			id,
		)
		.fetch_optional(&self.pool)
		.await
		.context(SELECT_ERROR)
	}

	pub async fn get_all_users(&self) -> Result<Vec<User>> {
		sqlx::query_as!(
			User,
			r#"SELECT
				id, name, pfp_url, type AS "user_type: UserType", feedback_private,
				feedback_public, date_last_fetch, date_joined
			FROM Users;"#,
		)
		.fetch_all(&self.pool)
		.await
		.context(SELECT_ERROR)
	}

	pub async fn update_user_role(&self, id: i32, role: UserType) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Users
			SET
				type = $2
			WHERE id = $1;",
			id,
			role as _
		)
		.execute(&self.pool)
		.await
		.context(UPDATE_ERROR)?
		.rows_affected())
	}

	pub async fn update_user_feedback(
		&self, id: i32, private_msg: Option<String>, public_msg: Option<String>,
	) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Users
			SET
				feedback_private = $2,
				feedback_public = $3
			WHERE id = $1;",
			id,
			private_msg,
			public_msg
		)
		.execute(&self.pool)
		.await
		.context(UPDATE_ERROR)?
		.rows_affected())
	}

	pub async fn delete_user(&self, id: i32) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Users WHERE id = $1;", id)
			.execute(&self.pool)
			.await
			.context(DELETE_ERROR)?
			.rows_affected())
	}

	pub async fn delete_all_users(&self) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Users;")
			.execute(&self.pool)
			.await
			.context(DELETE_ERROR)?
			.rows_affected())
	}

	pub async fn insert_session(
		&self, token: &str, user_id: i32, user_agent: &str,
	) -> Result<Session> {
		sqlx::query_as!(
			Session,
			"INSERT INTO Tokens
				(token, user_id, user_agent)
			VALUES
				($1, $2, $3)
			RETURNING
				token, user_id, user_agent, last_seen, date_created;",
			token,
			user_id,
			user_agent
		)
		.fetch_one(&self.pool)
		.await
		.context(INSERT_ERROR)
	}

	pub async fn get_session_by_token(&self, token: &str) -> Result<Option<Session>> {
		sqlx::query_as!(
			Session,
			"SELECT token, user_id, user_agent, last_seen, date_created
			FROM Tokens WHERE token = $1;",
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
				token, user_id, user_agent, last_seen, date_created
			FROM Tokens
			WHERE user_id = $1;",
			user_id
		)
		.fetch_all(&self.pool)
		.await
		.context(SELECT_ERROR)
	}

	pub async fn get_all_sessions(&self) -> Result<Vec<Session>> {
		sqlx::query_as!(
			Session,
			"SELECT token, user_id, user_agent, last_seen, date_created FROM Tokens;",
		)
		.fetch_all(&self.pool)
		.await
		.context(SELECT_ERROR)
	}

	pub async fn delete_session(&self, token: &str) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Tokens WHERE token = $1;", token)
			.execute(&self.pool)
			.await
			.context(DELETE_ERROR)?
			.rows_affected())
	}

	pub async fn delete_sessions_by_user_id(&self, user_id: i32) -> Result<u64> {
		Ok(
			sqlx::query!("DELETE FROM Tokens WHERE user_id = $1;", user_id)
				.execute(&self.pool)
				.await
				.context(DELETE_ERROR)?
				.rows_affected(),
		)
	}

	pub async fn delete_all_sessions(&self) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Tokens;")
			.execute(&self.pool)
			.await
			.context(DELETE_ERROR)?
			.rows_affected())
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

	pub async fn update_banned_user_reason(&self, id: i32, reason: &str) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Banned_users
			SET
				reason = $2
			WHERE id = $1;",
			id,
			reason
		)
		.execute(&self.pool)
		.await
		.context(UPDATE_ERROR)?
		.rows_affected())
	}

	pub async fn delete_banned_user(&self, id: i32) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Banned_users WHERE id = $1;", id)
			.execute(&self.pool)
			.await
			.context(DELETE_ERROR)?
			.rows_affected())
	}

	pub async fn delete_all_banned_users(&self) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Banned_users;")
			.execute(&self.pool)
			.await
			.context(DELETE_ERROR)?
			.rows_affected())
	}

	pub async fn insert_chapter(&self, title: &str, vote_duration: i32) -> Result<Chapter> {
		sqlx::query_as!(
			Chapter,
			"INSERT INTO Chapters
				(title, vote_duration)
			VALUES
				($1, $2)
			RETURNING
				id, title, vote_duration, minutes_left, fimfic_ch_id,
				intro_text, outro_text, chapter_order, date_created;",
			title,
			vote_duration
		)
		.fetch_one(&self.pool)
		.await
		.context(INSERT_ERROR)
	}

	pub async fn get_chapter(&self, id: i32) -> Result<Option<Chapter>> {
		sqlx::query_as!(
			Chapter,
			"SELECT
				id, title, vote_duration, minutes_left, fimfic_ch_id,
				intro_text, outro_text, chapter_order, date_created
			FROM Chapters WHERE id = $1 LIMIT 1;",
			id,
		)
		.fetch_optional(&self.pool)
		.await
		.context(SELECT_ERROR)
	}

	pub async fn get_chapter_by_order(&self, order: i32) -> Result<Option<Chapter>> {
		sqlx::query_as!(
			Chapter,
			"SELECT
				id, title, vote_duration, minutes_left, fimfic_ch_id,
				intro_text, outro_text, chapter_order, date_created
			FROM Chapters WHERE chapter_order = $1 LIMIT 1;",
			order,
		)
		.fetch_optional(&self.pool)
		.await
		.context(SELECT_ERROR)
	}

	pub async fn get_all_chapters(&self) -> Result<Vec<Chapter>> {
		sqlx::query_as!(
			Chapter,
			"SELECT
				id, title, vote_duration, minutes_left, fimfic_ch_id,
				intro_text, outro_text, chapter_order, date_created
			FROM Chapters;",
		)
		.fetch_all(&self.pool)
		.await
		.context(SELECT_ERROR)
	}

	pub async fn update_chapter_title(&self, id: i32, title: &str) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Chapters
			SET
				title = $2
			WHERE id = $1;",
			id,
			title
		)
		.execute(&self.pool)
		.await
		.context(UPDATE_ERROR)?
		.rows_affected())
	}

	pub async fn update_chapter_vote_duration(&self, id: i32, duration: i32) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Chapters
			SET
				vote_duration = $2
			WHERE id = $1;",
			id,
			duration
		)
		.execute(&self.pool)
		.await
		.context(UPDATE_ERROR)?
		.rows_affected())
	}

	pub async fn update_chapter_minutes_left(&self, id: i32, minutes: i32) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Chapters
			SET
				minutes_left = $2
			WHERE id = $1;",
			id,
			minutes
		)
		.execute(&self.pool)
		.await
		.context(UPDATE_ERROR)?
		.rows_affected())
	}

	pub async fn update_chapter_fimfic_id(&self, id: i32, fimfic_id: i32) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Chapters
			SET
				fimfic_ch_id = $2
			WHERE id = $1;",
			id,
			fimfic_id
		)
		.execute(&self.pool)
		.await
		.context(UPDATE_ERROR)?
		.rows_affected())
	}

	pub async fn update_chapter_intro(&self, id: i32, text: &str) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Chapters
			SET
				intro_text = $2
			WHERE id = $1;",
			id,
			text
		)
		.execute(&self.pool)
		.await
		.context(UPDATE_ERROR)?
		.rows_affected())
	}

	pub async fn update_chapter_outro(&self, id: i32, text: &str) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Chapters
			SET
				outro_text = $2
			WHERE id = $1;",
			id,
			text
		)
		.execute(&self.pool)
		.await
		.context(UPDATE_ERROR)?
		.rows_affected())
	}

	pub async fn update_chapter_ordering(&self, id: i32, order: i32) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Chapters
			SET
				chapter_order = $2
			WHERE id = $1;",
			id,
			order
		)
		.execute(&self.pool)
		.await
		.context(UPDATE_ERROR)?
		.rows_affected())
	}

	pub async fn delete_chapter(&self, id: i32) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Chapters WHERE id = $1;", id)
			.execute(&self.pool)
			.await
			.context(DELETE_ERROR)?
			.rows_affected())
	}

	pub async fn delete_all_chapters(&self) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Chapters;")
			.execute(&self.pool)
			.await
			.context(DELETE_ERROR)?
			.rows_affected())
	}

	pub async fn insert_writing(
		&self, text: &str, creator_id: i32, previous_id: Option<i32>,
	) -> Result<Writing> {
		sqlx::query_as!(
			Writing,
			"INSERT INTO Writings
				(writing, created_by, previous_revision)
			VALUES
				($1, $2, $3)
			RETURNING
				id, writing, created_by, previous_revision, date_created;",
			text,
			creator_id,
			previous_id
		)
		.fetch_one(&self.pool)
		.await
		.context(INSERT_ERROR)
	}

	pub async fn get_writing(&self, id: i32) -> Result<Option<Writing>> {
		sqlx::query_as!(
			Writing,
			"SELECT
				id, writing, created_by, previous_revision, date_created
			FROM Writings WHERE id = $1 LIMIT 1;",
			id,
		)
		.fetch_optional(&self.pool)
		.await
		.context(SELECT_ERROR)
	}

	pub async fn get_all_writings(&self) -> Result<Vec<Writing>> {
		sqlx::query_as!(
			Writing,
			"SELECT
				id, writing, created_by, previous_revision, date_created
			FROM Writings;",
		)
		.fetch_all(&self.pool)
		.await
		.context(SELECT_ERROR)
	}

	pub async fn delete_writing(&self, id: i32) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Writings WHERE id = $1;", id)
			.execute(&self.pool)
			.await
			.context(DELETE_ERROR)?
			.rows_affected())
	}

	pub async fn delete_all_writings(&self) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Writings;")
			.execute(&self.pool)
			.await
			.context(DELETE_ERROR)?
			.rows_affected())
	}

	pub async fn insert_question(
		&self, text: &str, question_type: QuestionType, percent: f64, asked_by: &str,
		creator_id: i32, claiment_id: Option<i32>,
	) -> Result<Question> {
		sqlx::query_as!(
			Question,
			r#"INSERT INTO Questions
				(text, type, response_percent, asked_by, created_by, claimed_by)
			VALUES
				($1, $2, $3, $4, $5, $6)
			RETURNING
				id, text, type AS "type: QuestionType", response_percent, asked_by, created_by,
				claimed_by, chapter_id, chapter_order, latest_writing, date_created;"#,
			text,
			question_type as _,
			percent,
			asked_by,
			creator_id,
			claiment_id
		)
		.fetch_one(&self.pool)
		.await
		.context(INSERT_ERROR)
	}

	pub async fn get_question(&self, id: i32) -> Result<Option<Question>> {
		sqlx::query_as!(
			Question,
			r#"SELECT
				id, text, type AS "type: QuestionType", response_percent, asked_by, created_by,
				claimed_by, chapter_id, chapter_order, latest_writing, date_created
			FROM Questions WHERE id = $1 LIMIT 1;"#,
			id,
		)
		.fetch_optional(&self.pool)
		.await
		.context(SELECT_ERROR)
	}

	pub async fn get_questions_by_chapter(&self, chapter_id: Option<i32>) -> Result<Vec<Question>> {
		sqlx::query_as!(
			Question,
			r#"SELECT
				id, text, type AS "type: QuestionType", response_percent, asked_by, created_by,
				claimed_by, chapter_id, chapter_order, latest_writing, date_created
			FROM Questions WHERE chapter_id = $1;"#,
			chapter_id
		)
		.fetch_all(&self.pool)
		.await
		.context(SELECT_ERROR)
	}

	pub async fn get_questions_by_crator(&self, creator_id: i32) -> Result<Vec<Question>> {
		sqlx::query_as!(
			Question,
			r#"SELECT
				id, text, type AS "type: QuestionType", response_percent, asked_by, created_by,
				claimed_by, chapter_id, chapter_order, latest_writing, date_created
			FROM Questions WHERE created_by = $1;"#,
			creator_id
		)
		.fetch_all(&self.pool)
		.await
		.context(SELECT_ERROR)
	}

	pub async fn get_questions_by_claiment(
		&self, claiment_id: Option<i32>,
	) -> Result<Vec<Question>> {
		sqlx::query_as!(
			Question,
			r#"SELECT
				id, text, type AS "type: QuestionType", response_percent, asked_by, created_by,
				claimed_by, chapter_id, chapter_order, latest_writing, date_created
			FROM Questions WHERE claimed_by = $1;"#,
			claiment_id
		)
		.fetch_all(&self.pool)
		.await
		.context(SELECT_ERROR)
	}

	pub async fn get_all_questions(&self) -> Result<Vec<Question>> {
		sqlx::query_as!(
			Question,
			r#"SELECT
				id, text, type AS "type: QuestionType", response_percent, asked_by, created_by,
				claimed_by, chapter_id, chapter_order, latest_writing, date_created
			FROM Questions;"#,
		)
		.fetch_all(&self.pool)
		.await
		.context(SELECT_ERROR)
	}

	pub async fn update_question_text(&self, id: i32, text: &str) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Questions
			SET
				text = $2
			WHERE id = $1;",
			id,
			text
		)
		.execute(&self.pool)
		.await
		.context(UPDATE_ERROR)?
		.rows_affected())
	}

	pub async fn update_question_repsonse_percent(&self, id: i32, percent: f64) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Questions
			SET
				response_percent = $2
			WHERE id = $1;",
			id,
			percent
		)
		.execute(&self.pool)
		.await
		.context(UPDATE_ERROR)?
		.rows_affected())
	}

	pub async fn update_question_asked_by(&self, id: i32, asked_by: &str) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Questions
			SET
				asked_by = $2
			WHERE id = $1;",
			id,
			asked_by
		)
		.execute(&self.pool)
		.await
		.context(UPDATE_ERROR)?
		.rows_affected())
	}

	pub async fn update_question_claimed_by(
		&self, id: i32, claimed_by: Option<i32>,
	) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Questions
			SET
				claimed_by = $2
			WHERE id = $1;",
			id,
			claimed_by
		)
		.execute(&self.pool)
		.await
		.context(UPDATE_ERROR)?
		.rows_affected())
	}

	pub async fn update_question_chapter_id(
		&self, id: i32, chapter_id: Option<i32>,
	) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Questions
			SET
				chapter_id = $2
			WHERE id = $1;",
			id,
			chapter_id
		)
		.execute(&self.pool)
		.await
		.context(UPDATE_ERROR)?
		.rows_affected())
	}

	pub async fn update_question_chapter_order(
		&self, id: i32, chapter_order: Option<i32>,
	) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Questions
			SET
				chapter_order = $2
			WHERE id = $1;",
			id,
			chapter_order
		)
		.execute(&self.pool)
		.await
		.context(UPDATE_ERROR)?
		.rows_affected())
	}

	pub async fn update_question_latest_writing(
		&self, id: i32, writing_id: Option<i32>,
	) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Questions
			SET
				latest_writing = $2
			WHERE id = $1;",
			id,
			writing_id
		)
		.execute(&self.pool)
		.await
		.context(UPDATE_ERROR)?
		.rows_affected())
	}

	pub async fn delete_question(&self, id: i32) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Questions WHERE id = $1;", id)
			.execute(&self.pool)
			.await
			.context(DELETE_ERROR)?
			.rows_affected())
	}

	pub async fn delete_all_questions(&self) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Questions;")
			.execute(&self.pool)
			.await
			.context(DELETE_ERROR)?
			.rows_affected())
	}

	pub async fn insert_option(
		&self, question_id: i32, option_number: i32, text: &str, order_rank: i32,
	) -> Result<QuestionOption> {
		sqlx::query_as!(
			QuestionOption,
			"INSERT INTO Options
				(question_id, option_number, text, order_rank)
			VALUES
				($1, $2, $3, $4)
			RETURNING
				id, question_id, option_number, text,
				writing_id, order_rank, date_created;",
			question_id,
			option_number,
			text,
			order_rank
		)
		.fetch_one(&self.pool)
		.await
		.context(INSERT_ERROR)
	}

	pub async fn get_option(&self, id: i32) -> Result<Option<QuestionOption>> {
		sqlx::query_as!(
			QuestionOption,
			"SELECT
				id, question_id, option_number, text,
				writing_id, order_rank, date_created
			FROM Options WHERE id = $1 LIMIT 1;",
			id,
		)
		.fetch_optional(&self.pool)
		.await
		.context(SELECT_ERROR)
	}

	pub async fn get_options_by_question(&self, question_id: i32) -> Result<Vec<QuestionOption>> {
		sqlx::query_as!(
			QuestionOption,
			"SELECT
				id, question_id, option_number, text,
				writing_id, order_rank, date_created
			FROM Options WHERE question_id = $1 LIMIT 1;",
			question_id,
		)
		.fetch_all(&self.pool)
		.await
		.context(SELECT_ERROR)
	}

	pub async fn get_all_options(&self) -> Result<Vec<QuestionOption>> {
		sqlx::query_as!(
			QuestionOption,
			"SELECT
				id, question_id, option_number, text,
				writing_id, order_rank, date_created
			FROM Options;",
		)
		.fetch_all(&self.pool)
		.await
		.context(SELECT_ERROR)
	}

	pub async fn update_option_number(&self, id: i32, number: i32) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Options
			SET
				option_number = $2
			WHERE id = $1;",
			id,
			number
		)
		.execute(&self.pool)
		.await
		.context(UPDATE_ERROR)?
		.rows_affected())
	}

	pub async fn update_option_text(&self, id: i32, text: &str) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Options
			SET
				text = $2
			WHERE id = $1;",
			id,
			text
		)
		.execute(&self.pool)
		.await
		.context(UPDATE_ERROR)?
		.rows_affected())
	}

	pub async fn update_option_writing_id(&self, id: i32, writing_id: Option<i32>) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Options
			SET
				writing_id = $2
			WHERE id = $1;",
			id,
			writing_id
		)
		.execute(&self.pool)
		.await
		.context(UPDATE_ERROR)?
		.rows_affected())
	}

	pub async fn update_option_order_rank(&self, id: i32, order_rank: i32) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Options
			SET
				order_rank = $2
			WHERE id = $1;",
			id,
			order_rank
		)
		.execute(&self.pool)
		.await
		.context(UPDATE_ERROR)?
		.rows_affected())
	}

	pub async fn delete_option(&self, id: i32) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Options WHERE id = $1;", id)
			.execute(&self.pool)
			.await
			.context(DELETE_ERROR)?
			.rows_affected())
	}

	pub async fn delete_options_by_question_id(&self, question_id: i32) -> Result<u64> {
		Ok(
			sqlx::query!("DELETE FROM Options WHERE question_id = $1;", question_id)
				.execute(&self.pool)
				.await
				.context(DELETE_ERROR)?
				.rows_affected(),
		)
	}

	pub async fn delete_all_options(&self) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM options;")
			.execute(&self.pool)
			.await
			.context(DELETE_ERROR)?
			.rows_affected())
	}

	pub async fn insert_vote(
		&self, user_id: i32, question_id: i32, option_id: i32,
	) -> Result<Vote> {
		sqlx::query_as!(
			Vote,
			"INSERT INTO Votes
				(voter_id, question_id, option_id)
			VALUES
				($1, $2, $3)
			RETURNING
				voter_id, question_id, option_id, date_created;",
			user_id,
			question_id,
			option_id
		)
		.fetch_one(&self.pool)
		.await
		.context(INSERT_ERROR)
	}

	pub async fn get_all_votes_by_user(&self, user_id: i32) -> Result<Vec<Vote>> {
		sqlx::query_as!(
			Vote,
			"SELECT
				voter_id, question_id, option_id, date_created
			FROM Votes
			WHERE voter_id = $1;",
			user_id
		)
		.fetch_all(&self.pool)
		.await
		.context(SELECT_ERROR)
	}

	pub async fn get_all_votes_by_question(&self, question_id: i32) -> Result<Vec<Vote>> {
		sqlx::query_as!(
			Vote,
			"SELECT
				voter_id, question_id, option_id, date_created
			FROM Votes
			WHERE question_id = $1;",
			question_id
		)
		.fetch_all(&self.pool)
		.await
		.context(SELECT_ERROR)
	}

	pub async fn get_all_votes_by_option(&self, option_id: i32) -> Result<Vec<Vote>> {
		sqlx::query_as!(
			Vote,
			"SELECT
				voter_id, question_id, option_id, date_created
			FROM Votes
			WHERE option_id = $1;",
			option_id
		)
		.fetch_all(&self.pool)
		.await
		.context(SELECT_ERROR)
	}

	pub async fn get_all_votes(&self) -> Result<Vec<Vote>> {
		sqlx::query_as!(
			Vote,
			"SELECT voter_id, question_id, option_id, date_created FROM Votes;",
		)
		.fetch_all(&self.pool)
		.await
		.context(SELECT_ERROR)
	}

	pub async fn delete_votes_by_user(&self, user_id: i32) -> Result<u64> {
		Ok(
			sqlx::query!("DELETE FROM Votes WHERE voter_id = $1;", user_id)
				.execute(&self.pool)
				.await
				.context(DELETE_ERROR)?
				.rows_affected(),
		)
	}

	pub async fn delete_votes_by_option(&self, question_id: i32) -> Result<u64> {
		Ok(
			sqlx::query!("DELETE FROM Votes WHERE question_id = $1;", question_id)
				.execute(&self.pool)
				.await
				.context(DELETE_ERROR)?
				.rows_affected(),
		)
	}

	pub async fn delete_votes_by_question(&self, option_id: i32) -> Result<u64> {
		Ok(
			sqlx::query!("DELETE FROM Votes WHERE option_id = $1;", option_id)
				.execute(&self.pool)
				.await
				.context(DELETE_ERROR)?
				.rows_affected(),
		)
	}

	pub async fn delete_all_votes(&self) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Votes;")
			.execute(&self.pool)
			.await
			.context(DELETE_ERROR)?
			.rows_affected())
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

	pub async fn delete_story_update(&self, date_cached: DateTime<Utc>) -> Result<u64> {
		Ok(sqlx::query!(
			"DELETE FROM Story_updates WHERE date_cached = $1;",
			date_cached
		)
		.execute(&self.pool)
		.await
		.context(DELETE_ERROR)?
		.rows_affected())
	}

	pub async fn delete_story_updates_in_range(
		&self, start: DateTime<Utc>, end: DateTime<Utc>,
	) -> Result<u64> {
		Ok(sqlx::query!(
			"DELETE FROM Story_updates
			WHERE date_cached > $1 AND date_cached > $2;",
			start,
			end
		)
		.execute(&self.pool)
		.await
		.context(DELETE_ERROR)?
		.rows_affected())
	}

	pub async fn delete_all_story_updates(&self) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Story_updates;")
			.execute(&self.pool)
			.await
			.context(DELETE_ERROR)?
			.rows_affected())
	}
}
