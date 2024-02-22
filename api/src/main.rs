use std::env;

use axum::Router;
use axum::routing::{get, post};
use deadpool::Runtime;

use moonshine_db::client::Manager;
use moonshine_db::Pool;

mod account_statement_handler;
mod db;
mod model;
mod transaction_handler;


#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let moonshine_url = env::var("DATABASE_URL").unwrap_or("localhost:9942".to_string());
    let manager = Manager::new(moonshine_url);

    let pool = Pool::builder(manager)
        .max_size(15)
        .runtime(Runtime::Tokio1)
        .build()
        .unwrap();

    let port = env::var("PORT").unwrap_or("9999".to_string());
    db::init(&mut pool.get().await.unwrap()).await;

    let app = Router::new()
        .route("/clientes/:client_id/extrato", get(account_statement_handler::handle))
        .route("/clientes/:client_id/transacoes", post(transaction_handler::handle))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await.unwrap();
    println!("⚗️ 🥂moonshine-api running at http://localhost:{}/", port);

    axum::serve(listener, app.into_make_service())
        .await.unwrap();
}
