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

#[derive(Debug, Serialize, Deserialize)]
pub struct Success<T> {
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Failure {
    pub code: u16,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSuccess<T> {
    pub api_version: String,

    #[serde(flatten)]
    pub body: Success<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiFailure {
    pub api_version: String,

    #[serde(flatten)]
    pub body: Failure,
}
