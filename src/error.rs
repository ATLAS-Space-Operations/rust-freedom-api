use serde::Serialize;

/// Result type for the API
pub type Result<T> = std::result::Result<T, Error>;

/// The combined error type for the client builder and for API errors
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq, Serialize)]
pub enum Error {
    #[error("Failed to get valid response from server: {0}")]
    Response(String),

    #[error("Failed to deserialize the response: {0}")]
    Deserialization(String),

    #[error("Paginated item failed deserialized: {0}")]
    PaginationItemDeserialization(String),

    // We do not place the time::error::Error in the error since it includes std::io::Error which
    // does not implement Clone, PartialEq, or Eq
    #[error("Time parsing error: {0}")]
    TimeFormatError(String),

    #[error("Failed to parse item into valid URI: {0}")]
    InvalidUri(String),

    #[error("Failed to retrieve the HATEOAS URI: {0}")]
    MissingUri(&'static str),

    #[error("Failed to parse the final segment of the path as an ID.")]
    InvalidId,
}

impl Error {
    /// Shorthand for creating a runtime pagination error
    pub(crate) fn pag_item(s: String) -> Self {
        Self::PaginationItemDeserialization(s)
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::Response(value.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Deserialization(value.to_string())
    }
}

impl From<time::error::Error> for Error {
    fn from(value: time::error::Error) -> Self {
        Error::TimeFormatError(value.to_string())
    }
}

impl From<time::error::Format> for Error {
    fn from(value: time::error::Format) -> Self {
        Error::TimeFormatError(value.to_string())
    }
}

impl From<url::ParseError> for Error {
    fn from(value: url::ParseError) -> Self {
        Self::InvalidUri(value.to_string())
    }
}
