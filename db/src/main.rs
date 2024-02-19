use std::env;
use std::error::Error;
use tokio::net::TcpListener;
use tokio::signal;
use moonshine_db::server;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    // env::set_var("RUST_LOG", "info");
    env::set_var("RUST_LOG", "warn");
    env_logger::init();

    let port = env::var("PORT").unwrap_or("9942".to_string());
    let db_path = env::var("DB_PATH").unwrap_or("moonshine.db".to_string());

    // Bind a TCP listener
    let listener = TcpListener::bind(&format!("0.0.0.0:{}", port)).await?;
    println!("⚗️ 💾moonshine-db running at http://localhost:{}/", port);

    server::run(listener, db_path, signal::ctrl_c()).await;

    Ok(())
}