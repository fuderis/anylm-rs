use anylm::{AiChunk, Completions, Messages, ToolCall};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

/// The weather tool data
#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct LocationData {
    location: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // create messages:
    let messages = Messages::new()
        .system(vec!["You are a helpful assistant.".into()])
        .user(vec!["What's the weather like in London?".into()])
        .wrap();

    // send request:
    let mut response = Completions::lmstudio("", "qwen/qwen2.5-vl-7b")
        .tool(serde_json::from_str(
            r#"{
                "name": "weather",
                "description": "Search weather by location",
                "properties": {
                    "location": { "type": "string", "optional": false }
                }
            }"#,
        )?)
        .send(messages.clone())
        .await?;

    let mut tool_calls = vec![];

    // read response chunks:
    while let Some(chunk) = response.next().await {
        match chunk? {
            AiChunk::Text(text_part) => {
                eprint!("{text_part}");
            }
            AiChunk::Tool(tool_call) => {
                tool_calls.push(tool_call);
            }
        }
    }

    // handle tool calls:
    for ToolCall { id, func, .. } in tool_calls {
        match func.name.as_ref() {
            "weather" => {
                let results = get_weather(func.parse_args()?);
                println!("{results}");

                messages.lock().await.add_tool(id, vec![results.into()])
            }
            _ => {}
        }
    }

    // dbg!(&messages);

    Ok(())
}

fn get_weather(_loc: LocationData) -> String {
    format!("74°F, Cloudy\n• Precipitation: 0%\n• Humidity: 64%\n• Wind: 3.11 mph")
}
