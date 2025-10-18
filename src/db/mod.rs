use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;

pub async fn init_db() -> Result<PgPool> {
    let database_url = env::var("DATABASE_URL")
        .expect(" DATABASE_URL must be set in .env");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("Connected to Postgres successfully");
    Ok(pool)
}
