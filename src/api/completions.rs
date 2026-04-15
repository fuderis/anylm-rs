use super::*;
use crate::{AiOptions, error::ResponseError, prelude::*};
use futures::StreamExt;
use reqwest::{Client, Proxy, header};
use std::time::Duration;
use tokio::sync::mpsc;

/// The completions response stream reader
#[derive(Debug)]
pub struct Stream {
    rx: mpsc::UnboundedReceiver<Result<AiChunk>>,
}

impl Stream {
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
    pub server: Option<String>,
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
            api_key: key.into(),
            proxy: None,
            timeout: Duration::from_secs(30),
            model: model.into(),
            messages: Vec::new(),
            max_tokens: if kind.is_anthropic_standart() {
                8096
            } else {
                -1
            },
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

    /// Sets the max context tokens count
    pub fn set_max_tokens(&mut self, count: i32) {
        self.max_tokens = count;
    }
    /// Sets the max context tokens count
    pub fn max_tokens(mut self, count: i32) -> Self {
        self.set_max_tokens(count);
        self
    }

    /// Sets the structured response schema
    pub fn set_schema(&mut self, schema: Schema) {
        self.schema.replace(schema);
    }
    /// Sets the structured response schema
    pub fn schema(mut self, schema: Schema) -> Self {
        self.set_schema(schema);
        self
    }

