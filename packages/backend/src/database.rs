use crate::error::Result;
use crate::structs::{
	BannedUser, Chapter, ChapterEdit, ChapterRevision, ChapterTable, Question, QuestionEdit,
	QuestionRevision, QuestionTable, QuestionType, Session, StoryUpdate, User, UserType, Vote,
};
use crate::utility::{count_options, count_outcomes};
use chrono::{DateTime, Utc};
use pony::fimfiction_api::story::StoryData;
use pony::fimfiction_api::user::UserData;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

fn insert_err(err: sqlx::Error) -> String {
	format!("database insertion error:\n{err}")
}

fn select_err(err: sqlx::Error) -> String {
	format!("database selection error:\n{err}")
}

fn update_err(err: sqlx::Error) -> String {
	format!("database updating error:\n{err}")
}

fn delete_err(err: sqlx::Error) -> String {
	format!("database deletion error:\n{err}")
}

fn count_err() -> &'static str {
	"database counting error"
}

fn exist_err() -> &'static str {
	"database exist error"
}

fn db_expect() -> &'static str {
	"database constraints means this resource will always be present in the database."
}

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

	pub async fn transaction(&self) -> Result<DbTransaction<'_>> {
		let tx = self.pool.begin().await?;
		Ok(DbTransaction { tx })
	}

	pub async fn insert_chapter(&mut self, data: ChapterEdit, user: User) -> Result<Chapter> {
		let mut tx = self.transaction().await?;
		let meta = tx.create_chapter().await?;
		tx.insert_chapter_revision(data, user.id, meta.id).await?;
		tx.commit().await?;
		Ok(meta)
	}

	pub async fn get_chapters_table(&mut self) -> Result<Vec<ChapterTable>> {
		let mut tx = self.transaction().await?;
		let chapters = tx.get_all_chapters().await?;
		let mut data = vec![];
		for chapter in chapters {
			let last_data = tx.get_oldest_chapter_revision(chapter.id).await?;
			let first_data = tx.get_latest_chapter_revision(chapter.id).await?;
			let last_user = tx.get_user(last_data.created_by).await?;
			let first_user = tx.get_user(first_data.created_by).await?;
			let revisions = tx.get_chapter_revisions_count_by_id(chapter.id).await?;
			let questions = tx.get_question_count_by_chapter(chapter.id).await?;
			let table_data = ChapterTable {
				meta: chapter,
				revisions,
				questions,
				first_data,
				last_data,
				first_user,
				last_user,
			};
			data.push(table_data);
		}
		tx.commit().await?;
		Ok(data)
	}

	pub async fn swap_chapters_by_order(
		&mut self, self_id: i32, other_id: i32, order: i32, movement: i32,
	) -> Result<()> {
		let mut tx = self.transaction().await?;
		tx.update_chapter_order_none(self_id).await?;
		tx.update_chapter_order(other_id, order).await?;
		tx.update_chapter_order(self_id, order + movement).await?;
		tx.commit().await
	}

	pub async fn remove_chapter_order(&mut self, id: i32, mut order: i32) -> Result<()> {
		let mut tx = self.transaction().await?;
		tx.update_chapter_order_none(id).await?;
		let chapters = tx.get_chapters_after_order(order).await?;
		for chapter in chapters {
			tx.update_chapter_order(chapter.id, order).await?;
			order += 1;
		}
		tx.commit().await
	}

	pub async fn insert_question(&mut self, data: QuestionEdit, user: User) -> Result<Question> {
		let mut tx = self.transaction().await?;
		let meta = tx.create_question().await?;
		tx.insert_question_revision(data, meta.id, user.id).await?;
		tx.commit().await?;
		Ok(meta)
	}

	pub async fn get_chapter_questions_table(
		&mut self, chapter_id: i32,
	) -> Result<Vec<QuestionTable>> {
		let mut tx = self.transaction().await?;
		let questions = tx.get_questions_for_table(chapter_id).await?;
		let mut data = vec![];
		for question in questions {
			let id = question.id;
			let claiment = match question.claimed_by {
				Some(id) => Some(tx.get_user(id).await?),
				None => None,
			};
			let last_data = tx.get_oldest_question_revision(question.id).await?;
			let first_data = tx.get_latest_question_revision(question.id).await?;
			let last_user = tx.get_user(last_data.created_by).await?;
			let first_user = tx.get_user(first_data.created_by).await?;
			let table_data = QuestionTable {
				meta: question,
				revisions: tx.get_question_revision_count(id).await?,
				options: count_options(&last_data.option_writing.clone().unwrap_or_default()),
				outcomes: count_outcomes(&last_data.result_writing.clone().unwrap_or_default()),
				claiment,
				first_data,
				last_data,
				first_user,
				last_user,
			};
			data.push(table_data);
		}
		tx.commit().await?;
		Ok(data)
	}

	pub async fn get_questions_table(&mut self) -> Result<Vec<QuestionTable>> {
		let mut tx = self.transaction().await?;
		let questions = tx.get_all_questions().await?;
		let mut data = vec![];
		for question in questions {
			let id = question.id;
			let claiment = match question.claimed_by {
				Some(id) => Some(tx.get_user(id).await?),
				None => None,
			};
			let last_data = tx.get_oldest_question_revision(question.id).await?;
			let first_data = tx.get_latest_question_revision(question.id).await?;
			let last_user = tx.get_user(last_data.created_by).await?;
			let first_user = tx.get_user(first_data.created_by).await?;
			let table_data = QuestionTable {
				meta: question,
				revisions: tx.get_question_revision_count(id).await?,
				options: count_options(&last_data.option_writing.clone().unwrap_or_default()),
				outcomes: count_outcomes(&last_data.result_writing.clone().unwrap_or_default()),
				claiment,
				first_data,
				last_data,
				first_user,
				last_user,
			};
			data.push(table_data);
		}
		tx.commit().await?;
		Ok(data)
	}

	pub async fn swap_questions_by_order(
		&mut self, self_id: i32, other_id: i32, order: i32, movement: i32,
	) -> Result<()> {
		let mut tx = self.transaction().await?;
		tx.update_question_chapter_order(self_id, 0).await?;
		tx.update_question_chapter_order(other_id, order).await?;
		tx.update_question_chapter_order(self_id, order + movement)
			.await?;
		tx.commit().await
	}
}

