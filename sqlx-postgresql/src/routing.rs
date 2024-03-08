use axum::{extract::State, http::StatusCode, routing::get, Router};
use sqlx::{PgPool, Pool, Postgres};

use crate::{internal_error, DatabaseConnection};

pub fn conn_routes() -> Router<Pool<Postgres>> {
    Router::new().route(
        "/",
        get(using_connection_pool_extractor).post(using_connection_extractor),
    )
}

async fn using_connection_pool_extractor(
    State(pool): State<PgPool>,
) -> Result<String, (StatusCode, String)> {
    sqlx::query_scalar("select 'hello world from pg'")
        .fetch_one(&pool)
        .await
        .map_err(internal_error)
}

async fn using_connection_extractor(
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<String, (StatusCode, String)> {
    sqlx::query_scalar("select 'hello world from pg'")
        .fetch_one(&mut *conn)
        .await
        .map_err(internal_error)
}
