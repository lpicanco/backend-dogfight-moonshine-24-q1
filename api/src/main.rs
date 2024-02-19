use std::env;
use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use deadpool::managed::Object;
use deadpool::Runtime;
use validator::Validate;

use moonshine_db::{Client, Pool};
use moonshine_db::client::Manager;
// use moonshine_db::client::{Manager};

mod account_statement_handler;
mod db;
mod model;
mod transaction_handler;


#[derive(Clone)]
struct AppState {
    client: Arc<Client>,
}


#[tokio::main]
async fn main() {
    // let database_url = env::var("DATABASE_URL").unwrap();
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let moonshine_url = env::var("DATABASE_URL").unwrap_or("localhost:9942".to_string());
    // let mut cfg = Config::new(moonshine_url);
    // let manager = Manager::from_config(&cfg, Runtime::Tokio1);
    // let pool = Pool::builder(manager)
    //     .config(cfg.get_pool_config())
    //     .build().unwrap();

    // let mgr = Manager {};
    // let pool = Pool::builder(mgr).build().unwrap();
    // let mut conn = pool.get().await.unwrap();
    // let answer = conn .get_answer().await;
    // assert_eq!(answer, 42);

    let manager = Manager::new(moonshine_url);
    // let pool = Pool::builder(manager).build().unwrap();

    // let manager = Manager::new(moonshine_url);
    let pool = Pool::builder(manager)
        .max_size(16)
        .runtime(Runtime::Tokio1)
        .build()
        .unwrap();

    let mut client: Object<Manager> = pool.get().await.unwrap();

    let response = client.echo("Hello, world!".as_bytes().to_vec()).await.unwrap();
    println!("{:?}", String::from_utf8(response).unwrap());

    client.open_partition("smoke").await.unwrap();
    client.put("smoke", "my-key", "Hello, world!".as_bytes().to_vec()).await.unwrap();

    let response = client.get("smoke", "my-key").await.unwrap();
    println!("{:?}", String::from_utf8(response).unwrap());


    let port = env::var("PORT").unwrap_or("9999".to_string());
    db::init(&mut pool.get().await.unwrap()).await;

    // let state_guard = Arc::new(moonshine);
    let app = Router::new()
        .route("/clientes/:client_id/extrato", get(account_statement_handler::handle))
        .route("/clientes/:client_id/transacoes", post(transaction_handler::handle))
        .with_state(pool);
;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await.unwrap();

    axum::serve(listener, app.into_make_service())
        .await.unwrap();
}
