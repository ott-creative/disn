//pub mod passbase;
use poem::{handler, http::StatusCode};
use uuid::Uuid;

#[tracing:: instrument(
    name = "health check",
    fields(request_id = %Uuid::new_v4())
)]
#[handler]
pub fn health_check() -> StatusCode {
    StatusCode::OK
}
