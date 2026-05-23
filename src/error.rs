use crate::chunk::ResponseError;
use macron::{Display, Error, From};

/// The error
#[derive(Debug, Display, Error, From)]
pub enum Error {
    #[from]
    Io(std::io::Error),

    #[from]
    Json(serde_json::Error),

    #[from]
    Request(reqwest::Error),

    #[display = "Context is overflowing, try deleting old messages."]
    ContextOverflowing,

    #[display = "Bad request - missing a new user message."]
    BadRequest,

    #[display = "Base64 encoded string is invalid."]
    InvalidBase64Url,

    #[display = "AI-generation error: {}"]
    ResponseError(ResponseError),
}
