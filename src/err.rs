use axum::response::IntoResponse;
use reqwest::StatusCode;
use serde_json::Value;
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

// Maybe have multiple error types so general errors are propagated but function specific ones can be handled explicitly?
#[derive(Error, Debug)]
pub enum Error {
    #[error("cannot connect to database properly")]
    Connection(#[from] reqwest::Error),

    #[error("failed to DatabaseLogin. response: {0:?}")]
    DatabaseLogin(Value),

    #[error("bad status: {0:?}")]
    BadHttpStatus(StatusCode),

    #[error("entity not found: {0:?}")]
    NoSuchEntity(String),

    #[error("bad query")]
    BadQuery(Value),

    #[error("missing data. expected field: {0}")]
    MissingData(String),

    #[error("malformed field. expected {field} to be {exp} rather than {real}")]
    MalformedData {
        field: String,
        exp: String,
        real: String,
    },

    #[error("ID {0:?} already exists")]
    IDExists(String),

    #[error("failed to hash password")]
    HashFailure,

    #[error("bad string {0:?} potential sql injection attempted")]
    BadString(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        format!("{:?}", self).into_response()
    }
}
