use std::{env, error::Error, str::FromStr};

use sqlx::{SqlitePool, migrate::Migrator, sqlite::SqliteConnectOptions};

static MIGRATOR: Migrator = sqlx::migrate!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:sqlite.db".to_string());

    let opts =
        SqliteConnectOptions::from_str(&database_url)?.create_if_missing(true);

    let pool = SqlitePool::connect_with(opts).await?;

    MIGRATOR.run(&pool).await?;

    Ok(())
}
