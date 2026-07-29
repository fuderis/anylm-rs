use super::*;
use crate::{ToolCall, ToolCallFunction, chunk::*, options::Options, prelude::*};

use atoman::{StreamExt, StreamReader, StreamReader};
use reqwest::{Client, Proxy, header};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::{Mutex, mpsc};

/// The completions response stream reader
#[derive(Debug)]
pub struct AiStream {
    rx: mpsc::UnboundedReceiver<Result<Chunk>>,
    handle: tokio::task::JoinHandle<()>,
}

impl AiStream {
    /// Read a next completions response chunk
    pub async fn next(&mut self) -> Option<Result<Chunk>> {
        self.rx.recv().await
    }
}

impl Drop for AiStream {
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
struct PartialToolCall {
    id: String,
    name: String,
    args_buf: String,
}

// ============================================================================
// BASE COMPLETIONS STRUCT
// ============================================================================

/// Base configuration for LM API chat completions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Completions {
    #[serde(skip)]
    pub api_kind: ApiKind,
    #[serde(skip)]
    pub api_version: Option<String>,
    #[serde(skip)]
    pub api_key: String,
    #[serde(skip)]
    pub host: Option<String>,
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
            api_key: key.into(),
            proxy: None,
            timeout: Duration::from_secs(600),
            model: model.into(),
            max_tokens: if kind.is_anthropic() { 8096 } else { -1 },
            temperature: 0.7,
            tokens_count: 0,
            schema: None,
            tools: Vec::new(),
            api_kind: kind,
        }
    }

    // Constructors for specialized types
    pub fn openai(key: impl Into<String>, model: impl Into<String>) -> OpenAiCompletions {
        OpenAiCompletions(Self::new(ApiKind::OpenAI, key, model))
    }

    pub fn anthropic(key: impl Into<String>, model: impl Into<String>) -> AnthropicCompletions {
        AnthropicCompletions(Self::new(ApiKind::Anthropic, key, model))
    }

    pub fn google(key: impl Into<String>, model: impl Into<String>) -> GoogleCompletions {
        GoogleCompletions(Self::new(ApiKind::Google, key, model))
    }

    pub fn lmstudio(key: impl Into<String>, model: impl Into<String>) -> OpenAiCompletions {
        OpenAiCompletions(Self::new(ApiKind::LmStudio, key, model))
    }

    pub fn chatgpt(key: impl Into<String>, model: impl Into<String>) -> OpenAiCompletions {
        OpenAiCompletions(Self::new(ApiKind::ChatGpt, key, model))
    }

    pub fn cerebras(key: impl Into<String>, model: impl Into<String>) -> OpenAiCompletions {
        OpenAiCompletions(Self::new(ApiKind::Cerebras, key, model))
    }

    pub fn claude(key: impl Into<String>, model: impl Into<String>) -> AnthropicCompletions {
        AnthropicCompletions(Self::new(ApiKind::Claude, key, model))
    }

    pub fn openrouter(key: impl Into<String>, model: impl Into<String>) -> OpenAiCompletions {
        OpenAiCompletions(Self::new(ApiKind::OpenRouter, key, model))
    }

    pub fn perplexity(key: impl Into<String>, model: impl Into<String>) -> OpenAiCompletions {
        OpenAiCompletions(Self::new(ApiKind::Perplexity, key, model))
    }

    pub fn gemini(key: impl Into<String>, model: impl Into<String>) -> GoogleCompletions {
        GoogleCompletions(Self::new(ApiKind::Gemini, key, model))
    }

    // Common helper to build reqwest client
    fn build_client(&mut self) -> Result<Client> {
        let mut builder = Client::builder().timeout(self.timeout);
        if let Some(proxy) = self.proxy.take() {
            builder = builder.proxy(proxy).danger_accept_invalid_certs(true);
        }
        Ok(builder.build()?)
    }

    // Common helper to build full URL
    fn build_url(&self) -> String {
        if let Some(host) = &self.host {
            str!(
                "{host}{}{}",
                if host.ends_with('/') { "" } else { "/" },
                self.api_kind.completions_path(&self.model)
            )
        } else {
            str!(
                "{}/{}",
                self.api_kind.host(),
                self.api_kind.completions_path(&self.model)
            )
        }
    }

    // Context validation
    async fn validate_context(&self, messages: &Arc<Mutex<Messages>>) -> Result<()> {
        if self.max_tokens > 0 {
            if self.tokens_count > self.max_tokens as usize {
                return Err(Error::ContextOverflowing.into());
            }
            let lock = messages.lock().await;
            if let Some(msg) = lock.messages.last()
                && !msg.role.is_user()
            {
                return Err(Error::BadRequest.into());
            }
        }
        Ok(())
    }
}

