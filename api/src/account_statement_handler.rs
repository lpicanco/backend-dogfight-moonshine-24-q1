use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use moonshine_db::Pool;

use crate::db::get_account_statement;

pub async fn handle(
    State(pool): State<Pool>,
    Path(client_id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {

    if client_id > 5 {
        return Err(StatusCode::NOT_FOUND);
    }

    return Ok(get_account_statement(client_id, pool).await);
}
