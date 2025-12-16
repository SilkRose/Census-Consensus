use anyhow::Result;
use sqlx::{ Pool, Postgres };
use sqlx::postgres::PgPoolOptions;

#[derive(Clone)]
pub struct Db {
	pool: Pool<Postgres>
}

impl Db {
	pub async fn new(database_url: &str) -> Result<Self> {
		let pool = PgPoolOptions::new()
			.max_connections(16)
			.connect(database_url)
			.await?;

		sqlx::migrate!("./db-migrations")
			.run(&pool)
			.await?;

		Ok(Self { pool })
	}
}
