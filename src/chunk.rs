use crate::prelude::*;

/// The AI response chunk
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ResponseChunk {
    OpenAi(OpenAIChunk),
    Anthropic(AnthropicChunk),
    Google(GoogleChunk),
    Error(ResponseErrorMessage),
}

//       OPENAI:

#[derive(Debug, Deserialize)]
pub struct OpenAIChunk {
    pub choices: Vec<OpenAIChoice>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct OpenAIChoice {
    pub delta: OpenAIDelta,
    #[serde(default)]
    pub finish_reason: Option<String>,
}
#[derive(Debug, Deserialize)]
pub struct OpenAIDelta {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCallDelta>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ToolCallDelta {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub _kind: Option<String>,
    #[serde(default)]
    pub index: Option<usize>,
    #[serde(default)]
    pub function: Option<FunctionDelta>,
}
#[allow(dead_code)]
#[derive(Debug, Deserialize, Default)]
pub struct FunctionDelta {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub arguments: Option<String>,
}

//       ANTHROPIC:

#[derive(Debug, Deserialize)]
pub struct AnthropicChunk {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub index: Option<usize>,
    pub delta: Option<AnthropicDelta>,
    pub content_block: Option<ContentBlock>,
}

#[derive(Debug, Deserialize)]
pub struct AnthropicDelta {
    #[serde(rename = "type")]
    pub kind: String,
    pub text: Option<String>,
    #[serde(rename = "partial_json")]
    pub partial_json: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ContentBlock {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub kind: String,
    pub name: String,
}

//       GOOGLE:

#[derive(Debug, Deserialize)]
pub struct GoogleChunk {
    pub candidates: Vec<GeminiCandidate>,
    #[serde(default)]
    pub usage_metadata: Option<JsonValue>,
}

#[derive(Debug, Deserialize)]
pub struct GeminiCandidate {
    pub content: Option<GeminiContent>,
    #[serde(rename = "finishReason")]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GeminiContent {
    pub parts: Vec<GeminiPart>,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GeminiPart {
    Text {
        text: String,
    },
    FunctionCall {
        #[serde(rename = "functionCall")]
        function_call: GeminiFunctionCall,
    },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GeminiFunctionCall {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    pub args: JsonValue,
}

//       ERROR

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
#[display(fmt = "{error}")]
pub struct ResponseError {
    pub error: ResponseErrorMessage,
}

/// The simple error structure
#[derive(Debug, Deserialize)]
pub struct ResponseSimpleError {
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
