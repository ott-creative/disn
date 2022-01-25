use chrono::Utc;
use sqlx::PgPool;

use crate::{
    dto::VcTpltInput,
    error::{Error, Result},
    model::{CreateVcTpltData, VcTplt},
};

pub struct VcTpltService;

impl VcTpltService {
    pub async fn create(input: VcTpltInput, pool: &PgPool) -> Result<VcTplt> {
        if VcTplt::find_by_name(&input.name, &pool).await.is_ok() {
            return Err(Error::DuplicateVcTpltName);
        }

        let data = CreateVcTpltData {
            name: input.name,
            purpose: input.purpose,
            fields: input.fields,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        Ok(VcTplt::create(data, &pool).await?)
    }

    // Make a VC template instance
    //pub async fn publish(tplt_id: Uuid, fields: String, pool: &PgPool) -> Result<VcTplt> {}
}
