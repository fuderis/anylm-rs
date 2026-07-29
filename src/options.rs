use crate::{api::ApiKind, prelude::*};

/// The AI options
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Options {
    #[serde(rename = "type")]
    pub kind: ApiKind,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_var: Option<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,

    pub model: String,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

impl Options {
    pub fn new(kind: ApiKind) -> Self {
        Self {
            kind,
            ..Default::default()
        }
    }

    pub fn openai() -> Self {
        Self::new(ApiKind::OpenAi)
    }

    pub fn anthropic() -> Self {
        Self::new(ApiKind::Anthropic)
    }

    pub fn google() -> Self {
        Self::new(ApiKind::Google)
    }

    pub fn kind(mut self, kind: ApiKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        let val = url.into();
        self.base_url = if val.is_empty() { None } else { Some(val) };
        self
    }

    pub fn env_var(mut self, var: impl Into<String>) -> Self {
        let val = var.into();
        self.env_var = if val.is_empty() { None } else { Some(val) };
        self
    }

    pub fn proxy(mut self, proxy: impl Into<String>) -> Self {
        let val = proxy.into();
        self.proxy = if val.is_empty() { None } else { Some(val) };
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn max_tokens(mut self, tokens: i32) -> Self {
        self.max_tokens = Some(tokens);
        self
    }

    pub fn temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }
}