    /// Adds the tool calls
    pub fn set_tools(&mut self, tools: Vec<Tool>) {
        self.tools.extend(tools);
    }
    /// Adds the tool calls
    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
        self.set_tools(tools);
        self
    }

    /// Adds the tool call
    pub fn set_tool(&mut self, tool: Tool) {
        self.tools.push(tool);
    }
    /// Adds the tool call
    pub fn tool(mut self, tool: Tool) -> Self {
        self.set_tool(tool);
        self
    }

    /// Sends the request to LM server
    pub async fn send(&mut self) -> Result<Stream> {
        let is_openai_standart = self.api_kind.is_openai_standart();

        // generate URL:
        let url = if let Some(url) = &self.server {
            self.api_kind.custom_completions_url(url)
        } else {
            self.api_kind.completions_url()
        };

        // remove extra context:
        if self.max_tokens > 0 {
            let mut idx = 0;
            let mut max_idx = self.messages.len() - 1;
            let max_count = self.max_tokens as usize;

            // removing non-system old messages to free context size:
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

            // check last message role for user query:
            if let Some(msg) = self.messages.last()
                && msg.role.is_assistant()
            {
                return Err(Error::IncorrectContext.into());
            }
        }

        // serialize request data:
        let mut data = json::to_value(&self).map_err(Error::from)?;
        let data = data.as_object_mut().unwrap();

        // remove field 'tokens_count':
        data.remove("tokens_count");
        if let Some(messages) = data.get_mut("messages").and_then(|v| v.as_array_mut()) {
            for msg in messages {
                if let Some(msg_obj) = msg.as_object_mut() {
                    msg_obj.remove("tokens_count");
                }
            }
        }
        data.insert(str!("stream"), JsonValue::Bool(true));

        // set output JSON-schema:
        if let Some(mut schema) = self.schema.take() {
            data.remove("schema");

            // open ai:
            if is_openai_standart {
                data.insert(
                    str!("response_format"),
                    json!({
                        "type": "json_schema",
                        "json_schema": {
                            "name": "response",
                            "schema": schema,
                            "strict": true
                        }
                    }),
                );
            }
            // anthropic:
            else {
                schema.additional_properties = Some(false);

                data.insert(
                    str!("output_config"),
                    json!({
                        "format": {
                            "type": "json_schema",
                            "schema": schema,
                        }
                    }),
                );
            }
        }

        // set tool calls:
        if !self.tools.is_empty() {
            let mut tools_json = vec![];

            for tool in &self.tools {
                // open ai:
                if is_openai_standart {
                    tools_json.push(json!({
                        "type": "function",
                        "function": tool
                    }));
                }
                // anthropic:
                else {
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

        // create client:
        let mut client = Client::builder().timeout(self.timeout);

        // set proxy:
        if let Some(proxy) = self.proxy.take() {
            client = client.proxy(proxy);
            client = client.danger_accept_invalid_certs(true); // VPN SSL
        }

        // send request:
        let mut request = client
            .build()?
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::ACCEPT, "text/event-stream")
            .json(&data);

        // open ai:
        if is_openai_standart {
            request = request.header(header::AUTHORIZATION, fmt!("Bearer {}", self.api_key));
        }
        // anthropic:
        else {
            request = request.header("x-api-key", &self.api_key);
            request = request.header(
                "anthropic-version",
                self.api_version.take().unwrap_or(str!("2023-06-01")),
            );
        }

        // send request:
        let response = request.send().await?;
        let (tx, rx) = mpsc::unbounded_channel::<Result<AiChunk>>();

        // spawn stream reader:
        tokio::spawn(async move {
            let mut buffer = String::new();
            let mut stream = response.bytes_stream();
            let mut tool_buffers = HashMap::<usize, (String, String)>::new();

            // read chunks:
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        let bytes_stringify = String::from_utf8_lossy(&bytes);

                        // check for an error:
                        if let Some(e) = ResponseError::from_str(&bytes_stringify) {
                            tx.send(Err(Error::ResponseError(e).into())).ok();
                            return;
                        }

                        // else parse buffer:
                        buffer.push_str(&bytes_stringify);
                        while let Some(pos) = buffer.find("\n\n") {
                            let event = buffer.drain(..=pos).collect::<String>();
                            if let Some(data_line) =
                                event.lines().find(|line| line.starts_with("data: "))
                            {
                                let json_data = &data_line[6..]; // cut "data: "
                                if json_data == "[DONE]" {
                                    break;
                                }
                                let mut text = String::new();

                                // OpenAI API standart:
                                if is_openai_standart {
                                    #[derive(Debug, Deserialize)]
                                    struct OpenAIAiChunk {
                                        choices: Vec<OpenAIChoice>,
                                    }
                                    #[allow(dead_code)]
                                    #[derive(Debug, Deserialize)]
                                    struct OpenAIChoice {
                                        delta: OpenAIDelta,
                                        #[serde(default)]
                                        finish_reason: Option<String>,
                                    }
                                    #[derive(Debug, Deserialize)]
                                    struct OpenAIDelta {
                                        #[serde(default)]
                                        content: Option<String>,
                                        #[serde(default)]
                                        tool_calls: Option<Vec<ToolCallDelta>>,
                                    }

                                    #[derive(Debug, Deserialize, Default)]
                                    struct ToolCallDelta {
                                        #[serde(rename = "type")]
                                        _kind: Option<String>,
                                        #[serde(default)]
                                        index: Option<usize>,
                                        #[serde(default)]
                                        function: Option<FunctionDelta>,
                                    }
                                    #[allow(dead_code)]
                                    #[derive(Debug, Deserialize, Default)]
                                    struct FunctionDelta {
                                        #[serde(default)]
                                        name: Option<String>,
                                        #[serde(default)]
                                        arguments: Option<String>,
                                    }

                                    // parse chunk:
                                    if let Ok(OpenAIAiChunk { choices }) =
                                        json::from_str::<OpenAIAiChunk>(json_data)
                                            .map_err(Error::from)
                                    {
                                        for choice in choices {
                                            // text:
                                            if let Some(content) = choice.delta.content {
                                                text.push_str(&content);
                                            }

                                            // tool calls:
                                            if let Some(tool_calls) = choice.delta.tool_calls {
                                                for tool_call in tool_calls {
                                                    if let (Some(index), Some(function)) =
                                                        (tool_call.index, tool_call.function)
                                                    {
                                                        let tool =
                                                            tool_buffers.entry(index).or_default();

                                                        if let Some(name) = function.name {
                                                            tool.0 = name;
                                                        }
                                                        if let Some(arguments) = &function.arguments
                                                        {
                                                            tool.1.push_str(arguments);
                                                        }
                                                    }
                                                }
                                            }

                                            // send tool calls to receiver:
                                            tool_buffers.retain(|_index, (name, args)| {
                                                if json::from_str::<JsonValue>(&args).is_ok() {
                                                    tx.send(Ok(AiChunk::Tool {
                                                        name: name.clone(),
                                                        json_str: args.clone(),
                                                    }))
                                                    .ok();
                                                    false
                                                } else {
                                                    true
                                                }
                                            })
                                        }
                                    }
                                }
                                // Anthropic API standart:
                                else {
                                    #[derive(Debug, Deserialize)]
                                    struct AnthropicEvent {
                                        #[serde(rename = "type")]
                                        kind: String,
                                        #[serde(default)]
                                        index: Option<usize>,
                                        delta: Option<AnthropicDelta>,
                                        content_block: Option<ContentBlock>,
                                    }

                                    #[derive(Debug, Deserialize)]
                                    struct AnthropicDelta {
                                        #[serde(rename = "type")]
                                        kind: String,
                                        text: Option<String>,
                                        #[serde(rename = "partial_json")]
                                        partial_json: Option<String>,
                                    }

                                    #[derive(Debug, Deserialize)]
                                    struct ContentBlock {
                                        #[serde(rename = "type")]
                                        kind: String,
                                        name: String,
                                    }

                                    // parsing event:
                                    if let Ok(event) = json::from_str::<AnthropicEvent>(json_data) {
                                        match event.kind.as_str() {
                                            "content_block_delta" => {
                                                let index = event.index.unwrap_or(0);
                                                let tool = tool_buffers.entry(index).or_default();

                                                if let Some(delta) = event.delta {
                                                    match delta.kind.as_str() {
                                                        "text_delta" => {
                                                            if let Some(content) = delta.text {
                                                                text.push_str(&content);
                                                            }
                                                        }
                                                        "input_json_delta" => {
                                                            if let Some(partial_json) =
                                                                delta.partial_json
                                                            {
                                                                tool.1.push_str(&partial_json);
                                                            }
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                            }
                                            "content_block_start" => {
                                                if let Some(block) = event.content_block
                                                    && let Some(index) = event.index
                                                    && block.kind == "tool_use"
                                                    && !block.name.is_empty()
                                                {
                                                    let tool =
                                                        tool_buffers.entry(index).or_default();

                                                    tool.0 = block.name;
                                                }
                                            }
                                            _ => {}
                                        }

                                        // send tool calls to receiver:
                                        tool_buffers.retain(|_index, (name, args)| {
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
                                }

                                // send text chunk to receiver:
                                if !text.is_empty() {
                                    tx.send(Ok(AiChunk::Text { text })).ok();
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tx.send(Err(e.into())).ok();
                        break;
                    }
                }
            }
        });

        Ok(Stream { rx })
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
            this.set_server(host.to_owned());
        }
        // set proxy options:
        if let Some(proxy) = ops.proxy.as_ref() {
            this.set_proxy(Proxy::all(proxy.to_owned())?);
        }

        Ok(this)
    }
}
