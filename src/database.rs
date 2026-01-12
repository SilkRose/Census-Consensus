use anyhow::Result;
use bon::bon;
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

		sqlx::migrate!("./migrations")
			.run(&pool)
			.await?;

		Ok(Self { pool })
	}

	/// Creates a new session for a user in the database, with the provided
	/// user id and token
	#[builder]
	pub async fn create_or_get_session(
		&self,
		username: &str,
		id: u32,
		user_type: UserType,
		pfp_link: &str,
		token: &str
	) -> Result<String> {
		let _ = (username, id, user_type, pfp_link, token);

		todo!()
	}
}

pub enum UserType {
	Admin,
	Writer,
	Voter
}
