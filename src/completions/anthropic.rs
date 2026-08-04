use super::*;
use crate::api::*;

use atoman::{Stream as StreamHelper, StreamExt, StreamReader};
use reqwest::header;
use std::collections::HashMap;

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
    pub async fn send(&mut self, messages: Arc<Mutex<Messages>>) -> Result<Stream> {
        let url = self.build_url();

        let mut data = json::to_value(&self.0)?;
        let data_obj = data.as_object_mut().unwrap();
        data_obj.remove("tokens_count");

        let mut msgs_val = messages.lock().await.to_json()?;
        if let Some(arr) = msgs_val.as_array_mut() {
            for msg in arr {
                if let Some(obj) = msg.as_object_mut() {
                    obj.remove("tokens_count");
                    obj.remove("timestamp");
                }
            }
        }
        data_obj.insert("messages".to_string(), msgs_val);
        data_obj.insert("stream".to_string(), JsonValue::Bool(true));

        if let Some(schema) = self.schema.take() {
            data_obj.remove("schema");
            data_obj.insert("output_config".to_string(), schema.to_anthropic_format()?);
        }

        if !self.tools.is_empty() {
            let mut tools_json = Vec::new();
            for tool in &self.tools {
                tools_json.push(tool.to_anthropic_format()?);
            }
            data_obj.insert("tools".to_string(), JsonValue::Array(tools_json));
        }

        let api_version = self
            .api_version
            .take()
            .unwrap_or_else(|| "2023-06-01".to_string());

        let client = self.build_client()?;
        let response = client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::ACCEPT, "text/event-stream")
            .header(
                "x-api-key",
                if let Some(key) = self.api_key.as_ref() {
                    key
                } else {
                    ""
                },
            )
            .header("anthropic-version", api_version)
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
            format!("call_{}", buf.name)
        } else {
            buf.id
        };

        allocated_tool_ids.push(final_id.clone());

        let tool = ToolCall {
            id: final_id,
            kind: "function".to_string(),
            func: ToolCallFunction {
                name: buf.name,
                json_str: buf.args_buf,
            },
        };

        tool_calls.push(tool.clone());
        let _ = tx.send(Ok(Chunk::Tool(tool)));
    }
}
