/*

Success:
Single Item:
{
    "apiVersion": "1.0",
    "data": {
    }
}

Pages:
{
    "apiVersion": "1.0",
    "data": {
        "items": [],
        "itemsPerPage": 10,
        "totalItems": 100,
        "totalPages": 10,
        "pageIndex": 1, // Start from 1
    }
}

Failure:
{
    "apiVersion": "2.0",
    "error": {
        "code": 404,
        "message": "File Not Found",
    }
}
*/
use crate::CONFIG;
use poem_openapi::Object;

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct ApiVersion {
    api_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Success<T> {
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct Failure {
    pub code: u16,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSuccess<T> {
    #[serde(flatten)]
    pub api_version: ApiVersion,

    #[serde(flatten)]
    pub body: Success<T>,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct ApiFailure {
    #[serde(flatten)]
    pub api_version: ApiVersion,

    #[serde(flatten)]
    pub body: Failure,
}

impl Default for ApiVersion {
    fn default() -> ApiVersion {
        ApiVersion {
            api_version: CONFIG.api_version.clone(),
        }
    }
}
