use anylm::{AiChunk, Completions, Messages, Proxy, Schema, Tool, ToolCall};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

/// The weather tool data
#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct LocationData {
    location: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")?;

    // prepare messages:
    let messages = Messages::new()
        .user(vec!["What's the weather like in London?".into()])
        .wrap();

    // send request:
    let mut response = Completions::anthropic(api_key, "claude-opus-4-6")
        .proxy(Proxy::all("socks5://127.0.0.1:1080")?)
        .tool(
            Tool::new("weather", "Search weather by location")
                .required_property("location", Schema::string("The location")),
        )
        .send(messages.clone())
        .await?;

    let mut tool_calls = vec![];

    // read response stream:
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

    Ok(())
}

fn get_weather(_loc: LocationData) -> String {
    format!("74°F, Cloudy\n• Precipitation: 0%\n• Humidity: 64%\n• Wind: 3.11 mph")
}
