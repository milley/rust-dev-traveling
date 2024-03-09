use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Pool, Postgres};

use crate::{internal_error, DatabaseConnection};

pub fn conn_routes() -> Router<Pool<Postgres>> {
    Router::new().route(
        "/",
        get(using_connection_pool_extractor).post(using_connection_extractor),
    )
}

pub fn todo_routes() -> Router<Pool<Postgres>> {
    Router::new().route("/todos", get(todos_index).post(todos_create))
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

#[derive(Debug, Serialize, Clone)]
pub struct Todo {
    id: i32,
    title: String,
    completed: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct Pagination {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

async fn todos_index(
    pagination: Option<Query<Pagination>>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    //let records = sqlx::query_as::<_, Todo>(
    let records = sqlx::query_as!(
        Todo,
        r#"
        SELECT id, title, completed
        FROM todos
        ORDER BY id
        "#,
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    let Query(pagination) = pagination.unwrap_or_default();

    let todos = records
        .iter()
        .skip(pagination.offset.unwrap_or(0))
        .take(pagination.limit.unwrap_or(usize::MAX))
        .cloned()
        .collect::<Vec<_>>();

    Json(todos)
}

#[derive(Debug, Deserialize)]
pub struct CreateTodo {
    pub title: String,
}

async fn todos_create(
    State(pool): State<PgPool>,
    Json(input): Json<CreateTodo>,
) -> impl IntoResponse {
    let record = sqlx::query!(
        r#"
        INSERT INTO todos (title)
        VALUES ($1)
        RETURNING id
        "#,
        input.title
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    (StatusCode::CREATED, Json(record.id))
}
