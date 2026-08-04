use super::*;
use crate::api::*;

use atoman::{Stream as StreamHelper, StreamExt, StreamReader};
use reqwest::header;

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
    pub async fn send(&mut self, messages: Arc<Mutex<Messages>>) -> Result<Stream> {
        let url = self.build_url();

        let mut data = json::to_value(&self.0)?;
        let data_obj = data.as_object_mut().unwrap();
        data_obj.remove("tokens_count");
        data_obj.remove("model");
        data_obj.insert("stream".to_string(), JsonValue::Bool(true));

        let raw_messages = messages.lock().await.to_json()?;
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
            data_obj.insert("contents".to_string(), json!(contents));
        }

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
                data_obj.insert("generationConfig".to_string(), google_config);
            }
        }

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
            data_obj.insert("tools".to_string(), JsonValue::Array(tools_json));
        }

        let client = self.build_client()?;
        let response = client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::ACCEPT, "text/event-stream")
            .header(
                "x-goog-api-key",
                if let Some(key) = self.api_key.as_ref() {
                    key
                } else {
                    ""
                },
            )
            .json(&data_obj)
            .send()
            .await?;

        let bytes_stream = response.bytes_stream().map(|r| r.map_err(Into::into));
        let reader = StreamHelper::read::<ResponseChunk>(bytes_stream);
        let (tx, rx) = mpsc::unbounded_channel();
        let handle = Self::spawn_reader(reader, tx, messages);

        Ok(Stream { rx, handle })
    }

    fn spawn_reader(
        mut reader: StreamReader<ResponseChunk>,
        tx: mpsc::UnboundedSender<Result<Chunk>>,
        messages: Arc<Mutex<Messages>>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut full_text = String::new();
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
                                                let final_id =
                                                    function_call.id.unwrap_or_else(|| {
                                                        format!("call_{}", tool_name)
                                                    });

                                                allocated_tool_ids.push(final_id.clone());

                                                let tool = ToolCall {
                                                    id: final_id,
                                                    kind: "function".to_string(),
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
