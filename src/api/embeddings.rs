use super::ApiKind;
use crate::{AiOptions, chunk::ResponseError, prelude::*};

use reqwest::{Client, Proxy, header};
use std::time::Duration;

/// The embedding search optimization
#[derive(Debug, Display, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum EmbeddingSearch {
    /// Uses for save context
    #[display(fmt = "search_document")]
    Document,
    /// Uses for search context
    #[display(fmt = "search_query")]
    Query,
}

/// The embeddings usage info
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Usage {
    pub total_tokens: usize,
}

/// The embeddings response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbeddingsData {
    pub object: String,
    pub data: Vec<Embedding>,
    pub model: String,
    pub usage: Usage,
}

/// The embedding chunk
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Embedding {
    pub object: String,
    pub index: usize,
    pub embedding: Vec<f32>,
}

/// The LM API embeddings request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Embeddings {
    /// The API version
    #[serde(skip)]
    pub api_version: Option<String>,
    /// The API standart
    #[serde(skip)]
    pub api_kind: ApiKind,
    /// The API authorization key
    #[serde(skip)]
    pub api_key: String,
    /// The custom server host
    #[serde(skip)]
    pub host: Option<String>,
    /// The proxy tunnel settings
    #[serde(skip)]
    pub proxy: Option<Proxy>,
    /// The connection timeout
    #[serde(skip)]
    pub timeout: Duration,
    /// The AI model name
    pub model: String,
    /// The input texts
    pub input: Vec<String>,
    /// The embedding search type optimization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<EmbeddingSearch>,
}

impl Embeddings {
    /// Creates a new LM embeddings request
    pub fn new(kind: ApiKind, key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            host: if kind.is_lmstudio() {
                Some(str!("http://127.0.0.1:1234"))
            } else {
                None
            },
            api_version: if kind.is_anthropic() {
                Some(str!("2023-06-01"))
            } else {
                None
            },
            api_kind: kind,
            api_key: key.into(),
            proxy: None,
            timeout: Duration::from_secs(30),
            model: model.into(),
            input: Vec::new(),
            search: None,
        }
    }

    /// Creates a new OpenAI (ChatGPT) embeddings request
    pub fn openai(key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new(ApiKind::OpenAI, key, model)
    }

    /// Creates a new Anthropic (Voyage) embeddings request
    pub fn anthropic(key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new(ApiKind::Voyage, key, model)
    }

    /// Creates a new LM Studio embeddings request
    pub fn lmstudio(key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new(ApiKind::LmStudio, key, model)
    }

    /// Creates a new ChatGPT embeddings request
    pub fn chatgpt(key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new(ApiKind::ChatGpt, key, model)
    }

    /// Creates a new Cerebras embeddings request
    pub fn cerebras(key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new(ApiKind::Cerebras, key, model)
    }

    /// Creates a new Claude (Voyage) embeddings request
    pub fn claude(key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new(ApiKind::Voyage, key, model)
    }

    /// Creates a new OpenRouter embeddings request
    pub fn openrouter(key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new(ApiKind::OpenRouter, key, model)
    }

    /// Creates a new Perplexity AI embeddings request
    pub fn perplexity(key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new(ApiKind::Perplexity, key, model)
    }

    /// Creates a new Voyage embeddings request
    pub fn voyage(key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new(ApiKind::Voyage, key, model)
    }

    /// Creates a new Google AI embeddings request
    pub fn google(key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new(ApiKind::Google, key, model)
    }

    /// Creates a new Google Gemini AI embeddings request
    pub fn gemini(key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new(ApiKind::Gemini, key, model)
    }

    /// Sets the LM API authorization key
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.api_key = key.into();
    }
    /// Sets the LM API authorization key
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.set_key(key);
        self
    }

    /// Sets the custom API server host
    pub fn host(mut self, url: impl Into<String>) -> Self {
        self.host = Some(url.into());
        self
    }

    /// Sets a proxy tunnel settings
    pub fn proxy(mut self, proxy: Proxy) -> Self {
        self.proxy = Some(proxy);
        self
    }

    /// Sets a connection timeout
    pub fn timeout(mut self, dur: Duration) -> Self {
        self.timeout = dur;
        self
    }

    /// Sets a connection timeout (from seconds)
    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout = Duration::from_secs(secs);
        self
    }

    /// Sets a connection timeout (from millis)
    pub fn timeout_ms(mut self, secs: u64) -> Self {
        self.timeout = Duration::from_millis(secs);
        self
    }

    /// Sets the LM model name
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Adds embedding input
    pub fn input(mut self, input: impl Into<String>) -> Self {
        self.input.push(input.into());
        self
    }

    /// Sets the embedding search type optimization
    pub fn search(mut self, search: EmbeddingSearch) -> Self {
        self.search = Some(search);
        self
    }

    /// Shortcut to set search optimization to Document
    pub fn document(self) -> Self {
        self.search(EmbeddingSearch::Document)
    }

    /// Shortcut to set search optimization to Query
    pub fn query(self) -> Self {
        self.search(EmbeddingSearch::Query)
    }

    /// Sends the request to LM server
    pub async fn send(&mut self) -> Result<EmbeddingsData> {
        // generate URL:
        let url = if let Some(host) = &self.host {
            str!(
                "{host}{}{}",
                if host.ends_with("/") { "" } else { "/" },
                self.api_kind.embeddings_path(&self.model)
            )
        } else {
            str!(
                "{}/{}",
                self.api_kind.host(),
                self.api_kind.embeddings_path(&self.model)
            )
        };

        // serialize request data:
        let mut data = json::to_value(&self).map_err(Error::from)?;
        let obj = data.as_object_mut().unwrap();

        if self.api_kind.is_google() {
            let parts: Vec<JsonValue> = self
                .input
                .iter()
                .map(|text| json!({ "text": text }))
                .collect();

            *obj = json!({
                "content": { "parts": parts }
            })
            .as_object()
            .unwrap()
            .clone();
        }

        // create client & configure proxy:
        let mut client = Client::builder().timeout(self.timeout);
        if let Some(proxy) = self.proxy.take() {
            client = client.proxy(proxy);
            client = client.danger_accept_invalid_certs(true); // VPN SSL
        }

        // send request:
        let mut request = client
            .build()?
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&obj);

        // set api key:
        if self.api_kind.is_google() {
            request = request.header("x-goog-api-key", &self.api_key);
        } else {
            request = request.header(header::AUTHORIZATION, str!("Bearer {}", self.api_key));
        }

        let response = request.send().await.map_err(Error::from)?;
        let output = response.text().await?;

        // check for an error:
        if let Some(e) = ResponseError::from_str(&output) {
            return Err(Error::ResponseError(e).into());
        }

        // else parse response:
        let embeddings = json::from_str(&output)?;

        Ok(embeddings)
    }
}

impl TryFrom<AiOptions> for Embeddings {
    type Error = DynError;

    fn try_from(ops: AiOptions) -> Result<Self> {
        let mut this = Self::new(
            // choose AI service
            ops.kind,
            // read API key
            if let Some(v) = ops.env_var.as_ref() {
                std::env::var(v).unwrap_or_default()
            } else {
                String::new()
            },
            // choose model
            ops.model,
        );

        // set default server host:
        if let Some(host) = ops.server.as_ref() {
            this = this.host(host.to_owned());
        }
        // set proxy options:
        if let Some(proxy) = ops.proxy.as_ref() {
            this = this.proxy(Proxy::all(proxy.to_owned())?);
        }

        Ok(this)
    }
}
