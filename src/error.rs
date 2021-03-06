use crate::response::{ApiFailure, Failure};
use poem::{http::StatusCode, web::Json};
use serde_json::Error as SerdeJsonError;
use serde_json::{json, Value};
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    BcryptError(#[from] bcrypt::BcryptError),
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
    #[error(transparent)]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error(transparent)]
    TokioRecvError(#[from] tokio::sync::oneshot::error::RecvError),
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),
    #[error(transparent)]
    SsiDidJwkGenerationError(#[from] ssi::error::Error),
    #[error(transparent)]
    CommandError(#[from] io::Error),
    #[error(transparent)]
    Web3Error(#[from] web3::Error),
    #[error(transparent)]
    EthAbiError(#[from] ethabi::Error),
    #[error(transparent)]
    SerdeJsonError(#[from] SerdeJsonError),
    #[error(transparent)]
    StringConvertUtf8Error(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    OpensslError(#[from] openssl::error::ErrorStack),
    #[error(transparent)]
    Base64DecodeError(#[from] base64::DecodeError),
    #[error(transparent)]
    Web3ContractError(#[from] web3::contract::Error),
    //#[error(transparent)]
    //PoemError(#[from] poem::Error),
    #[error("wrong credentials")]
    WrongCredentials,
    #[error("password doesn't match")]
    WrongPassword,
    #[error("email is already taken")]
    DuplicateUserEmail,
    #[error("name is already taken")]
    DuplicateUserName,
    #[error("name is already taken")]
    DuplicateVcTpltName,
    #[error("DID generate error")]
    DidGenerateError,
    #[error("VC Issuer running error")]
    VcIssuerRunningError,
    #[error("VC Issuer not running error")]
    VcIssuerNotRunning,
    #[error("VC Issue error")]
    VcIssueError,
    #[error("VC Verify parser json error")]
    VcVerifyParserJsonError,
    #[error("VC Verify error")]
    VcVerifyError,
    #[error("VC Revoked")]
    VcRevokedError,
    #[error("VC Revoke not issuer")]
    VcRevokeNotIssuer,
    #[error("VC Credential not found")]
    VcCredentialNotFound,
    #[error("VP Verify error")]
    VpVerifyError,
    #[error("Passbase identity read error")]
    PassbaseIdentityReadError,
    #[error("Passbase identity notify backend error")]
    PassbaseIdentityNotifyBackendError,
    #[error("Passbase identity update db error")]
    PassbaseIdentityUpdateError,
    #[error("Chain data convert error")]
    ChainDataConvertError,
}
pub type Result<T> = std::result::Result<T, Error>;

pub type ApiError = (StatusCode, Json<Value>);
pub type _ApiResult<T> = std::result::Result<T, ApiError>;

impl From<Error> for ApiError {
    fn from(err: Error) -> Self {
        let status = match err {
            Error::WrongCredentials => StatusCode::UNAUTHORIZED,
            Error::ValidationError(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        //let payload = json!({"message": err.to_string()});
        let payload = json!(ApiFailure {
            api_version: Default::default(),
            body: Failure {
                code: status.as_u16(),
                message: err.to_string()
            }
        });
        (status, Json(payload))
    }
}
