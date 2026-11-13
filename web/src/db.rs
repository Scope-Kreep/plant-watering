use toasty::Db;
use topcoat::context::{Cx, context};

const DATABASE_URL: &str = "sqlite:data.db";

pub async fn connect() -> toasty::Result<Db> {
    Db::builder()
        .models(toasty::models!(crate::*))
        .connect(DATABASE_URL)
        .await
}

#[allow(dead_code)]
pub fn db(cx: &Cx) -> Db {
    context::<Db>(cx).clone()
}
