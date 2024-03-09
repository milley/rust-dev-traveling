use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
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
    Router::new()
        .route("/todos", get(todos_index).post(todos_create))
        .route("/todos/:id", patch(todos_update).delete(todos_delete))
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
struct Pagination {
    offset: Option<usize>,
    limit: Option<usize>,
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
struct CreateTodo {
    title: String,
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

#[derive(Debug, Deserialize)]
struct UpdateTodo {
    title: Option<String>,
    completed: Option<bool>,
}

async fn todos_update(
    Path(id): Path<i32>,
    State(pool): State<PgPool>,
    Json(input): Json<UpdateTodo>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut todo = sqlx::query_as!(
        Todo,
        r#"
        SELECT id, title, completed
        FROM todos
        WHERE id = $1
        "#,
        id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    if let Some(title) = input.title {
        todo.title = title;
    }

    if let Some(completed) = input.completed {
        todo.completed = completed;
    }

    let rows_affected = sqlx::query!(
        r#"
        UPDATE todos
        SET title = $2, completed = $3
        WHERE id = $1
        "#,
        todo.id,
        todo.title,
        todo.completed
    )
    .execute(&pool)
    .await
    .unwrap()
    .rows_affected();

    if rows_affected == 0 {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(todo))
}

async fn todos_delete(Path(id): Path<i32>, State(pool): State<PgPool>) -> impl IntoResponse {
    let rows_affected = sqlx::query!(
        r#"
            DELETE FROM todos WHERE id = $1
        "#,
        id
    )
    .execute(&pool)
    .await
    .unwrap()
    .rows_affected();

    if rows_affected == 0 {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::OK
    }
}