// Macro to implement common builder chain methods on child structs
macro_rules! impl_completions_builders {
    ($type:ty) => {
        impl $type {
            pub fn version(mut self, version: impl Into<String>) -> Self {
                self.0.api_version = Some(version.into());
                self
            }
            pub fn key(mut self, key: impl Into<String>) -> Self {
                self.0.api_key = key.into();
                self
            }
            pub fn host(mut self, url: impl Into<String>) -> Self {
                self.0.host = Some(url.into());
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

// ============================================================================
// OPENAI PROVIDER
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenAiCompletions(pub Completions);

impl std::ops::Deref for OpenAiCompletions {
    type Target = Completions;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for OpenAiCompletions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl_completions_builders!(OpenAiCompletions);

impl OpenAiCompletions {
    pub async fn send(&mut self, messages: Arc<Mutex<Messages>>) -> Result<AiStream> {
        self.validate_context(&messages).await?;
        let url = self.build_url();

        let mut data = json::to_value(&self.0)?;
        let data_obj = data.as_object_mut().unwrap();
        data_obj.remove("tokens_count");

        let mut msgs_val = json::to_value(&*messages.lock().await)?;
        if let Some(arr) = msgs_val.as_array_mut() {
            for msg in arr {
                if let Some(obj) = msg.as_object_mut() {
                    obj.remove("tokens_count");
                    obj.remove("timestamp");
                }
            }
        }
        data_obj.insert(str!("messages"), msgs_val);
        data_obj.insert(str!("stream"), JsonValue::Bool(true));

        if let Some(schema) = self.schema.take() {
            data_obj.remove("schema");
            data_obj.insert(str!("response_format"), schema.to_openai_format()?);
        }

        if !self.tools.is_empty() {
            let mut tools_json = Vec::new();
            for tool in &self.tools {
                tools_json.push(tool.to_openai_format()?);
            }
            data_obj.insert(str!("tools"), JsonValue::Array(tools_json));
        }

        let client = self.build_client()?;
        let response = client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::ACCEPT, "text/event-stream")
            .header(header::AUTHORIZATION, str!("Bearer {}", self.api_key))
            .json(&data_obj)
            .send()
            .await?;

        let bytes_stream = response.bytes_stream().map(|r| r.map_err(Into::into));
        let reader = StreamReader::read::<ResponseChunk>(bytes_stream);
        let (tx, rx) = mpsc::unbounded_channel();
        let handle = Self::spawn_reader(reader, tx, messages);

        Ok(AiStream { rx, handle })
    }

    fn spawn_reader(
        mut reader: StreamReader<ResponseChunk>,
        tx: mpsc::UnboundedSender<Result<Chunk>>,
        messages: Arc<Mutex<Messages>>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut full_text = str!();
            let mut tool_calls = vec![];
            let mut allocated_tool_ids = vec![];
            let mut tool_buffers = HashMap::<usize, PartialToolCall>::new();

            loop {
                if tx.is_closed() {
                    return;
                }

                match reader.read().await {
                    Ok(Some(chunk)) => match chunk {
                        ResponseChunk::OpenAi(OpenAIChunk { choices }) => {
                            for choice in choices {
                                if let Some(content) = choice.delta.content {
                                    if !content.is_empty() {
                                        full_text.push_str(&content);
                                        if tx.send(Ok(Chunk::Text(content))).is_err() {
                                            return;
                                        }
                                    }
                                }

                                if let Some(tc_list) = choice.delta.tool_calls {
                                    for tc in tc_list {
                                        if let Some(idx) = tc.index {
                                            let entry = tool_buffers.entry(idx).or_default();

                                            // Обновляем id и name ТОЛЬКО если они пришли в чанке (не перезаписываем пустыми)
                                            if let Some(id) = tc.id
                                                && !id.is_empty()
                                            {
                                                entry.id = id;
                                            }
                                            if let Some(fn_delta) = tc.function {
                                                if let Some(name) = fn_delta.name
                                                    && !name.is_empty()
                                                {
                                                    entry.name = name;
                                                }
                                                if let Some(args) = fn_delta.arguments {
                                                    entry.args_buf.push_str(&args);
                                                }
                                            }
                                        }
                                    }
                                }

                                // Сбрасываем только если стрим для данного choice явно завершен
                                // (finish_reason == "tool_calls" или "stop")
                                if choice.finish_reason.is_some() {
                                    Self::flush_buffers(
                                        &mut tool_buffers,
                                        &mut tool_calls,
                                        &mut allocated_tool_ids,
                                        &tx,
                                    );
                                }
                            }
                        }
                        ResponseChunk::Error(err) => {
                            let _ = tx
                                .send(Err(
                                    Error::ResponseError(ResponseError { error: err }).into()
                                ));
                            return;
                        }
                        _ => {}
                    },
                    Ok(None) => break,
                    Err(e) => {
                        let _ = tx.send(Err(e.into()));
                        break;
                    }
                }
            }

            // Финальный флеш для гарантии при закрытии соединения
            Self::flush_buffers(
                &mut tool_buffers,
                &mut tool_calls,
                &mut allocated_tool_ids,
                &tx,
            );

            let mut lock = messages.lock().await;
            lock.add_assistant(vec![full_text.into()], tool_calls);
            for id in allocated_tool_ids {
                lock.push_str(Some(&id), "");
            }
        })
    }

    fn flush_buffers(
        tool_buffers: &mut HashMap<usize, PartialToolCall>,
        tool_calls: &mut Vec<ToolCall>,
        allocated_tool_ids: &mut Vec<String>,
        tx: &mpsc::UnboundedSender<Result<Chunk>>,
    ) {
        for (_, buf) in tool_buffers.drain() {
            if buf.name.is_empty() {
                continue;
            }
            let final_id = if buf.id.is_empty() {
                str!("call_{}", buf.name)
            } else {
                buf.id
            };

            allocated_tool_ids.push(final_id.clone());

            let tool = ToolCall {
                id: final_id,
                kind: str!("function"),
                func: ToolCallFunction {
                    name: buf.name,
                    json_str: buf.args_buf,
                },
            };

            tool_calls.push(tool.clone());
            let _ = tx.send(Ok(Chunk::Tool(tool)));
        }
    }
}

// ============================================================================
// ANTHROPIC PROVIDER
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnthropicCompletions(pub Completions);

impl std::ops::Deref for AnthropicCompletions {
    type Target = Completions;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for AnthropicCompletions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl_completions_builders!(AnthropicCompletions);

impl AnthropicCompletions {
    pub async fn send(&mut self, messages: Arc<Mutex<Messages>>) -> Result<AiStream> {
        self.validate_context(&messages).await?;
        let url = self.build_url();

        let mut data = json::to_value(&self.0)?;
        let data_obj = data.as_object_mut().unwrap();
        data_obj.remove("tokens_count");

        let mut msgs_val = json::to_value(&*messages.lock().await)?;
        if let Some(arr) = msgs_val.as_array_mut() {
            for msg in arr {
                if let Some(obj) = msg.as_object_mut() {
                    obj.remove("tokens_count");
                    obj.remove("timestamp");
                }
            }
        }
        data_obj.insert(str!("messages"), msgs_val);
        data_obj.insert(str!("stream"), JsonValue::Bool(true));

        if let Some(schema) = self.schema.take() {
            data_obj.remove("schema");
            data_obj.insert(str!("output_config"), schema.to_anthropic_format()?);
        }

        if !self.tools.is_empty() {
            let mut tools_json = Vec::new();
            for tool in &self.tools {
                tools_json.push(tool.to_anthropic_format()?);
            }
            data_obj.insert(str!("tools"), JsonValue::Array(tools_json));
        }

        let api_version = self
            .api_version
            .take()
            .unwrap_or_else(|| str!("2023-06-01"));

        let client = self.build_client()?;
        let response = client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::ACCEPT, "text/event-stream")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", api_version)
            .json(&data_obj)
            .send()
            .await?;

        let bytes_stream = response.bytes_stream().map(|r| r.map_err(Into::into));
        let reader = StreamReader::read::<ResponseChunk>(bytes_stream);
        let (tx, rx) = mpsc::unbounded_channel();
        let handle = Self::spawn_reader(reader, tx, messages);

        Ok(AiStream { rx, handle })
    }

    fn spawn_reader(
        mut reader: StreamReader<ResponseChunk>,
        tx: mpsc::UnboundedSender<Result<Chunk>>,
        messages: Arc<Mutex<Messages>>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut full_text = str!();
            let mut tool_calls = vec![];
            let mut allocated_tool_ids = vec![];
            let mut tool_buffers = HashMap::<usize, PartialToolCall>::new();

            loop {
                if tx.is_closed() {
                    return;
                }

                match reader.read().await {
                    Ok(Some(chunk)) => match chunk {
                        ResponseChunk::Anthropic(anth) => {
                            let idx = anth.index.unwrap_or(0);

                            if let Some(block) = anth.content_block {
                                if block.kind == "tool_use" {
                                    let entry = tool_buffers.entry(idx).or_default();
                                    entry.name = block.name;
                                    if let Some(id) = block.id {
                                        entry.id = id;
                                    }
                                }
                            }

                            if let Some(delta) = anth.delta {
                                if let Some(t) = delta.text {
                                    if !t.is_empty() {
                                        full_text.push_str(&t);
                                        if tx.send(Ok(Chunk::Text(t))).is_err() {
                                            return;
                                        }
                                    }
                                }
                                if let Some(pj) = delta.partial_json {
                                    tool_buffers.entry(idx).or_default().args_buf.push_str(&pj);
                                }
                            }

                            // If block completes, process tool call immediately
                            if anth.kind == "content_block_stop" {
                                if let Some(buf) = tool_buffers.remove(&idx) {
                                    Self::emit_tool(
                                        buf,
                                        &mut tool_calls,
                                        &mut allocated_tool_ids,
                                        &tx,
                                    );
                                }
                            }
                        }
                        ResponseChunk::Error(err) => {
                            let _ = tx
                                .send(Err(
                                    Error::ResponseError(ResponseError { error: err }).into()
                                ));
                            return;
                        }
                        _ => {}
                    },
                    Ok(None) => break,
                    Err(e) => {
                        let _ = tx.send(Err(e.into()));
                        break;
                    }
                }
            }

            // Flush remaining buffers at EOF
            for (_, buf) in tool_buffers.drain() {
                Self::emit_tool(buf, &mut tool_calls, &mut allocated_tool_ids, &tx);
            }

            let mut lock = messages.lock().await;
            lock.add_assistant(vec![full_text.into()], tool_calls);
            for id in allocated_tool_ids {
                lock.push_str(Some(&id), "");
            }
        })
    }

    fn emit_tool(
        buf: PartialToolCall,
        tool_calls: &mut Vec<ToolCall>,
        allocated_tool_ids: &mut Vec<String>,
        tx: &mpsc::UnboundedSender<Result<Chunk>>,
    ) {
        if buf.name.is_empty() {
            return;
        }
        let final_id = if buf.id.is_empty() {
            str!("call_{}", buf.name)
        } else {
            buf.id
        };

        allocated_tool_ids.push(final_id.clone());

        let tool = ToolCall {
            id: final_id,
            kind: str!("function"),
            func: ToolCallFunction {
                name: buf.name,
                json_str: buf.args_buf,
            },
        };

        tool_calls.push(tool.clone());
        let _ = tx.send(Ok(Chunk::Tool(tool)));
    }
}

