use std::error::Error;

use sqlx::{SqlitePool, migrate::Migrator, sqlite::SqliteConnectOptions};

mod app;

static MIGRATOR: Migrator = sqlx::migrate!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opts = SqliteConnectOptions::new()
        .filename("sqlite.db")
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(opts).await?;

    MIGRATOR.run(&pool).await?;

    topcoat::start(app::router().build()).await.unwrap();

    Ok(())
}
