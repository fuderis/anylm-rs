use crate::{api::*, chunk::ResponseError, options::Options, prelude::*};

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
    /// The API standard
    #[serde(skip)]
    pub api_kind: ApiKind,
    /// The API authorization key
    #[serde(skip)]
    pub api_key: Option<String>,
    /// The custom server base URL
    #[serde(skip)]
    pub base_url: Option<String>,
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
    pub fn new(kind: ApiKind) -> Self {
        Self {
            api_kind: kind,

            api_version: {
                #[cfg(feature = "anthropic")]
                {
                    if kind.is_anthropic() {
                        Some("2023-06-01".to_string())
                    } else {
                        None
                    }
                }
                #[cfg(not(feature = "anthropic"))]
                {
                    None
                }
            },
            api_key: None,
            base_url: None,
            proxy: None,
            timeout: Duration::from_secs(30),
            model: String::new(),
            input: Vec::new(),
            search: None,
        }
    }

    /// Creates a new OpenAI embeddings request
    pub fn openai() -> Self {
        Self::new(ApiKind::OpenAi)
    }

    /// Creates a new Anthropic embeddings request
    #[cfg(feature = "anthropic")]
    pub fn anthropic() -> Self {
        Self::new(ApiKind::Anthropic)
    }

    /// Creates a new Google Gemini AI embeddings request
    #[cfg(feature = "google")]
    pub fn google() -> Self {
        Self::new(ApiKind::Google)
    }

    // --- Builder Methods ---

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.api_version = Some(version.into());
        self
    }

    pub fn read_key(self, env_var: &str) -> Result<Self> {
        let key = std::env::var(env_var)?;
        Ok(self.key(key))
    }

    pub fn key(mut self, key: impl Into<String>) -> Self {
        let key_str = key.into();
        if !key_str.is_empty() {
            self.api_key = Some(key_str);
        }
        self
    }

    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        let url_str = url.into();
        if !url_str.is_empty() {
            self.base_url = Some(url_str);
        }
        self
    }

    pub fn proxy(mut self, proxy: Proxy) -> Self {
        self.proxy = Some(proxy);
        self
    }

    pub fn timeout(mut self, dur: Duration) -> Self {
        self.timeout = dur;
        self
    }

    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout = Duration::from_secs(secs);
        self
    }

    pub fn timeout_ms(mut self, ms: u64) -> Self {
        self.timeout = Duration::from_millis(ms);
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn input(mut self, input: impl Into<String>) -> Self {
        let val = input.into();
        if !val.is_empty() {
            self.input.push(val);
        }
        self
    }

    pub fn inputs(mut self, inputs: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.input.extend(inputs.into_iter().map(Into::into));
        self
    }

    pub fn search(mut self, search: EmbeddingSearch) -> Self {
        self.search = Some(search);
        self
    }

    pub fn document(self) -> Self {
        self.search(EmbeddingSearch::Document)
    }

    pub fn query(self) -> Self {
        self.search(EmbeddingSearch::Query)
    }

    // --- Dynamic Resolvers ---

    /// Проверяет, используется ли дефолтный base_url провайдера
    pub fn is_default_base_url(&self) -> bool {
        match self.base_url.as_deref() {
            None => true,
            Some(url) => {
                url.trim_end_matches('/') == self.api_kind.default_host().trim_end_matches('/')
            }
        }
    }

    /// Возвращает итоговый base_url (или дефолтный)
    pub fn resolve_base_url(&self) -> &str {
        self.base_url
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.api_kind.default_host())
    }

    /// Возвращает API-ключ: из явного поля, либо из дефолтного env_var (если base_url дефолтный)
    pub fn resolve_api_key(&self) -> String {
        if let Some(key) = &self.api_key
            && !key.is_empty()
        {
            return key.clone();
        }

        if self.is_default_base_url() {
            if let Some(env_var) = self.api_kind.default_env_var() {
                if let Ok(env_key) = std::env::var(env_var) {
                    return env_key;
                }
            }
        }

        String::new()
    }

    pub(crate) fn build_url(&self) -> String {
        let host = self.resolve_base_url();

        format!(
            "{}{}{}",
            host,
            if host.ends_with('/') { "" } else { "/" },
            self.api_kind.embeddings_path(&self.model)
        )
    }

    /// Sends the request to LM server
    pub async fn send(mut self) -> Result<EmbeddingsData> {
        let url = self.build_url();
        let api_key = self.resolve_api_key();

        // serialize request data:
        let mut data = json::to_value(&self).map_err(Error::from)?;
        let obj = data.as_object_mut().unwrap();

        #[cfg(feature = "google")]
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
        let mut builder = Client::builder().timeout(self.timeout);
        if let Some(proxy) = self.proxy.take() {
            builder = builder.proxy(proxy).danger_accept_invalid_certs(true);
        }

        let client = builder.build()?;

        // send request:
        #[allow(unused_mut)]
        let mut request = client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&obj);

        // set api key:
        #[cfg(feature = "google")]
        if self.api_kind.is_google() {
            request = request.header("x-goog-api-key", &api_key);
        } else {
            request = request.header(header::AUTHORIZATION, format!("Bearer {api_key}"));
        }
        #[cfg(not(feature = "google"))]
        {
            request = request.header(header::AUTHORIZATION, format!("Bearer {api_key}"));
        }

        let response = request.send().await.map_err(Error::from)?;
        let output = response.text().await?;

        // check for an error:
        if let Some(e) = ResponseError::from_str(&output) {
            return Err(Error::ResponseError(e).into());
        }

        // parse response:
        let embeddings = json::from_str(&output)?;

        Ok(embeddings)
    }
}

impl TryFrom<Options> for Embeddings {
    type Error = DynError;

    fn try_from(ops: Options) -> Result<Self> {
        let mut this = Self::new(ops.kind).model(ops.model);

        if let Some(base_url) = ops.base_url {
            this = this.base_url(base_url);
        }

        if let Some(env_var) = ops.env_var.as_ref() {
            if let Ok(key) = std::env::var(env_var) {
                this = this.key(key);
            }
        }

        if let Some(proxy) = ops.proxy.as_ref() {
            this = this.proxy(Proxy::all(proxy.to_owned())?);
        }

        Ok(this)
    }
}