// ============================================================================
// GOOGLE PROVIDER
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GoogleCompletions(pub Completions);

impl std::ops::Deref for GoogleCompletions {
    type Target = Completions;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for GoogleCompletions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl_completions_builders!(GoogleCompletions);

impl GoogleCompletions {
    pub async fn send(&mut self, messages: Arc<Mutex<Messages>>) -> Result<AiStream> {
        self.validate_context(&messages).await?;
        let url = self.build_url();

        let mut data = json::to_value(&self.0)?;
        let data_obj = data.as_object_mut().unwrap();
        data_obj.remove("tokens_count");
        data_obj.remove("model");
        data_obj.insert(str!("stream"), JsonValue::Bool(true));

        // Format Gemini messages into "contents" BEFORE serializing request payload
        let raw_messages = json::to_value(&*messages.lock().await)?;
        if let Some(msg_array) = raw_messages.as_array() {
            let contents: Vec<JsonValue> = msg_array
                .iter()
                .map(|m| {
                    json!({
                        "role": if m["role"] == "assistant" { "model" } else { "user" },
                        "parts": m["content"]
                    })
                })
                .collect();
            data_obj.insert(str!("contents"), json!(contents));
        }

        // Schema formatting for Google
        if let Some(schema) = self.schema.take() {
            data_obj.remove("schema");
            let google_config = schema.to_google_format()?;

            if let Some(config) = data_obj
                .get_mut("generationConfig")
                .and_then(|c| c.as_object_mut())
            {
                if let Some(obj) = google_config.as_object() {
                    for (k, v) in obj {
                        config.insert(k.clone(), v.clone());
                    }
                }
            } else {
                data_obj.insert(str!("generationConfig"), google_config);
            }
        }

        // Tools formatting for Google
        if !self.tools.is_empty() {
            let mut tools_json = Vec::new();
            for tool in &self.tools {
                let formatted_tool = tool.to_google_format()?;
                if tools_json.is_empty() {
                    tools_json.push(formatted_tool);
                } else if let Some(first_tool) = tools_json.get_mut(0) {
                    if let Some(decls) = first_tool
                        .get_mut("function_declarations")
                        .and_then(|d| d.as_array_mut())
                    {
                        let tool_val = tool.to_json_tool()?;
                        decls.push(tool_val);
                    }
                }
            }
            data_obj.insert(str!("tools"), JsonValue::Array(tools_json));
        }

        let client = self.build_client()?;
        let response = client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::ACCEPT, "text/event-stream")
            .header("x-goog-api-key", &self.api_key)
            .json(&data_obj)
            .send()
            .await?;

