mod app;
mod components;
mod db;
mod features;
mod serial;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let db = db::connect().await?;
    db::migrate(&db).await?;

    let mut args = std::env::args().skip(1).peekable();
    if args.peek().is_some_and(|arg| arg == "toasty") {
        if let Err(error) = toasty_cli::ToastyCli::new(db).parse_from(args).await {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
        return Ok(());
    }

    if let Ok(port) = std::env::var("SERIAL_PORT") {
        tokio::spawn(serial::run(port, 115_200, db.clone()));
    } else {
        eprintln!("SERIAL_PORT not set, serial reader disabled");
    }

    topcoat::start(app::router().with(db).build()).await?;

    Ok(())
}
