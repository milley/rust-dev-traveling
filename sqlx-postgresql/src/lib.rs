mod db;
mod routing;

use axum::http::StatusCode;
pub use db::DatabaseConnection;
pub use routing::conn_routes;

pub fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
