use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use crate::{AppState, db};
use crate::model::Transaction;
use validator::Validate;
use moonshine_db::{Client, Pool};

pub async fn handle(
    State(pool): State<Pool>,
    Path(client_id): Path<i32>,
    Json(transaction): Json<Transaction>,
) -> Result<impl IntoResponse, StatusCode> {
    if let Err(errors) = transaction.validate() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    if transaction.valor.unwrap().ceil() != transaction.valor.unwrap() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    if client_id > 5 {
        return Err(StatusCode::NOT_FOUND);
    }

    let value = transaction.valor.unwrap() as i32;
    return Ok(db::create_transaction(transaction, client_id, value, pool).await);
}
