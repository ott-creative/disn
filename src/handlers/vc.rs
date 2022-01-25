use crate::config::env::API_VERSION;
use crate::response::{ApiSuccess, Success};
use axum::{extract::Extension, http::StatusCode, Json};
use sqlx::PgPool;

use crate::{
    dto::VcTpltInput,
    error::ApiResult,
    model::{User, VcTplt},
    service::vc::VcTpltService,
    utils::validate_payload,
};

pub async fn vc_tplt_create(
    _user: User,
    Json(input): Json<VcTpltInput>,
    Extension(pool): Extension<PgPool>,
) -> ApiResult<(StatusCode, Json<ApiSuccess<VcTplt>>)> {
    // TODO: check user role, only admin can create vc tplt
    validate_payload(&input)?;
    let vc_tplt = VcTpltService::create(input, &pool).await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiSuccess {
            api_version: API_VERSION.to_string(),
            body: Success { data: vc_tplt },
        }),
    ))
}

//pub async fn vc_issue(user: User,)
