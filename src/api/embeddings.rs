use super::ApiKind;
use crate::{error::ResponseError, prelude::*};
use reqwest::{Client, Proxy, header};
use std::time::Duration;

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
    pub server: Option<String>,
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
}

impl Embeddings {
    /// Creates a new LM embeddings request
    pub fn new(kind: ApiKind, key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            server: if kind == ApiKind::LmStudio {
                Some(str!("http://127.0.0.1:1234"))
            } else {
                None
            },
            api_version: if kind.is_anthropic_standart() {
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

    /// Sets the LM API authorization key
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.api_key = key.into();
    }
    /// Sets the LM API authorization key
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.set_key(key);
        self
    }

    /// Sets the custom LM API server URL host
    pub fn set_server(&mut self, url: impl Into<String>) {
        self.server = Some(url.into());
    }
    /// Sets the custom LM API server URL host
    pub fn server(mut self, url: impl Into<String>) -> Self {
        self.set_server(url);
        self
    }

    /// Sets a proxy tunnel settings
    pub fn set_proxy(&mut self, proxy: Proxy) {
        self.proxy = Some(proxy);
    }
    /// Sets a proxy tunnel settings
    pub fn proxy(mut self, proxy: Proxy) -> Self {
        self.set_proxy(proxy);
        self
    }

    /// Sets a connection timeout
    pub fn set_timeout(&mut self, dur: Duration) {
        self.timeout = dur;
    }
    /// Sets a connection timeout
    pub fn timeout(mut self, dur: Duration) -> Self {
        self.set_timeout(dur);
        self
    }

    /// Sets a connection timeout (from seconds)
    pub fn set_timeout_secs(&mut self, secs: u64) {
        self.timeout = Duration::from_secs(secs);
    }
    /// Sets a connection timeout (from seconds)
    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.set_timeout_secs(secs);
        self
    }

    /// Sets a connection timeout (from millis)
    pub fn set_timeout_ms(&mut self, secs: u64) {
        self.timeout = Duration::from_millis(secs);
    }
    /// Sets a connection timeout (from millis)
    pub fn timeout_ms(mut self, secs: u64) -> Self {
        self.set_timeout_ms(secs);
        self
    }

    /// Sets the LM model name
    pub fn set_model(&mut self, model: impl Into<String>) {
        self.model = model.into();
    }
    /// Sets the LM model name
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.set_model(model);
        self
    }

    /// Adds embedding input
    pub fn add_input(&mut self, input: impl Into<String>) {
        self.input.push(input.into());
    }
    /// Adds embedding input
    pub fn input(mut self, input: impl Into<String>) -> Self {
        self.add_input(input);
        self
    }

    /// Sends the request to LM server
    pub async fn send(&mut self) -> Result<EmbeddingsData> {
        // generate URL:
        let url = if let Some(url) = &self.server {
            self.api_kind.custom_embeddings_url(url)
        } else {
            self.api_kind.embeddings_url()
        };

        // serialize request data:
        let mut data = json::to_value(&self).map_err(Error::from)?;
        let obj = data.as_object_mut().unwrap();

        // create client:
        let mut client = Client::builder().timeout(self.timeout);

        // set proxy:
        if let Some(proxy) = self.proxy.take() {
            client = client.proxy(proxy);
            client = client.danger_accept_invalid_certs(true); // VPN SSL
        }

        // send request:
        let response = client
            .build()?
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::AUTHORIZATION, &self.api_key)
            .json(&obj)
            .send()
            .await
            .map_err(Error::from)?;

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
