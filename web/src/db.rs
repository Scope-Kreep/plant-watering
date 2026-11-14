use toasty::{Db, embed_migrations};
use topcoat::context::{Cx, context};

const DATABASE_URL: &str = "sqlite:data.db";

pub async fn connect() -> toasty::Result<Db> {
    Db::builder()
        .models(toasty::models!(crate::*))
        .connect(DATABASE_URL)
        .await
}

static MIGRATIONS: toasty::migration::MigrationSet = embed_migrations!();

pub async fn migrate(db: &toasty::Db) -> toasty::Result<()> {
    let report = MIGRATIONS.apply(db).await?;

    println!("applied {} migrations", report.applied());
    Ok(())
}

#[allow(dead_code)]
pub fn db(cx: &Cx) -> Db {
    context::<Db>(cx).clone()
}
