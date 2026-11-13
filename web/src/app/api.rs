use topcoat::{Result, context::Cx, router::route};

#[route(GET "./health")]
async fn health(cx: &Cx) -> Result<&'static str> {
    Ok("ok")
}
