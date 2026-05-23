use super::*;
use crate::{AiOptions, ToolCall, ToolCallFunction, chunk::*, prelude::*};
use atoman::{Stream, StreamExt, StreamReader};
use reqwest::{Client, Proxy, header};
use std::{sync::Arc, time::Duration};
use tokio::sync::{Mutex, mpsc};

/// The completions response stream reader
#[derive(Debug)]
pub struct AiStream {
    rx: mpsc::UnboundedReceiver<Result<AiChunk>>,
    handle: tokio::task::JoinHandle<()>,
}

impl AiStream {
    /// Read a next completions response chunk
    pub async fn next(&mut self) -> Option<Result<AiChunk>> {
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
pub enum AiChunk {
    Text(String),
    Tool(ToolCall),
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
    pub async fn send(&mut self, messages: Arc<Mutex<Messages>>) -> Result<AiStream> {
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

        // validate context:
        if self.max_tokens > 0 {
            // check tokens count:
            if self.tokens_count > self.max_tokens as usize {
                return Err(Error::ContextOverflowing.into());
            }

            // check last message:
            let lock = messages.lock().await;
            if let Some(msg) = lock.messages.last()
                && !msg.role.is_user()
            {
                return Err(Error::BadRequest.into());
            }
        }

        // serialize & clean data:
        let mut data = json::to_value(&self)?;
        let data_obj = data.as_object_mut().unwrap();
        data_obj.remove("tokens_count");
        data_obj.insert(str!("messages"), json::to_value(&*messages.lock().await)?);
        if let Some(messages) = data_obj.get_mut("messages").and_then(|v| v.as_array_mut()) {
            for msg in messages {
                if let Some(msg_obj) = msg.as_object_mut() {
                    msg_obj.remove("tokens_count");
                    msg_obj.remove("timestamp");
                }
            }
        }
        data_obj.insert(str!("stream"), JsonValue::Bool(true));

        // prepare JSON-schema:
        if let Some(schema) = self.schema.take() {
            data_obj.remove("schema");

            if self.api_kind.is_openai() {
                data_obj.insert(str!("response_format"), schema.to_openai_format()?);
            } else if self.api_kind.is_google() {
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
            } else {
                data_obj.insert(str!("output_config"), schema.to_anthropic_format()?);
            }
        }

        // prepare tools schemes:
        if !self.tools.is_empty() {
            let mut tools_json = Vec::new();

            for tool in &self.tools {
                let formatted_tool = if self.api_kind.is_openai() {
                    tool.to_openai_format()
                } else if self.api_kind.is_google() {
                    tool.to_google_format()
                } else {
                    tool.to_anthropic_format()
                }?;

                if self.api_kind.is_google() {
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
                } else {
                    tools_json.push(formatted_tool);
                }
            }

            data_obj.insert("tools".to_string(), JsonValue::Array(tools_json));
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
            .json(&data_obj);

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
            let messages = data_obj.remove("messages").unwrap_or(json!([]));
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
            data_obj.insert(str!("contents"), json!(contents));
            data_obj.remove("model");
        }

        // send & spawn reader:
        let response = request.send().await?;
        let bytes_stream = response.bytes_stream().map(|r| r.map_err(Into::into));
        let reader = Stream::read::<ResponseChunk>(bytes_stream);

        let (tx, rx) = mpsc::unbounded_channel::<Result<AiChunk>>();
        let handle = Self::spawn_reader(reader, tx, messages);

        Ok(AiStream { rx, handle })
    }

    /// Spawns a background task to process the incoming SSE stream
    fn spawn_reader(
        mut reader: StreamReader<ResponseChunk>,
        tx: mpsc::UnboundedSender<Result<AiChunk>>,
        messages: Arc<Mutex<Messages>>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut full_text = str!();
            let mut tool_calls = vec![];

            // (id, name, arguments)
            let mut tool_buffers = HashMap::<usize, (String, String, String)>::new();

            loop {
                // if client is disconnected:
                if tx.is_closed() {
                    return;
                }

                // read next chunk:
                match reader.read().await {
                    Ok(Some(chunk)) => {
                        let mut text_output = String::new();

                        match chunk {
                            // OpenAI standart:
                            ResponseChunk::OpenAi(OpenAIChunk { choices }) => {
                                for choice in choices {
                                    if let Some(content) = choice.delta.content {
                                        text_output.push_str(&content);
                                    }
                                    if let Some(tool_calls) = choice.delta.tool_calls {
                                        for tc in tool_calls {
                                            if let Some(idx) = tc.index {
                                                let entry = tool_buffers.entry(idx).or_default();

                                                if let Some(id) = tc.id {
                                                    entry.0 = id;
                                                }

                                                if let Some(fn_delta) = tc.function {
                                                    if let Some(name) = fn_delta.name {
                                                        entry.1 = name;
                                                    }
                                                    if let Some(args) = fn_delta.arguments {
                                                        entry.2.push_str(&args);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Anthropic standart:
                            ResponseChunk::Anthropic(anth) => {
                                let idx = anth.index.unwrap_or(0);

                                if let Some(block) = anth.content_block {
                                    if block.kind == "tool_use" {
                                        let entry = tool_buffers.entry(idx).or_default();
                                        entry.1 = block.name;
                                        if let Some(id) = block.id {
                                            entry.0 = id;
                                        }
                                    }
                                }

                                if let Some(delta) = anth.delta {
                                    if let Some(t) = delta.text {
                                        text_output.push_str(&t);
                                    }
                                    if let Some(pj) = delta.partial_json {
                                        tool_buffers.entry(idx).or_default().2.push_str(&pj);
                                    }
                                }
                            }

                            // Google standart:
                            ResponseChunk::Google(google) => {
                                for cand in google.candidates {
                                    if let Some(content) = cand.content {
                                        for part in content.parts {
                                            match part {
                                                GeminiPart::Text { text } => {
                                                    text_output.push_str(&text)
                                                }
                                                GeminiPart::FunctionCall { function_call } => {
                                                    let tool_name = function_call.name;
                                                    let final_id = function_call.id;

                                                    let tool = ToolCall {
                                                        id: final_id.unwrap_or_default(),
                                                        kind: str!("function"),
                                                        func: ToolCallFunction {
                                                            name: tool_name,
                                                            json_str: function_call
                                                                .args
                                                                .to_string(),
                                                        },
                                                    };
                                                    tool_calls.push(tool.clone());

                                                    if tx.send(Ok(AiChunk::Tool(tool))).is_err() {
                                                        return;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Error
                            ResponseChunk::Error(err) => {
                                let _ = tx
                                    .send(Err(
                                        Error::ResponseError(ResponseError { error: err }).into()
                                    ));
                                return;
                            }
                        }

                        if !text_output.is_empty() {
                            full_text.push_str(&text_output);
                            if tx.send(Ok(AiChunk::Text(text_output))).is_err() {
                                break;
                            }
                        }

                        // check buffers for tool calls:
                        tool_buffers.retain(|_, (id, name, args)| {
                            if json::from_str::<JsonValue>(args).is_ok() {
                                let tool = ToolCall {
                                    id: id.clone(),
                                    kind: str!("function"),
                                    func: ToolCallFunction {
                                        name: name.clone(),
                                        json_str: args.clone(),
                                    },
                                };
                                tool_calls.push(tool.clone());

                                if tx.send(Ok(AiChunk::Tool(tool))).is_err() {
                                    return false;
                                }
                                false // sent -> delete from buffer
                            } else {
                                true // not complete yet -> leave it in buffer
                            }
                        });
                    }

                    Ok(None) => break,

                    Err(e) => {
                        let _ = tx.send(Err(e.into()));
                        break;
                    }
                }
            }

            messages
                .lock()
                .await
                .add_assistant(vec![full_text.into()], tool_calls);
        })
    }

    /* /// Generates an unique id for tool call
    fn gen_tool_id(tool_name: &str) -> String {
        use std::hash::{Hash, Hasher};

        // get system time:
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        // hashing nanos and name:
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        tool_name.hash(&mut hasher);
        nanos.hash(&mut hasher);
        let hash_result = hasher.finish();

        // formatting:
        str!("call_{hash_result:016x}")
    } */
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
