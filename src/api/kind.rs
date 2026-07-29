use crate::prelude::*;

/// Supported LLM API protocol standards
#[derive(Clone, Copy, Debug, Display, Serialize, Deserialize, Eq, PartialEq, Hash, Default)]
#[serde(rename_all = "lowercase")]
pub enum ApiKind {
    /// OpenAI-compatible API (/v1/chat/completions)
    #[default]
    OpenAi,
    /// Anthropic API (/v1/messages)
    Anthropic,
    /// Google Gemini API (/v1beta/models)
    Google,
}

impl ApiKind {
    pub fn is_openai(&self) -> bool {
        matches!(self, Self::OpenAi)
    }

    pub fn is_anthropic(&self) -> bool {
        matches!(self, Self::Anthropic)
    }

    pub fn is_google(&self) -> bool {
        matches!(self, Self::Google)
    }

    /// Returns default API host for the protocol
    pub fn default_host(&self) -> &'static str {
        match self {
            ApiKind::OpenAi => "https://api.openai.com/v1",
            ApiKind::Anthropic => "https://api.anthropic.com/v1",
            ApiKind::Google => "https://generativelanguage.googleapis.com/v1beta",
        }
    }

    /// Returns default ENV var name for receive API key
    pub fn default_env_var(&self) -> Option<&'static str> {
        match self {
            ApiKind::OpenAi => Some("OPENAI_API_KEY"),
            ApiKind::Anthropic => Some("ANTHROPIC_API_KEY"),
            ApiKind::Google => Some("GEMINI_API_KEY"),
        }
    }

    /// Returns completions path for current protocol
    pub fn completions_path(&self, model: &str) -> String {
        match self {
            Self::Google => format!("v1beta/models/{}:streamGenerateContent?alt=sse", model),
            Self::Anthropic => "v1/messages".to_string(),
            Self::OpenAi => "v1/chat/completions".to_string(),
        }
    }

    /// Returns embeddings path
    pub fn embeddings_path(&self, model: &str) -> String {
        match self {
            Self::Google => format!("v1beta/models/{}:embedContent", model),
            Self::OpenAi | Self::Anthropic => "v1/embeddings".to_string(),
        }
    }
}
