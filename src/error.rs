use crate::prelude::*;
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

    #[display = "Incorrect context - missing a new user request"]
    IncorrectContext,

    #[display = "Encoded base64 string is invalid"]
    InvalidBase64Url,

    #[display = "AI-generation error: {}"]
    ResponseError(ResponseError),
}

/// The LM error message
#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseErrorMessage {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<u32>,
    pub message: String,
    #[serde(default)]
    #[serde(flatten)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, JsonValue>,
}

impl std::fmt::Display for ResponseErrorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}",
            self.message,
            json::to_string(&self.extra).unwrap()
        )
    }
}

/// The LM error structure
#[derive(Debug, Display, Serialize, Deserialize)]
#[display = "{error}"]
pub struct ResponseError {
    pub error: ResponseErrorMessage,
}

/// The simple error structure
#[derive(Debug, Deserialize)]
struct ResponseSimpleError {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<u32>,
    pub error: String,
    #[serde(default)]
    #[serde(flatten)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, JsonValue>,
}

impl ResponseError {
    /// Parse string from response buffer
    pub fn from_str(s: &str) -> Option<Self> {
        if let Ok(error) = json::from_str::<ResponseError>(&s) {
            Some(error)
        } else if let Ok(error) = json::from_str::<ResponseErrorMessage>(&s)
            && !error.message.is_empty()
        {
            Some(Self { error })
        } else if let Ok(error) = json::from_str::<ResponseSimpleError>(&s) {
            Some(Self {
                error: ResponseErrorMessage {
                    code: error.code,
                    message: error.error,
                    extra: error.extra,
                },
            })
        } else {
            None
        }
    }
}
