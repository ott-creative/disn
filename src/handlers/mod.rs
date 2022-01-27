pub mod did;
pub mod user;
pub mod vc;

use axum::http::StatusCode;
use uuid::Uuid;

#[tracing:: instrument(
    name = "health check",
    fields(request_id = %Uuid::new_v4())
)]
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}
