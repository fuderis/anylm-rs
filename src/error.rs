use crate::chunk::ResponseError;
use macron::{Display, Error, From};

/// The error
#[derive(Debug, Display, Error, From)]
pub enum Error {
    Io(std::io::Error),
    Json(serde_json::Error),
    Request(reqwest::Error),

    #[display(fmt = "Context is overflowing, try deleting old messages.")]
    ContextOverflowing,

    #[display(fmt = "Bad request - missing a new user message.")]
    BadRequest,

    #[display(fmt = "Base64 encoded string is invalid.")]
    InvalidBase64Url,

    #[display(fmt = "AI-generation error: {0}")]
    ResponseError(ResponseError),
}
