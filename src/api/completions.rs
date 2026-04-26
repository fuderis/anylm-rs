use super::*;
use crate::{AiOptions, prelude::*};
use atoman::{Stream, StreamExt};
use reqwest::{Client, Proxy, header};
use std::time::Duration;
use tokio::sync::mpsc;

/// The completions response stream reader
#[derive(Debug)]
pub struct AiStream {
    rx: mpsc::UnboundedReceiver<Result<AiChunk>>,
}

impl AiStream {
    /// Read a next completions response chunk
    pub async fn next(&mut self) -> Option<Result<AiChunk>> {
        self.rx.recv().await
    }
}

/// The completions response chunk
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AiChunk {
    Text { text: String },
    Tool { name: String, json_str: String },
}

/// The LM API chat completions request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Completions {
    /// The API standart
    #[serde(skip)]
    pub api_kind: ApiKind,
    /// The API version
    #[serde(skip)]
    pub api_version: Option<String>,
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
    /// The request messages
    pub messages: Vec<Message>,
    /// The maximum tokens count
    pub max_tokens: i32,
    /// The AI generation temperature
    pub temperature: f32,
    /// The response schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<Schema>,
    /// The tool calls
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<Tool>,
    /// The summary tokens count
    pub tokens_count: usize,
}

impl Completions {
    /// Creates a new LM chat completions request
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
            messages: Vec::new(),
            max_tokens: if kind.is_anthropic() { 8096 } else { -1 },
            temperature: 0.7,
            tokens_count: 0,
            schema: None,
            tools: Vec::new(),
            api_kind: kind,
        }
    }

    /// Sets the API version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.api_version = Some(version.into());
        self
    }
    /// Sets the API version
    pub fn set_version(&mut self, version: impl Into<String>) {
        self.api_version = Some(version.into());
    }

    /// Creates a new OpenAI (ChatGPT) request
    pub fn openai(key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new(ApiKind::OpenAI, key, model)
    }

    /// Creates a new Anthropic (Claude) request
    pub fn anthropic(key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new(ApiKind::Anthropic, key, model)
    }

    /// Creates a new LM Studio request
    pub fn lmstudio(key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new(ApiKind::LmStudio, key, model)
    }

    /// Creates a new ChatGPT request
    pub fn chatgpt(key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new(ApiKind::ChatGpt, key, model)
    }

    /// Creates a new Cerebras AI request
    pub fn cerebras(key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new(ApiKind::Cerebras, key, model)
    }

    /// Creates a new Claude AI request
    pub fn claude(key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new(ApiKind::Claude, key, model)
    }

    /// Creates a new OpenRouter AI request
    pub fn openrouter(key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new(ApiKind::OpenRouter, key, model)
    }

    /// Creates a new Perplexity AI request
    pub fn perplexity(key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new(ApiKind::Perplexity, key, model)
    }

    /// Creates a new Google AI request
    pub fn google(key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new(ApiKind::Google, key, model)
    }

    /// Creates a new Google Gemini AI request
    pub fn gemini(key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new(ApiKind::Gemini, key, model)
    }

    /// Sets the LM API authorization key
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.api_key = key.into();
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

    /// Adds a messages to request
    pub fn messages(mut self, msgs: Vec<Message>) -> Self {
        for msg in &msgs {
            self.tokens_count += msg.tokens_count;
        }
        self.messages.extend(msgs);
        self
    }
    /// Adds a messages to request
    pub fn add_messages(&mut self, msgs: Vec<Message>) {
        for msg in &msgs {
            self.tokens_count += msg.tokens_count;
        }
        self.messages.extend(msgs);
    }

    /// Adds a message to request
    pub fn message(mut self, role: Role, content: Vec<Content>) -> Self {
        let msg = Message::new(role, content);
        self.tokens_count += msg.tokens_count;
        self.messages.push(msg);
        self
    }
    /// Adds a system message to request
    pub fn system_message(self, content: Vec<Content>) -> Self {
        self.message(Role::System, content)
    }
    /// Adds a user message to request
    pub fn user_message(self, content: Vec<Content>) -> Self {
        self.message(Role::User, content)
    }
    /// Adds a assistant message to request
    pub fn assistant_message(self, content: Vec<Content>) -> Self {
        self.message(Role::Assistant, content)
    }

    /// Adds a message to request
    pub fn add_message(&mut self, role: Role, content: Vec<Content>) {
        let msg = Message::new(role, content);
        self.tokens_count += msg.tokens_count;
        self.messages.push(msg);
    }
    /// Adds a system message to request
    pub fn add_system_message(&mut self, content: Vec<Content>) {
        self.add_message(Role::System, content)
    }
    /// Adds a user message to request
    pub fn add_user_message(&mut self, content: Vec<Content>) {
        self.add_message(Role::User, content)
    }
    /// Adds a assistant message to request
    pub fn add_assistant_message(&mut self, content: Vec<Content>) {
        self.add_message(Role::Assistant, content)
    }

    /// Sets the AI generation temperature
    pub fn set_temperature(&mut self, temperature: f32) {
        self.temperature = temperature;
    }
    /// Sets the AI generation temperature
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.set_temperature(temperature);
        self
    }

    /// Sets the maximum context tokens count
    pub fn max_tokens(mut self, count: i32) -> Self {
        self.max_tokens = count;
        self
    }

    /// Sets the structured response schema
    pub fn schema(mut self, schema: Schema) -> Self {
        self.schema.replace(schema);
        self
    }

    /// Adds the tool calls
    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools.extend(tools);
        self
    }

    /// Adds the tool call
    pub fn tool(mut self, tool: Tool) -> Self {
        self.tools.push(tool);
        self
    }

    /// Sends the request to LM server
    pub async fn send(&mut self) -> Result<AiStream> {
        use crate::chunk::*;

        // generate URL:
        let url = if let Some(host) = &self.host {
            str!(
                "{host}{}{}",
                if host.ends_with("/") { "" } else { "/" },
                self.api_kind.completions_path(&self.model)
            )
        } else {
            str!(
                "{}/{}",
                self.api_kind.host(),
                self.api_kind.completions_path(&self.model)
            )
        };

        // context management:
        if self.max_tokens > 0 {
            let mut idx = 0;
            let mut max_idx = self.messages.len() - 1;
            let max_count = self.max_tokens as usize;

            while self.tokens_count > max_count && idx < max_idx {
                let msg = &self.messages[idx];
                if msg.role.is_system() {
                    idx += 1;
                    continue;
                }
                self.tokens_count -= msg.tokens_count;
                self.messages.remove(idx);
                max_idx -= 1;
            }

            if let Some(msg) = self.messages.last()
                && msg.role.is_assistant()
            {
                return Err(Error::IncorrectContext.into());
            }
        }

        // serialize & clean data:
        let mut data = json::to_value(&self).map_err(Error::from)?;
        let data = data.as_object_mut().unwrap();

        data.remove("tokens_count");
        if let Some(messages) = data.get_mut("messages").and_then(|v| v.as_array_mut()) {
            for msg in messages {
                if let Some(msg_obj) = msg.as_object_mut() {
                    msg_obj.remove("tokens_count");
                    msg_obj.remove("timestamp");
                }
            }
        }
        data.insert(str!("stream"), JsonValue::Bool(true));

        // JSON-schema & tool calls:
        if let Some(mut schema) = self.schema.take() {
            data.remove("schema");
            if self.api_kind.is_openai() {
                data.insert(
                    str!("response_format"),
                    json!({
                        "type": "json_schema",
                        "json_schema": { "name": "response", "schema": schema, "strict": true }
                    }),
                );
            } else if !self.api_kind.is_google() {
                schema.additional_properties = Some(false);
                data.insert(
                    str!("output_config"),
                    json!({
                        "format": { "type": "json_schema", "schema": schema }
                    }),
                );
            }
        }

        if !self.tools.is_empty() {
            let mut tools_json = vec![];
            for tool in &self.tools {
                if self.api_kind.is_openai() {
                    tools_json.push(json!({ "type": "function", "function": tool }));
                } else if !self.api_kind.is_google() {
                    let mut tool_json = json::to_value(tool).unwrap();
                    let tool_obj = tool_json.as_object_mut().unwrap();
                    let params = tool_obj.get_mut("parameters").cloned().unwrap();
                    tool_obj.remove("parameters");
                    tool_obj.insert("input_schema".to_string(), params);
                    tools_json.push(tool_json);
                }
            }
            data.insert(str!("tools"), JsonValue::Array(tools_json));
        }

        // create client & configure proxy:
        let mut client_builder = Client::builder().timeout(self.timeout);
        if let Some(proxy) = self.proxy.take() {
            client_builder = client_builder
                .proxy(proxy)
                .danger_accept_invalid_certs(true);
        }

        // build request & options:
        let mut request = client_builder
            .build()?
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::ACCEPT, "text/event-stream")
            .json(&data);

        // set api key:
        if self.api_kind.is_google() {
            request = request.header("x-goog-api-key", &self.api_key);
        } else if self.api_kind.is_anthropic() {
            request = request.header("x-api-key", &self.api_key);
            request = request.header(
                "anthropic-version",
                self.api_version.take().unwrap_or(str!("2023-06-01")),
            );
        } else {
            request = request.header(header::AUTHORIZATION, str!("Bearer {}", self.api_key));
        }

        if self.api_kind.is_google() {
            let messages = data.remove("messages").unwrap_or(json!([]));
            let contents: Vec<JsonValue> = messages
                .as_array()
                .unwrap()
                .iter()
                .map(|m| {
                    json!({
                        "role": if m["role"] == "assistant" { "model" } else { "user" },
                        "parts": m["content"]
                    })
                })
                .collect();
            data.insert(str!("contents"), json!(contents));
            data.remove("model");
        }

        // send & spawn reader:
        let response = request.send().await?;
        let bytes_stream = response.bytes_stream().map(|r| r.map_err(Into::into));

        let mut reader = Stream::read::<ResponseChunk>(bytes_stream);

        let (tx, rx) = mpsc::unbounded_channel::<Result<AiChunk>>();
        let mut tool_buffers = HashMap::<usize, (String, String)>::new();

        tokio::spawn(async move {
            loop {
                match reader.read().await {
                    Ok(Some(chunk)) => {
                        let mut text_output = String::new();

                        match chunk {
                            ResponseChunk::OpenAi(OpenAIChunk { choices }) => {
                                for choice in choices {
                                    if let Some(content) = choice.delta.content {
                                        text_output.push_str(&content);
                                    }
                                    if let Some(tool_calls) = choice.delta.tool_calls {
                                        for tc in tool_calls {
                                            if let (Some(idx), Some(fn_delta)) =
                                                (tc.index, tc.function)
                                            {
                                                let entry = tool_buffers.entry(idx).or_default();
                                                if let Some(name) = fn_delta.name {
                                                    entry.0 = name;
                                                }
                                                if let Some(args) = fn_delta.arguments {
                                                    entry.1.push_str(&args);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            ResponseChunk::Anthropic(anth) => {
                                if let Some(delta) = anth.delta {
                                    if let Some(t) = delta.text {
                                        text_output.push_str(&t);
                                    }
                                    if let Some(pj) = delta.partial_json {
                                        let idx = anth.index.unwrap_or(0);
                                        tool_buffers.entry(idx).or_default().1.push_str(&pj);
                                    }
                                }
                                if let Some(block) = anth.content_block
                                    && block.kind == "tool_use"
                                {
                                    let idx = anth.index.unwrap_or(0);
                                    tool_buffers.entry(idx).or_default().0 = block.name;
                                }
                            }
                            ResponseChunk::Google(google) => {
                                for cand in google.candidates {
                                    if let Some(content) = cand.content {
                                        for part in content.parts {
                                            match part {
                                                GeminiPart::Text { text } => {
                                                    text_output.push_str(&text)
                                                }
                                                GeminiPart::FunctionCall { function_call } => {
                                                    tx.send(Ok(AiChunk::Tool {
                                                        name: function_call["name"]
                                                            .as_str()
                                                            .unwrap_or("")
                                                            .to_string(),
                                                        json_str: function_call["args"].to_string(),
                                                    }))
                                                    .ok();
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            ResponseChunk::Error(err) => {
                                tx.send(Err(
                                    Error::ResponseError(ResponseError { error: err }).into()
                                ))
                                .ok();
                                return;
                            }
                        }

                        if !text_output.is_empty() {
                            tx.send(Ok(AiChunk::Text { text: text_output })).ok();
                        }

                        tool_buffers.retain(|_, (name, args)| {
                            if json::from_str::<JsonValue>(args).is_ok() {
                                tx.send(Ok(AiChunk::Tool {
                                    name: name.clone(),
                                    json_str: args.clone(),
                                }))
                                .ok();
                                false
                            } else {
                                true
                            }
                        });
                    }
                    Ok(None) => break,
                    Err(e) => {
                        tx.send(Err(e.into())).ok();
                        break;
                    }
                }
            }
        });

        Ok(AiStream { rx })
    }
}

impl TryFrom<AiOptions> for Completions {
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
        )
        .max_tokens(ops.max_tokens.unwrap_or(8096))
        .temperature(ops.temperature.unwrap_or(0.6));

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
