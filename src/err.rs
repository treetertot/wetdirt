use axum::response::IntoResponse;
use reqwest::StatusCode;
use serde_json::Value;
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

// Maybe have multiple error types so general errors are propagated but function specific ones can be handled explicitly?
#[derive(Error, Debug)]
pub enum Error {
    #[error("External request failed")]
    Connection(#[from] reqwest::Error),

    // this error can only be created by one function which prevents the program from starting
    #[error("failed to login to database. response: {0:?}")]
    DatabaseLogin(Value),

    #[error("bad status: {0:?}")]
    BadHttpStatus(StatusCode),

    #[error("entity not found")]
    NoSuchEntity,

    /// indicates either a badly written request or a badly configured database. Used often, found rarely.
    #[error("bad query: {0:?}")]
    BadQuery(Value),

    #[error("missing data. expected field: {0}")]
    MissingData(String),

    /// Error created when attempting to create an object with an existing id.
    #[error("ID {0:?} already exists")]
    IDExists(String),

    #[error("failed to hash password")]
    HashFailure,

    #[error("bad input string {0:?} potential sql injection attempted")]
    BadString(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        format!("{:?}", self).into_response()
    }
}
