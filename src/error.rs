use serde::{Deserialize, Serialize};

/// The combined error type for the client builder and for API errors
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq, Serialize)]
pub enum Error {
    #[error("Builder error: {0}")]
    Builder(BuilderError),

    #[error("Runtime error: {0}")]
    Runtime(RuntimeError),
}

impl Error {
    /// Shorthand for creating a runtime pagination error
    pub(crate) fn pag_item(s: String) -> Self {
        From::from(RuntimeError::PaginationItemDeserialization(s))
    }
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq, Deserialize, Serialize)]
pub enum BuilderError {
    #[error("Failed to build client from environment variables, {0} is missing.")]
    MissingEnv(String),

    #[error("Failed to build client from the provided environment file, could not find {0}")]
    InvalidEnvPath(String),

    #[error("Failed to build client, no username was provided")]
    MissingUsername,

    #[error("Failed to build client, no password was provided")]
    MissingPassword,

    #[error("Failed to build client, underlying client failed to build with: {0}")]
    ClientBuild(String),
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq, Deserialize, Serialize)]
pub enum RuntimeError {
    #[error("Failed to get valid response from server: {0}")]
    Response(String),

    #[error("Failed to deserialize the response: {0}")]
    Deserialization(String),

    #[error("Failed to serialize the post: {0}")]
    Serialization(String),

    #[error("Paginated item failed deserialized: {0}")]
    PaginationItemDeserialization(String),

    // We do not place the time::error::Error in the error since it includes std::io::Error which
    // does not implement Clone, PartialEq, or Eq
    #[error("Time parsing error: {0}")]
    TimeFormatError(String),

    #[error("Failed to deserialize the item into an Enum: {0}")]
    EnumDeserialization(String),

    #[error("Failed to parse item into valid URI: {0}")]
    InvalidUri(String),

    #[error("Failed to retrieve the HATEOAS URI: {0}")]
    MissingUri(&'static str),

    #[error("Failed to parse the final segment of the path as an ID.")]
    InvalidId,
}

impl From<RuntimeError> for Error {
    fn from(value: RuntimeError) -> Self {
        Error::Runtime(value)
    }
}

impl From<BuilderError> for Error {
    fn from(value: BuilderError) -> Self {
        Error::Builder(value)
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        println!("{:?}", value);
        Error::Runtime(RuntimeError::Response(value.to_string()))
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Runtime(RuntimeError::Deserialization(value.to_string()))
    }
}

impl From<time::error::Error> for Error {
    fn from(value: time::error::Error) -> Self {
        Error::Runtime(RuntimeError::TimeFormatError(value.to_string()))
    }
}

impl From<time::error::Format> for Error {
    fn from(value: time::error::Format) -> Self {
        Error::Runtime(RuntimeError::TimeFormatError(value.to_string()))
    }
}

impl From<url::ParseError> for Error {
    fn from(value: url::ParseError) -> Self {
        Self::Runtime(RuntimeError::InvalidUri(value.to_string()))
    }
}
