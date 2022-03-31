use poem::{handler, http::StatusCode, web::Data, web::Json};
use sqlx::PgPool;
//use uuid::Uuid;

/*
{
  "event": "VERIFICATION_COMPLETED",
  "key": "b76e244e-26a3-49ef-9c72-3e599bf0b5f2",
  "status": "pending",
  "created": 1582628711,
  "updated": 1582628999,
  "processed": 1582628999,
}

{
  "event": "VERIFICATION_REVIEWED",
  "key": "b76e244e-26a3-49ef-9c72-3e599bf0b5f2",
  "status": "approved",
  "created": 1582628711,
  "updated": 1582628999,
  "processed": 1582628999,
}

{
  "event": "DATAPOINT_UPDATED",
  "key": "ce31e763-e0a5-4ce7-9b43-1086541abf30",
  "resource_key": "6ff6c2bf-add7-4569-96ff-40ee86f946f1",
  "type": "SEX",
  "value": "male",
  "updated": 1619095419,
  "created": 1619095419
}
 */

/*#[derive(Enum)]
pub enum HookEvent {
    VERIFICATION_COMPLETED,
    VERIFICATION_REVIEWED,
    DATAPOINT_UPDATED,
}*/

#[derive(Deserialize, Debug)]
pub struct HookData {
    event: String,
    key: String,
    status: String,
    created: i64,
    updated: i64,
    processed: i64,
}

#[handler]
pub fn passbase_hook(_pool: Data<&PgPool>, data: Json<HookData>) -> StatusCode {
    tracing::info!("passbase webhook: {:?}", data);
    StatusCode::OK
}
