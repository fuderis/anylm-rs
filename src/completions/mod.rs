#![allow(unused_imports)]

pub mod openai;
pub use openai::OpenAiCompletions;

#[cfg(feature = "anthropic")]
pub use anthropic::AnthropicCompletions;
#[cfg(feature = "anthropic")]
pub mod anthropic;

#[cfg(feature = "google")]
pub mod google;
#[cfg(feature = "google")]
pub use google::GoogleCompletions;

use crate::{api::*, chunk::*, options::Options, prelude::*};

use reqwest::{Client, Proxy};
use std::{sync::Arc, time::Duration};
use tokio::sync::{Mutex, mpsc};

/// The completions response stream reader
#[derive(Debug)]
pub struct Stream {
    pub(crate) rx: mpsc::UnboundedReceiver<Result<Chunk>>,
    pub(crate) handle: tokio::task::JoinHandle<()>,
}

impl Stream {
    pub async fn next(&mut self) -> Option<Result<Chunk>> {
        self.rx.recv().await
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

/// The completions response chunk
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Chunk {
    Text(String),
    Tool(ToolCall),
}

/// Helper buffer for accumulating streaming tool calls
#[derive(Default, Debug)]
pub(crate) struct PartialToolCall {
    pub id: String,
    pub name: String,
    pub args_buf: String,
}

/// Base configuration for LM API chat completions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Completions {
    #[serde(skip)]
    pub api_kind: ApiKind,
    #[serde(skip)]
    pub api_version: Option<String>,
    #[serde(skip)]
    pub api_key: Option<String>,
    #[serde(skip)]
    pub base_url: Option<String>,
    #[serde(skip)]
    pub proxy: Option<Proxy>,
    #[serde(skip)]
    pub timeout: Duration,
    pub model: String,
    pub max_tokens: i32,
    pub temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<Schema>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<Tool>,
    pub tokens_count: usize,
}

impl Completions {
    pub fn new(kind: ApiKind) -> Self {
        Self {
            api_kind: kind,
            api_version: if kind.is_anthropic() {
                Some("2023-06-01".to_string())
            } else {
                None
            },
            api_key: None,
            base_url: None,
            proxy: None,
            timeout: Duration::from_secs(600),
            model: String::new(),
            max_tokens: if kind.is_anthropic() { 8192 } else { -1 },
            temperature: 0.7,
            tokens_count: 0,
            schema: None,
            tools: Vec::new(),
        }
    }

    // main constructors for protocol providers
    pub fn openai() -> OpenAiCompletions {
        OpenAiCompletions(Self::new(ApiKind::OpenAi))
    }

    #[cfg(feature = "anthropic")]
    pub fn anthropic() -> AnthropicCompletions {
        AnthropicCompletions(Self::new(ApiKind::Anthropic))
    }

    #[cfg(feature = "google")]
    pub fn google() -> GoogleCompletions {
        GoogleCompletions(Self::new(ApiKind::Google))
    }

    // builder methods directly for Completions
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

    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    pub fn max_tokens(mut self, count: i32) -> Self {
        self.max_tokens = count;
        self
    }

    pub fn schema(mut self, schema: Schema) -> Self {
        self.schema = Some(schema);
        self
    }

    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools.extend(tools);
        self
    }

    pub fn tool(mut self, tool: Tool) -> Self {
        self.tools.push(tool);
        self
    }

    /// Checks whether the provider's default `base_url` is used
    pub fn is_default_base_url(&self) -> bool {
        match self.base_url.as_deref() {
            None => true,
            Some(url) => {
                url.trim_end_matches('/') == self.api_kind.default_host().trim_end_matches('/')
            }
        }
    }

    /// Returns the resolved `base_url` (or default if not specified)
    pub fn resolve_base_url(&self) -> &str {
        self.base_url
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.api_kind.default_host())
    }

    /// Returns the API key from the explicit field or default env var (if `base_url` is default)
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

    pub(crate) fn build_client(&mut self) -> Result<Client> {
        let mut builder = Client::builder().timeout(self.timeout);
        if let Some(proxy) = self.proxy.take() {
            builder = builder.proxy(proxy).danger_accept_invalid_certs(true);
        }
        Ok(builder.build()?)
    }

    pub(crate) fn build_url(&self) -> String {
        let host = self.resolve_base_url();

        format!(
            "{}{}{}",
            host,
            if host.ends_with('/') { "" } else { "/" },
            self.api_kind.completions_path(&self.model)
        )
    }

    pub async fn send(self, messages: Arc<Mutex<Messages>>) -> Result<Stream> {
        match self.api_kind {
            ApiKind::OpenAi => OpenAiCompletions(self).send(messages).await,
            ApiKind::Anthropic => AnthropicCompletions(self).send(messages).await,
            ApiKind::Google => GoogleCompletions(self).send(messages).await,
        }
    }
}

macro_rules! impl_completions_builders {
    ($type:ty) => {
        impl $type {
            pub fn version(mut self, version: impl Into<String>) -> Self {
                self.0.api_version = Some(version.into());
                self
            }

            pub fn read_key(mut self, env_var: &str) -> Result<Self> {
                self.0.api_key = Some(std::env::var(env_var)?);
                Ok(self)
            }

            pub fn key(mut self, key: impl Into<String>) -> Self {
                let key_str = key.into();
                if !key_str.is_empty() {
                    self.0.api_key = Some(key_str);
                }
                self
            }

            pub fn base_url(mut self, url: impl Into<String>) -> Self {
                let url_str = url.into();
                if !url_str.is_empty() {
                    self.0.base_url = Some(url_str);
                }
                self
            }

            pub fn proxy(mut self, proxy: Proxy) -> Self {
                self.0.proxy = Some(proxy);
                self
            }

            pub fn timeout(mut self, dur: Duration) -> Self {
                self.0.timeout = dur;
                self
            }

            pub fn timeout_secs(mut self, secs: u64) -> Self {
                self.0.timeout = Duration::from_secs(secs);
                self
            }

            pub fn timeout_ms(mut self, ms: u64) -> Self {
                self.0.timeout = Duration::from_millis(ms);
                self
            }

            pub fn model(mut self, model: impl Into<String>) -> Self {
                self.0.model = model.into();
                self
            }

            pub fn temperature(mut self, temperature: f32) -> Self {
                self.0.temperature = temperature;
                self
            }

            pub fn max_tokens(mut self, count: i32) -> Self {
                self.0.max_tokens = count;
                self
            }

            pub fn schema(mut self, schema: Schema) -> Self {
                self.0.schema = Some(schema);
                self
            }

            pub fn tools(mut self, tools: Vec<Tool>) -> Self {
                self.0.tools.extend(tools);
                self
            }

            pub fn tool(mut self, tool: Tool) -> Self {
                self.0.tools.push(tool);
                self
            }
        }
    };
}

pub(crate) use impl_completions_builders;

impl TryFrom<Options> for Completions {
    type Error = DynError;

    fn try_from(ops: Options) -> Result<Self> {
        let mut this = Self::new(ops.kind)
            .model(ops.model)
            .max_tokens(ops.max_tokens.unwrap_or(8096))
            .temperature(ops.temperature.unwrap_or(0.6));

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
