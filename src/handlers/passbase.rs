use crate::service::passbase::PassbaseService;
use poem::{handler, http::StatusCode, web::Data, web::Json};
use tokio::task;
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

#[derive(Serialize, Deserialize, Debug)]
pub struct HookData {
    pub event: String,
    pub key: String,
    pub status: String,
    pub created: i64,
    pub updated: i64,
    pub processed: i64,
}

#[handler]
pub fn passbase_hook(data: Json<HookData>) -> StatusCode {
    tracing::info!("passbase webhook: {:?}", data);
    // TODO: security check

    // schedule a job to process the hook
    task::spawn(async move {
        // refresh identity state
        let _ = PassbaseService::refresh_identity_status(&data.0.key).await;
    });

    tracing::info!("passbase webhook return");
    StatusCode::OK
}
