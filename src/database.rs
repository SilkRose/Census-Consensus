use crate::error::Result;
use crate::structs::*;
use crate::utility::{count_options, count_outcomes};
use pony::fimfiction_api::user::UserData;
use pony::smart_map::SmartMap;
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

	pub async fn get_chapters_table(&mut self) -> Result<Vec<ChapterTable>> {
		let mut tx = self.transaction().await?;
		let chapters = tx.get_all_chapters().await?;
		let mut data = vec![];
		for chapter in chapters {
			let oldest_data = tx.get_oldest_chapter_revision(chapter.id).await?;
			let newest_data = tx.get_latest_chapter_revision(chapter.id).await?;
			let oldest_user = tx.get_user(oldest_data.created_by).await?;
			let newest_user = tx.get_user(newest_data.created_by).await?;
			let revisions = tx.get_chapter_revisions_count_by_id(chapter.id).await?;
			let questions = tx.get_question_count_by_chapter(chapter.id).await?;
			let table_data = ChapterTable {
				meta: chapter,
				revisions,
				questions,
				oldest_data,
				newest_data,
				oldest_user,
				newest_user,
			};
			data.push(table_data);
		}
		tx.commit().await?;
		Ok(data)
	}

	pub async fn get_questions_table(
		&mut self, chapter_id: Option<i32>,
	) -> Result<(
		Vec<QuestionTable>,
		SmartMap<i32, (Chapter, ChapterRevision)>,
	)> {
		let mut tx = self.transaction().await?;
		let questions = match chapter_id {
			Some(chapter_id) => tx.get_questions_for_table(chapter_id).await?,
			None => tx.get_all_questions().await?,
		};
		let mut data = vec![];
		let mut chapters = SmartMap::default();
		for question in questions {
			let id = question.id;
			let claimant = match question.claimed_by {
				Some(id) => Some(tx.get_user(id).await?),
				None => None,
			};
			if chapter_id.is_none()
				&& let Some(chapter_id) = question.chapter_id
			{
				let chapter = tx.get_chapter(chapter_id).await?.expect("Always present.");
				let rev = tx.get_latest_chapter_revision(chapter_id).await?;
				chapters.insert(chapter_id, (chapter, rev));
			}
			let oldest_data = tx.get_oldest_question_revision(question.id).await?;
			let newest_data = tx.get_latest_question_revision(question.id).await?;
			let oldest_user = tx.get_user(oldest_data.created_by).await?;
			let newest_user = tx.get_user(newest_data.created_by).await?;
			let question_type = newest_data.question_type.clone();
			let table_data = QuestionTable {
				meta: question,
				revisions: tx.get_question_revision_count(id).await?,
				options: count_options(
					&newest_data.option_writing.clone().unwrap_or_default(),
					question_type,
				),
				outcomes: count_outcomes(&newest_data.result_writing.clone().unwrap_or_default()),
				claimant,
				oldest_data,
				newest_data,
				oldest_user,
				newest_user,
			};
			data.push(table_data);
		}
		tx.commit().await?;
		Ok((data, chapters))
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

	async fn get_all_contributors(&mut self) -> Result<Vec<User>> {
		Ok(sqlx::query_as!(
			User,
			r#"SELECT
				id, name, pfp_url, type AS "user_type: UserType", feedback_private,
				feedback_public, date_last_fetch, date_joined
			FROM Users
			WHERE type = $1 OR type = $2
			ORDER BY date_joined;"#,
			UserType::Writer as _,
			UserType::Admin as _
		)
		.fetch_all(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn get_all_users(&mut self) -> Result<Vec<User>> {
		Ok(sqlx::query_as!(
			User,
			r#"SELECT
				id, name, pfp_url, type AS "user_type: UserType", feedback_private,
				feedback_public, date_last_fetch, date_joined
			FROM Users
			ORDER BY date_joined;"#,
		)
		.fetch_all(self.executor())
		.await
		.map_err(select_err)?)
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
			ORDER BY date_created ASC
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

	async fn get_questions_by_chapter(&mut self, chapter_id: i32) -> Result<Vec<Question>> {
		Ok(sqlx::query_as!(
			Question,
			"SELECT
				id, claimed_by, chapter_id, chapter_order, last_edit
			FROM Questions
			WHERE chapter_id = $1
			ORDER BY chapter_order;",
			chapter_id
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
        ORDER BY c.chapter_order NULLS LAST, c.id, q.chapter_order, q.id;"
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

	async fn get_all_votes_by_question(&mut self, question_id: i32) -> Result<Vec<Vote>> {
		Ok(sqlx::query_as!(
			Vote,
			"SELECT
				voter_id, question_id, option_id, date_created
			FROM Votes
			WHERE question_id = $1
			ORDER BY option_id;",
			question_id
		)
		.fetch_all(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn insert_vote_complete(
		&mut self, user_id: i32, question_id: i32, option_id: &str,
	) -> Result<Vote> {
		Ok(sqlx::query_as!(
			Vote,
			"INSERT INTO Votes_complete
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

	async fn get_all_votes_complete_by_question(&mut self, question_id: i32) -> Result<Vec<Vote>> {
		Ok(sqlx::query_as!(
			Vote,
			"SELECT
				voter_id, question_id, option_id, date_created
			FROM Votes_complete
			WHERE question_id = $1
			ORDER BY option_id;",
			question_id
		)
		.fetch_all(self.executor())
		.await
		.map_err(select_err)?)
	}

	async fn delete_votes_complete_by_question_and_user(
		&mut self, question_id: i32, user_id: i32,
	) -> Result<u64> {
		Ok(sqlx::query!(
			"DELETE FROM Votes_complete
			WHERE
				question_id = $1
			AND
				voter_id = $2;",
			question_id,
			user_id
		)
		.execute(self.executor())
		.await
		.map_err(delete_err)?
		.rows_affected())
	}

	async fn get_logo_stats_census_count_by_user(&mut self, user_id: i32) -> Result<i64> {
		Ok(sqlx::query!(
			"SELECT count(*) FROM Logo_stats WHERE logo = $1 AND user_id = $2;",
			Logo::Census as _,
			user_id
		)
		.fetch_one(self.executor())
		.await
		.map_err(select_err)?
		.count
		.ok_or_else(count_err)?)
	}

	async fn get_logo_stats_consensus_count_by_user(&mut self, user_id: i32) -> Result<i64> {
		Ok(sqlx::query!(
			"SELECT count(*) FROM Logo_stats WHERE logo = $1 AND user_id = $2;",
			Logo::Consensus as _,
			user_id
		)
		.fetch_one(self.executor())
		.await
		.map_err(select_err)?
		.count
		.ok_or_else(count_err)?)
	}

	async fn get_settings(&mut self) -> Result<Settings> {
		Ok(sqlx::query_as!(
			Settings,
			"SELECT
				story_id, population, start_time
			FROM
				Settings
			LIMIT 1;"
		)
		.fetch_one(self.executor())
		.await
		.map_err(select_err)?)
	}
}
