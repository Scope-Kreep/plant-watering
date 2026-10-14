use topcoat::{Result, router::route};

#[route(GET "./health")]
async fn health() -> Result<&'static str> {
    Ok("ok")
}
