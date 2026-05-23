use anylm::{AiChunk, Completions, Schema, Tool, prelude::*};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = std::env::var("OPENROUTER_API_KEY")?;

    /// The weather tool data
    #[allow(dead_code)]
    #[derive(Debug, serde::Deserialize)]
    struct LocationData {
        location: String,
    }

    // send request:
    let mut response = Completions::openrouter(api_key, "qwen/qwen3-vl-30b-a3b-thinking")
        .proxy(Proxy::all("socks5://127.0.0.1:1080")?)
        .user_message(vec!["What's the weather like in London?".into()])
        .tool(
            Tool::new("weather", "Search weather by location")
                .required_property("location", Schema::string("The location")),
        )
        .send()
        .await?;

    // read response stream:
    let mut tool_calls = vec![];
    while let Some(chunk) = response.next().await {
        match chunk? {
            AiChunk::Text { text } => {
                eprint!("{text}");
            }
            AiChunk::Tool { id, name, json_str } => {
                tool_calls.push((id, name, json_str));
            }
        }
    }
    println!();

    // handle tool calls:
    for (id, name, json_str) in tool_calls {
        match name.as_ref() {
            "weather" => {
                let location: LocationData = serde_json::from_str(&json_str)?;
                println!("Tool call [{id}]: {location:#?}");
            }
            _ => {}
        }
    }

    Ok(())
}
