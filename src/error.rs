use std::fmt;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Http(#[from] http::Error),
    #[cfg(feature = "client")]
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("HTTP status: {}", _0)]
    HttpStatus(#[source] HttpStatusError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("other error: {}", _0)]
    Generic(#[from] anyhow::Error),
}

#[derive(Debug, thiserror::Error)]
pub struct HttpStatusError(pub http::StatusCode);

impl fmt::Display for HttpStatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<http::StatusCode> for Error {
    fn from(e: http::StatusCode) -> Self {
        Error::HttpStatus(HttpStatusError(e))
    }
}