impl DbExecutor for Db {
	type Executor<'c> = &'c Pool<Postgres>;

	fn executor(&mut self) -> &Pool<Postgres> {
		&self.pool
	}
}

pub struct DbTransaction<'c> {
	tx: sqlx::Transaction<'c, Postgres>,
}

impl<'c> DbTransaction<'c> {
	async fn commit(self) -> Result<()> {
		self.tx.commit().await?;
		Ok(())
	}
}

impl<'c> DbExecutor for DbTransaction<'c> {
	type Executor<'c2>
		= &'c2 mut sqlx::PgConnection
	where
		Self: 'c2;

	fn executor(&mut self) -> &mut sqlx::PgConnection {
		&mut self.tx
	}
}

#[expect(
	async_fn_in_trait,
	reason = "we don't need any implemented auto traits"
)]
pub trait DbExecutor {
	type Executor<'c>: sqlx::Executor<'c, Database = Postgres>
	where
		Self: 'c;

	fn executor(&mut self) -> Self::Executor<'_>;

	async fn insert_user(
		&mut self, id: i32, data: &UserData<i32>, user_type: UserType,
	) -> Result<User> {
		Ok(sqlx::query_as!(
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
		.fetch_one(self.executor())
		.await
		.map_err(insert_err)?)
	}

	async fn get_user_opt(&mut self, id: i32) -> Result<Option<User>> {
		Ok(sqlx::query_as!(
			User,
			r#"SELECT
				id, name, pfp_url, type AS "user_type: UserType", feedback_private,
				feedback_public, date_last_fetch, date_joined
			FROM Users WHERE id = $1 LIMIT 1;"#,
			id,
		)
		.fetch_optional(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_user(&mut self, id: i32) -> Result<User> {
		Ok(self.get_user_opt(id).await?.ok_or_else(db_expect)?)
	}

	async fn get_all_users(&mut self) -> Result<Vec<User>> {
		Ok(sqlx::query_as!(
			User,
			r#"SELECT
				id, name, pfp_url, type AS "user_type: UserType", feedback_private,
				feedback_public, date_last_fetch, date_joined
			FROM Users;"#,
		)
		.fetch_all(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_users_count(&mut self) -> Result<i64> {
		Ok(sqlx::query!("SELECT count(*) FROM Users;")
			.fetch_one(self.executor())
			.await
			.map_err(select_err)?
			.count
			.ok_or_else(count_err)?)
	}

	async fn update_user_role(&mut self, id: i32, role: UserType) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Users
			SET
				type = $2
			WHERE id = $1;",
			id,
			role as _
		)
		.execute(self.executor())
		.await
		.map_err(update_err)?
		.rows_affected())
	}

	async fn update_user_feedback(
		&mut self, id: i32, private_msg: Option<String>, public_msg: Option<String>,
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
		.execute(self.executor())
		.await
		.map_err(update_err)?
		.rows_affected())
	}

	async fn delete_user(&mut self, id: i32) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Users WHERE id = $1;", id)
			.execute(self.executor())
			.await
			.map_err(delete_err)?
			.rows_affected())
	}

	async fn delete_all_users(&mut self) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Users;")
			.execute(self.executor())
			.await
			.map_err(delete_err)?
			.rows_affected())
	}

	async fn insert_session(
		&mut self, token: &str, user_id: i32, user_agent: &str,
	) -> Result<Session> {
		Ok(sqlx::query_as!(
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
		.fetch_one(self.executor())
		.await
		.map_err(insert_err)?)
	}

	/// Use when you need to get the session without updating the last seen time.
	async fn get_session_by_token(&mut self, token: &str) -> Result<Option<Session>> {
		Ok(sqlx::query_as!(
			Session,
			"SELECT token, user_id, user_agent, last_seen, date_created
			FROM Tokens WHERE token = $1;",
			token
		)
		.fetch_optional(self.executor())
		.await
		.map_err(select_err)?)
	}

	/// Use when you need to get the session and update the last seen time.
	async fn update_session_last_seen(&mut self, token: &str) -> Result<Option<Session>> {
		Ok(sqlx::query_as!(
			Session,
			"UPDATE Tokens SET last_seen = now() WHERE token = $1
			RETURNING
				token, user_id, user_agent, last_seen, date_created;",
			token
		)
		.fetch_optional(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_all_user_sessions(&mut self, user_id: i32) -> Result<Vec<Session>> {
		Ok(sqlx::query_as!(
			Session,
			"SELECT
				token, user_id, user_agent, last_seen, date_created
			FROM Tokens
			WHERE user_id = $1;",
			user_id
		)
		.fetch_all(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_all_sessions(&mut self) -> Result<Vec<Session>> {
		Ok(sqlx::query_as!(
			Session,
			"SELECT token, user_id, user_agent, last_seen, date_created FROM Tokens;",
		)
		.fetch_all(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_sessions_count(&mut self) -> Result<i64> {
		Ok(sqlx::query!("SELECT count(*) FROM Tokens;")
			.fetch_one(self.executor())
			.await
			.map_err(select_err)?
			.count
			.ok_or_else(count_err)?)
	}

	async fn delete_session(&mut self, token: &str) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Tokens WHERE token = $1;", token)
			.execute(self.executor())
			.await
			.map_err(delete_err)?
			.rows_affected())
	}

	async fn delete_sessions_by_user_id(&mut self, user_id: i32) -> Result<u64> {
		Ok(
			sqlx::query!("DELETE FROM Tokens WHERE user_id = $1;", user_id)
				.execute(self.executor())
				.await
				.map_err(delete_err)?
				.rows_affected(),
		)
	}

	async fn delete_all_sessions(&mut self) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Tokens;")
			.execute(self.executor())
			.await
			.map_err(delete_err)?
			.rows_affected())
	}

	async fn insert_banned_user(&mut self, user_id: i32, reason: &str) -> Result<BannedUser> {
		Ok(sqlx::query_as!(
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
		.fetch_one(self.executor())
		.await
		.map_err(insert_err)?)
	}

	async fn get_banned_user(&mut self, id: i32) -> Result<Option<BannedUser>> {
		Ok(sqlx::query_as!(
			BannedUser,
			"SELECT
				id, reason, date_banned
			FROM
				Banned_users
			WHERE id = $1
			LIMIT 1;",
			id
		)
		.fetch_optional(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_all_banned_users(&mut self) -> Result<Vec<BannedUser>> {
		Ok(sqlx::query_as!(
			BannedUser,
			"SELECT id, reason, date_banned FROM Banned_users;",
		)
		.fetch_all(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_banned_users_count(&mut self) -> Result<i64> {
		Ok(sqlx::query!("SELECT count(*) FROM Banned_users;")
			.fetch_one(self.executor())
			.await
			.map_err(select_err)?
			.count
			.ok_or_else(count_err)?)
	}

	async fn update_banned_user_reason(&mut self, id: i32, reason: &str) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Banned_users
			SET
				reason = $2
			WHERE id = $1;",
			id,
			reason
		)
		.execute(self.executor())
		.await
		.map_err(update_err)?
		.rows_affected())
	}

	async fn delete_banned_user(&mut self, id: i32) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Banned_users WHERE id = $1;", id)
			.execute(self.executor())
			.await
			.map_err(delete_err)?
			.rows_affected())
	}

	async fn delete_all_banned_users(&mut self) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Banned_users;")
			.execute(self.executor())
			.await
			.map_err(delete_err)?
			.rows_affected())
	}

	async fn insert_chapter_revision(
		&mut self, revision: ChapterEdit, created_by: i32, chapter_id: i32,
	) -> Result<ChapterRevision> {
		Ok(sqlx::query_as!(
			ChapterRevision,
			"INSERT INTO Chapter_revisions
				(title, intro_text, outro_text, created_by, chapter_id)
			VALUES
				($1, $2, $3, $4, $5)
			RETURNING
				id, title, intro_text, outro_text,
				created_by, chapter_id, date_created;",
			revision.title,
			revision.intro_text,
			revision.outro_text,
			created_by,
			chapter_id
		)
		.fetch_one(self.executor())
		.await
		.map_err(insert_err)?)
	}

	async fn get_chapter_revision(&mut self, id: i32) -> Result<Option<ChapterRevision>> {
		Ok(sqlx::query_as!(
			ChapterRevision,
			"SELECT
				id, title, intro_text, outro_text,
				created_by, chapter_id, date_created
			FROM Chapter_revisions
			WHERE id = $1
			LIMIT 1;",
			id
		)
		.fetch_optional(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_latest_chapter_revision_opt(
		&mut self, chapter_id: i32,
	) -> Result<Option<ChapterRevision>> {
		Ok(sqlx::query_as!(
			ChapterRevision,
			"SELECT
				id, title, intro_text, outro_text,
				created_by, chapter_id, date_created
			FROM Chapter_revisions
			WHERE chapter_id = $1
			ORDER BY date_created DESC
			LIMIT 1;",
			chapter_id
		)
		.fetch_optional(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_latest_chapter_revision(&mut self, chapter_id: i32) -> Result<ChapterRevision> {
		Ok(self
			.get_latest_chapter_revision_opt(chapter_id)
			.await?
			.ok_or_else(db_expect)?)
	}

	async fn get_oldest_chapter_revision_opt(
		&mut self, chapter_id: i32,
	) -> Result<Option<ChapterRevision>> {
		Ok(sqlx::query_as!(
			ChapterRevision,
			"SELECT
				id, title, intro_text, outro_text,
				created_by, chapter_id, date_created
			FROM Chapter_revisions
			WHERE chapter_id = $1
			ORDER BY date_created
			LIMIT 1;",
			chapter_id
		)
		.fetch_optional(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_oldest_chapter_revision(&mut self, chapter_id: i32) -> Result<ChapterRevision> {
		Ok(self
			.get_oldest_chapter_revision_opt(chapter_id)
			.await?
			.ok_or_else(db_expect)?)
	}

	async fn get_all_chapter_revisions_by_chapter(
		&mut self, chapter_id: i32,
	) -> Result<Vec<ChapterRevision>> {
		Ok(sqlx::query_as!(
			ChapterRevision,
			"SELECT
				id, title, intro_text, outro_text,
				created_by, chapter_id, date_created
			FROM Chapter_revisions
			WHERE chapter_id = $1
			ORDER BY date_created DESC;",
			chapter_id
		)
		.fetch_all(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_all_chapter_revisions(&mut self) -> Result<Vec<ChapterRevision>> {
		Ok(sqlx::query_as!(
			ChapterRevision,
			"SELECT
				id, title, intro_text, outro_text,
				created_by, chapter_id, date_created
			FROM Chapter_revisions;",
		)
		.fetch_all(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_chapter_revisions_count_by_id(&mut self, id: i32) -> Result<i64> {
		Ok(sqlx::query!(
			"SELECT count(*) FROM Chapter_revisions WHERE chapter_id = $1;",
			id
		)
		.fetch_one(self.executor())
		.await
		.map_err(select_err)?
		.count
		.ok_or_else(count_err)?)
	}

	async fn get_chapter_revisions_count(&mut self) -> Result<i64> {
		Ok(sqlx::query!("SELECT count(*) FROM Chapter_revisions;")
			.fetch_one(self.executor())
			.await
			.map_err(select_err)?
			.count
			.ok_or_else(count_err)?)
	}

	async fn delete_chapter_revision(&mut self, id: i32) -> Result<u64> {
		Ok(
			sqlx::query!("DELETE FROM Chapter_revisions WHERE id = $1;", id)
				.execute(self.executor())
				.await
				.map_err(delete_err)?
				.rows_affected(),
		)
	}

	async fn delete_all_chapter_revisions(&mut self) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Chapter_revisions;")
			.execute(self.executor())
			.await
			.map_err(delete_err)?
			.rows_affected())
	}

	async fn create_chapter(&mut self) -> Result<Chapter> {
		Ok(sqlx::query_as!(
			Chapter,
			"INSERT INTO Chapters DEFAULT VALUES
			RETURNING
				id, vote_duration, minutes_left, fimfic_ch_id, chapter_order, last_edit;",
		)
		.fetch_one(self.executor())
		.await
		.map_err(insert_err)?)
	}

	async fn get_chapter(&mut self, id: i32) -> Result<Option<Chapter>> {
		Ok(sqlx::query_as!(
			Chapter,
			"SELECT
				id, vote_duration, minutes_left, fimfic_ch_id, chapter_order, last_edit
			FROM Chapters WHERE id = $1 LIMIT 1;",
			id,
		)
		.fetch_optional(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_chapter_exists(&mut self, id: i32) -> Result<bool> {
		Ok(
			sqlx::query!("SELECT EXISTS(SELECT 1 FROM Chapters WHERE id = $1);", id)
				.fetch_one(self.executor())
				.await
				.map_err(select_err)?
				.exists
				.ok_or_else(exist_err)?,
		)
	}

	async fn get_chapter_by_order(&mut self, order: i32) -> Result<Option<Chapter>> {
		Ok(sqlx::query_as!(
			Chapter,
			"SELECT
				id, vote_duration, minutes_left, fimfic_ch_id, chapter_order, last_edit
			FROM Chapters WHERE chapter_order = $1 LIMIT 1;",
			order,
		)
		.fetch_optional(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_chapters_after_order(&mut self, order: i32) -> Result<Vec<Chapter>> {
		Ok(sqlx::query_as!(
			Chapter,
			"SELECT
				id, vote_duration, minutes_left, fimfic_ch_id, chapter_order, last_edit
			FROM Chapters
			WHERE chapter_order > $1
			ORDER BY chapter_order;",
			order,
		)
		.fetch_all(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_all_chapters(&mut self) -> Result<Vec<Chapter>> {
		Ok(sqlx::query_as!(
			Chapter,
			"SELECT
				id, vote_duration, minutes_left, fimfic_ch_id, chapter_order, last_edit
			FROM Chapters
			ORDER BY chapter_order NULLS LAST, id;",
		)
		.fetch_all(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_chapters_count(&mut self) -> Result<i64> {
		Ok(sqlx::query!("SELECT count(*) FROM Chapters;")
			.fetch_one(self.executor())
			.await
			.map_err(select_err)?
			.count
			.ok_or_else(count_err)?)
	}

	async fn update_chapter_vote_duration(&mut self, id: i32, duration: i32) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Chapters
			SET
				vote_duration = $2,
				last_edit = now()
			WHERE id = $1;",
			id,
			duration
		)
		.execute(self.executor())
		.await
		.map_err(update_err)?
		.rows_affected())
	}

	async fn update_chapter_minutes_left(&mut self, id: i32, minutes: i32) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Chapters
			SET
				minutes_left = $2,
				last_edit = now()
			WHERE id = $1;",
			id,
			minutes
		)
		.execute(self.executor())
		.await
		.map_err(update_err)?
		.rows_affected())
	}

	async fn update_chapter_fimfic_id(&mut self, id: i32, fimfic_id: i32) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Chapters
			SET
				fimfic_ch_id = $2,
				last_edit = now()
			WHERE id = $1;",
			id,
			fimfic_id
		)
		.execute(self.executor())
		.await
		.map_err(update_err)?
		.rows_affected())
	}

	async fn update_chapter_order(&mut self, id: i32, order: i32) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Chapters
			SET
				chapter_order = $2,
				last_edit = now()
			WHERE id = $1;",
			id,
			order
		)
		.execute(self.executor())
		.await
		.map_err(update_err)?
		.rows_affected())
	}

	async fn update_chapter_order_none(&mut self, id: i32) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Chapters
			SET
				chapter_order = NULL,
				last_edit = now()
			WHERE id = $1;",
			id,
		)
		.execute(self.executor())
		.await
		.map_err(update_err)?
		.rows_affected())
	}

	async fn delete_chapter(&mut self, id: i32) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Chapters WHERE id = $1;", id)
			.execute(self.executor())
			.await
			.map_err(delete_err)?
			.rows_affected())
	}

	async fn delete_all_chapters(&mut self) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Chapters;")
			.execute(self.executor())
			.await
			.map_err(delete_err)?
			.rows_affected())
	}

	async fn insert_question_revision(
		&mut self, edit: QuestionEdit, question_id: i32, creator_id: i32,
	) -> Result<QuestionRevision> {
		Ok(sqlx::query_as!(
			QuestionRevision,
			r#"INSERT INTO Question_revisions
				(question_text, type, asked_by, response_percent,
				option_writing, result_writing, question_id, created_by)
			VALUES
				($1, $2, $3, $4, $5, $6, $7, $8)
			RETURNING
				id, question_text, type AS "question_type: QuestionType", asked_by, response_percent,
				option_writing, result_writing, question_id, created_by, date_created;"#,
			edit.question_text,
			edit.question_type as _,
			edit.asked_by,
			edit.response_percent,
			edit.option_writing,
			edit.result_writing,
			question_id,
			creator_id,
		)
		.fetch_one(self.executor())
		.await
		.map_err(insert_err)?)
	}

	async fn get_question_revision(&mut self, id: i32) -> Result<Option<QuestionRevision>> {
		Ok(sqlx::query_as!(
			QuestionRevision,
			r#"SELECT
				id, question_text, type AS "question_type: QuestionType", asked_by, response_percent,
				option_writing, result_writing, question_id, created_by, date_created
			FROM Question_revisions WHERE id = $1 LIMIT 1;"#,
			id,
		)
		.fetch_optional(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_latest_question_revision_opt(
		&mut self, question_id: i32,
	) -> Result<Option<QuestionRevision>> {
		Ok(sqlx::query_as!(
			QuestionRevision,
			r#"SELECT
				id, question_text, type AS "question_type: QuestionType", asked_by, response_percent,
				option_writing, result_writing, question_id, created_by, date_created
			FROM question_revisions
			WHERE question_id = $1
			ORDER BY date_created DESC
			LIMIT 1;"#,
			question_id
		)
		.fetch_optional(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_latest_question_revision(&mut self, question_id: i32) -> Result<QuestionRevision> {
		Ok(self
			.get_latest_question_revision_opt(question_id)
			.await?
			.ok_or_else(db_expect)?)
	}

	async fn get_oldest_question_revision_opt(
		&mut self, question_id: i32,
	) -> Result<Option<QuestionRevision>> {
		Ok(sqlx::query_as!(
			QuestionRevision,
			r#"SELECT
				id, question_text, type AS "question_type: QuestionType", asked_by, response_percent,
				option_writing, result_writing, question_id, created_by, date_created
			FROM question_revisions
			WHERE question_id = $1
			ORDER BY date_created
			LIMIT 1;"#,
			question_id
		)
		.fetch_optional(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_oldest_question_revision(&mut self, question_id: i32) -> Result<QuestionRevision> {
		Ok(self
			.get_oldest_question_revision_opt(question_id)
			.await?
			.ok_or_else(db_expect)?)
	}

	async fn get_all_question_revisions_by_question(
		&mut self, question_id: i32,
	) -> Result<Vec<QuestionRevision>> {
		Ok(sqlx::query_as!(
			QuestionRevision,
			r#"SELECT
				id, question_text, type AS "question_type: QuestionType", asked_by, response_percent,
				option_writing, result_writing, question_id, created_by, date_created
			FROM Question_revisions
			WHERE question_id = $1
			ORDER BY date_created DESC;"#,
			question_id
		)
		.fetch_all(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_all_question_revisions(&mut self) -> Result<Vec<QuestionRevision>> {
		Ok(sqlx::query_as!(
			QuestionRevision,
			r#"SELECT
				id, question_text, type AS "question_type: QuestionType", asked_by, response_percent,
				option_writing, result_writing, question_id, created_by, date_created
			FROM Question_revisions;"#,
		)
		.fetch_all(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_question_revision_count(&mut self, question_id: i32) -> Result<i64> {
		Ok(sqlx::query!(
			"SELECT count(*) FROM Question_revisions WHERE question_id = $1;",
			question_id
		)
		.fetch_one(self.executor())
		.await
		.map_err(select_err)?
		.count
		.ok_or_else(count_err)?)
	}

	async fn get_question_revisions_count(&mut self) -> Result<i64> {
		Ok(sqlx::query!("SELECT count(*) FROM Question_revisions;")
			.fetch_one(self.executor())
			.await
			.map_err(select_err)?
			.count
			.ok_or_else(count_err)?)
	}

	async fn delete_question_writing(&mut self, id: i32) -> Result<u64> {
		Ok(
			sqlx::query!("DELETE FROM Question_revisions WHERE id = $1;", id)
				.execute(self.executor())
				.await
				.map_err(delete_err)?
				.rows_affected(),
		)
	}

	async fn delete_all_question_revisions(&mut self) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Question_revisions;")
			.execute(self.executor())
			.await
			.map_err(delete_err)?
			.rows_affected())
	}

	async fn create_question(&mut self) -> Result<Question> {
		Ok(sqlx::query_as!(
			Question,
			"INSERT INTO Questions DEFAULT VALUES
			RETURNING
				id, claimed_by, chapter_id, chapter_order, last_edit;"
		)
		.fetch_one(self.executor())
		.await
		.map_err(insert_err)?)
	}

	async fn get_question(&mut self, id: i32) -> Result<Option<Question>> {
		Ok(sqlx::query_as!(
			Question,
			"SELECT
				id, claimed_by, chapter_id, chapter_order, last_edit
			FROM Questions WHERE id = $1 LIMIT 1;",
			id,
		)
		.fetch_optional(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_question_by_chapter_and_order(
		&mut self, chapter_id: i32, order: i32,
	) -> Result<Option<Question>> {
		Ok(sqlx::query_as!(
			Question,
			"SELECT
				id, claimed_by, chapter_id, chapter_order, last_edit
			FROM Questions
			WHERE
				chapter_id = $1 AND chapter_order = $2
			LIMIT 1;",
			chapter_id,
			order
		)
		.fetch_optional(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_questions_by_chapter(&mut self, chapter_id: i32) -> Result<Vec<Question>> {
		Ok(sqlx::query_as!(
			Question,
			"SELECT
				id, claimed_by, chapter_id, chapter_order, last_edit
			FROM Questions WHERE chapter_id = $1;",
			chapter_id
		)
		.fetch_all(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_questions_by_claiment(
		&mut self, claiment_id: Option<i32>,
	) -> Result<Vec<Question>> {
		Ok(sqlx::query_as!(
			Question,
			"SELECT
				id, claimed_by, chapter_id, chapter_order, last_edit
			FROM Questions WHERE claimed_by = $1;",
			claiment_id
		)
		.fetch_all(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_questions_for_table(&mut self, chapter_id: i32) -> Result<Vec<Question>> {
		Ok(sqlx::query_as!(
			Question,
			"SELECT
				id, claimed_by, chapter_id, chapter_order, last_edit
			FROM Questions
			WHERE chapter_id = $1 OR chapter_id IS NULL
			ORDER BY chapter_order NULLS LAST, id;",
			chapter_id
		)
		.fetch_all(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_all_questions(&mut self) -> Result<Vec<Question>> {
		Ok(sqlx::query_as!(
			Question,
			"SELECT
            q.id, q.claimed_by, q.chapter_id, q.chapter_order, q.last_edit
        FROM Questions AS q
        LEFT JOIN Chapters AS c ON q.chapter_id = c.id
        ORDER BY c.chapter_order NULLS LAST, q.chapter_order, q.id;"
		)
		.fetch_all(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_question_count_by_chapter(&mut self, chapter_id: i32) -> Result<i64> {
		Ok(sqlx::query!(
			"SELECT count(*) FROM Questions WHERE chapter_id = $1;",
			chapter_id
		)
		.fetch_one(self.executor())
		.await
		.map_err(select_err)?
		.count
		.ok_or_else(count_err)?)
	}

	async fn get_questions_count(&mut self) -> Result<i64> {
		Ok(sqlx::query!("SELECT count(*) FROM Questions;")
			.fetch_one(self.executor())
			.await
			.map_err(select_err)?
			.count
			.ok_or_else(count_err)?)
	}

	async fn update_question_claimed_by(
		&mut self, id: i32, claimed_by: Option<i32>,
	) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Questions
			SET
				claimed_by = $2,
				last_edit = now()
			WHERE id = $1;",
			id,
			claimed_by
		)
		.execute(self.executor())
		.await
		.map_err(update_err)?
		.rows_affected())
	}

	async fn update_question_chapter_id(
		&mut self, id: i32, chapter_id: Option<i32>,
	) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Questions
			SET
				chapter_id = $2,
				last_edit = now()
			WHERE id = $1;",
			id,
			chapter_id
		)
		.execute(self.executor())
		.await
		.map_err(update_err)?
		.rows_affected())
	}

	async fn update_question_chapter_order(&mut self, id: i32, chapter_order: i32) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Questions
			SET
				chapter_order = $2,
				last_edit = now()
			WHERE id = $1;",
			id,
			chapter_order
		)
		.execute(self.executor())
		.await
		.map_err(update_err)?
		.rows_affected())
	}

	async fn update_question_chapter_order_none(&mut self, id: i32) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Questions
			SET
				chapter_order = NULL,
				last_edit = now()
			WHERE id = $1;",
			id,
		)
		.execute(self.executor())
		.await
		.map_err(update_err)?
		.rows_affected())
	}

	async fn update_question_chapter_id_order(
		&mut self, id: i32, chapter_id: i32, chapter_order: i32,
	) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Questions
			SET
				chapter_id = $2,
				chapter_order = $3,
				last_edit = now()
			WHERE id = $1;",
			id,
			chapter_id,
			chapter_order
		)
		.execute(self.executor())
		.await
		.map_err(update_err)?
		.rows_affected())
	}

	async fn update_question_chapter_id_order_none(&mut self, id: i32) -> Result<u64> {
		Ok(sqlx::query!(
			"UPDATE Questions
			SET
				chapter_id = NULL,
				chapter_order = NULL,
				last_edit = now()
			WHERE id = $1;",
			id,
		)
		.execute(self.executor())
		.await
		.map_err(update_err)?
		.rows_affected())
	}

	async fn delete_question(&mut self, id: i32) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Questions WHERE id = $1;", id)
			.execute(self.executor())
			.await
			.map_err(delete_err)?
			.rows_affected())
	}

	async fn delete_all_questions(&mut self) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Questions;")
			.execute(self.executor())
			.await
			.map_err(delete_err)?
			.rows_affected())
	}

	async fn insert_vote(
		&mut self, user_id: i32, question_id: i32, option_id: i32,
	) -> Result<Vote> {
		Ok(sqlx::query_as!(
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
		.fetch_one(self.executor())
		.await
		.map_err(insert_err)?)
	}

	async fn get_all_votes_by_user(&mut self, user_id: i32) -> Result<Vec<Vote>> {
		Ok(sqlx::query_as!(
			Vote,
			"SELECT
				voter_id, question_id, option_id, date_created
			FROM Votes
			WHERE voter_id = $1;",
			user_id
		)
		.fetch_all(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_all_votes_by_question(&mut self, question_id: i32) -> Result<Vec<Vote>> {
		Ok(sqlx::query_as!(
			Vote,
			"SELECT
				voter_id, question_id, option_id, date_created
			FROM Votes
			WHERE question_id = $1;",
			question_id
		)
		.fetch_all(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_all_votes_by_option(&mut self, option_id: i32) -> Result<Vec<Vote>> {
		Ok(sqlx::query_as!(
			Vote,
			"SELECT
				voter_id, question_id, option_id, date_created
			FROM Votes
			WHERE option_id = $1;",
			option_id
		)
		.fetch_all(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_all_votes(&mut self) -> Result<Vec<Vote>> {
		Ok(sqlx::query_as!(
			Vote,
			"SELECT voter_id, question_id, option_id, date_created FROM Votes;",
		)
		.fetch_all(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_votes_count(&mut self) -> Result<i64> {
		Ok(sqlx::query!("SELECT count(*) FROM Votes;")
			.fetch_one(self.executor())
			.await
			.map_err(select_err)?
			.count
			.ok_or_else(count_err)?)
	}

	async fn delete_votes_by_user(&mut self, user_id: i32) -> Result<u64> {
		Ok(
			sqlx::query!("DELETE FROM Votes WHERE voter_id = $1;", user_id)
				.execute(self.executor())
				.await
				.map_err(delete_err)?
				.rows_affected(),
		)
	}

	async fn delete_votes_by_option(&mut self, question_id: i32) -> Result<u64> {
		Ok(
			sqlx::query!("DELETE FROM Votes WHERE question_id = $1;", question_id)
				.execute(self.executor())
				.await
				.map_err(delete_err)?
				.rows_affected(),
		)
	}

	async fn delete_votes_by_question(&mut self, option_id: i32) -> Result<u64> {
		Ok(
			sqlx::query!("DELETE FROM Votes WHERE option_id = $1;", option_id)
				.execute(self.executor())
				.await
				.map_err(delete_err)?
				.rows_affected(),
		)
	}

	async fn delete_all_votes(&mut self) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Votes;")
			.execute(self.executor())
			.await
			.map_err(delete_err)?
			.rows_affected())
	}

	async fn insert_story_update(&mut self, data: StoryData<i32>) -> Result<StoryUpdate> {
		Ok(sqlx::query_as!(
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
		.fetch_one(self.executor())
		.await
		.map_err(insert_err)?)
	}

	async fn get_story_updates_in_range(
		&mut self, start: DateTime<Utc>, end: DateTime<Utc>,
	) -> Result<Vec<StoryUpdate>> {
		Ok(sqlx::query_as!(
			StoryUpdate,
			"SELECT
				title, short_description, description, views, total_views,
				words, chapters, comments, rating, likes, dislikes, date_cached
			FROM Story_updates
			WHERE date_cached > $1 AND date_cached < $2;",
			start,
			end
		)
		.fetch_all(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_all_story_updates(&mut self) -> Result<Vec<StoryUpdate>> {
		Ok(sqlx::query_as!(
			StoryUpdate,
			"SELECT
				title, short_description, description, views, total_views,
				words, chapters, comments, rating, likes, dislikes, date_cached
			FROM Story_updates;",
		)
		.fetch_all(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_story_updates_count(&mut self) -> Result<i64> {
		Ok(sqlx::query!("SELECT count(*) FROM Story_updates;")
			.fetch_one(self.executor())
			.await
			.map_err(select_err)?
			.count
			.ok_or_else(count_err)?)
	}

	async fn delete_story_update(&mut self, date_cached: DateTime<Utc>) -> Result<u64> {
		Ok(sqlx::query!(
			"DELETE FROM Story_updates WHERE date_cached = $1;",
			date_cached
		)
		.execute(self.executor())
		.await
		.map_err(delete_err)?
		.rows_affected())
	}

	async fn delete_story_updates_in_range(
		&mut self, start: DateTime<Utc>, end: DateTime<Utc>,
	) -> Result<u64> {
		Ok(sqlx::query!(
			"DELETE FROM Story_updates
			WHERE date_cached > $1 AND date_cached > $2;",
			start,
			end
		)
		.execute(self.executor())
		.await
		.map_err(delete_err)?
		.rows_affected())
	}

	async fn delete_all_story_updates(&mut self) -> Result<u64> {
		Ok(sqlx::query!("DELETE FROM Story_updates;")
			.execute(self.executor())
			.await
			.map_err(delete_err)?
			.rows_affected())
	}
}
