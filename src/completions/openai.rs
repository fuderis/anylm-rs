use super::*;
use crate::api::*;

use atoman::Receiver;
use futures::StreamExt;
use reqwest::header;
use std::collections::HashMap;

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
            data_obj.insert("response_format".to_string(), schema.to_openai_format()?);
        }

        if !self.tools.is_empty() {
            let mut tools_json = Vec::new();
            for tool in &self.tools {
                tools_json.push(tool.to_openai_format()?);
            }
            data_obj.insert("tools".to_string(), JsonValue::Array(tools_json));
        }

        let client = self.build_client()?;
        let response = client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::ACCEPT, "text/event-stream")
            .header(
                header::AUTHORIZATION,
                format!(
                    "Bearer {}",
                    if let Some(key) = self.api_key.as_ref() {
                        key
                    } else {
                        ""
                    }
                ),
            )
            .json(&data_obj)
            .send()
            .await?;

        let bytes_stream = response.bytes_stream().map(|r| r.map_err(Into::into));
        let reader = pearce::stream_reader::<ResponseChunk>(bytes_stream);
        let (tx, rx) = mpsc::unbounded_channel();
        let handle = Self::spawn_reader(reader, tx, messages);

        Ok(Stream { rx, handle })
    }

    fn spawn_reader(
        mut reader: Receiver<ResponseChunk>,
        tx: mpsc::UnboundedSender<Result<Chunk>>,
        messages: Arc<Mutex<Messages>>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut full_text = String::new();
            let mut tool_calls = vec![];
            let mut allocated_tool_ids = vec![];
            let mut tool_buffers = HashMap::<usize, PartialToolCall>::new();

            loop {
                // extract the next chunk or exit immediately if rx is blocked by the client.
                let res = tokio::select! {
                    _ = tx.closed() => {
                        // client disconnected — stopping reading the stream.
                        return;
                    }
                    res = reader.recv() => res,
                };

                match res {
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
}
