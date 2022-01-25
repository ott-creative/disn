pub mod did;
pub mod user;
pub mod vc;

use axum::http::StatusCode;

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}