        let bytes_stream = response.bytes_stream().map(|r| r.map_err(Into::into));
        let reader = StreamReader::read::<ResponseChunk>(bytes_stream);
        let (tx, rx) = mpsc::unbounded_channel();
        let handle = Self::spawn_reader(reader, tx, messages);

        Ok(AiStream { rx, handle })
    }

    fn spawn_reader(
        mut reader: StreamReader<ResponseChunk>,
        tx: mpsc::UnboundedSender<Result<Chunk>>,
        messages: Arc<Mutex<Messages>>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut full_text = str!();
            let mut tool_calls = vec![];
            let mut allocated_tool_ids = vec![];

            loop {
                if tx.is_closed() {
                    return;
                }

                match reader.read().await {
                    Ok(Some(chunk)) => match chunk {
                        ResponseChunk::Google(google) => {
                            for cand in google.candidates {
                                if let Some(content) = cand.content {
                                    for part in content.parts {
                                        match part {
                                            GeminiPart::Text { text } => {
                                                if !text.is_empty() {
                                                    full_text.push_str(&text);
                                                    if tx.send(Ok(Chunk::Text(text))).is_err() {
                                                        return;
                                                    }
                                                }
                                            }
                                            GeminiPart::FunctionCall { function_call } => {
                                                let tool_name = function_call.name;
                                                let final_id = function_call
                                                    .id
                                                    .unwrap_or_else(|| str!("call_{}", tool_name));

                                                allocated_tool_ids.push(final_id.clone());

                                                // Google returns structured JSON args directly, no chunk accumulation required
                                                let tool = ToolCall {
                                                    id: final_id,
                                                    kind: str!("function"),
                                                    func: ToolCallFunction {
                                                        name: tool_name,
                                                        json_str: function_call.args.to_string(),
                                                    },
                                                };

                                                tool_calls.push(tool.clone());
                                                if tx.send(Ok(Chunk::Tool(tool))).is_err() {
                                                    return;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        ResponseChunk::Error(err) => {
                            let _ = tx
                                .send(Err(
                                    Error::ResponseError(ResponseError { error: err }).into()
                                ));
                            return;
                        }
                        _ => {}
                    },
                    Ok(None) => break,
                    Err(e) => {
                        let _ = tx.send(Err(e.into()));
                        break;
                    }
                }
            }

            let mut lock = messages.lock().await;
            lock.add_assistant(vec![full_text.into()], tool_calls);
            for id in allocated_tool_ids {
                lock.push_str(Some(&id), "");
            }
        })
    }
}

// ============================================================================
// CONVERSIONS
// ============================================================================

impl TryFrom<Options> for Completions {
    type Error = DynError;

    fn try_from(ops: Options) -> Result<Self> {
        let mut this = Self::new(
            ops.kind,
            if let Some(v) = ops.env_var.as_ref() {
                std::env::var(v).unwrap_or_default()
            } else {
                String::new()
            },
            ops.model,
        )
        .max_tokens(ops.max_tokens.unwrap_or(8096))
        .temperature(ops.temperature.unwrap_or(0.6));

        if let Some(host) = ops.server.as_ref() {
            this = this.host(host.to_owned());
        }
        if let Some(proxy) = ops.proxy.as_ref() {
            this = this.proxy(Proxy::all(proxy.to_owned())?);
        }

        Ok(this)
    }
}
