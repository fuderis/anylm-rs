use anylm::{Chunk, Completions, Schema, Tool, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = std::env::var("CEREBRAS_API_KEY")?;

    /// The weather tool data
    #[allow(dead_code)]
    #[derive(Debug, serde::Deserialize)]
    struct LocationData {
        location: String,
    }

    // send request:
    let mut response = Completions::cerebras(api_key, "llama3.1-8b")
        .proxy(Proxy::all("socks5://127.0.0.1:1080")?)
        .user_message(vec!["What's the weather like in London?".into()])
        .tool(Tool::new(
            "weather",
            "Search weather by location",
            Schema::object("Location data")
                .required_property("location", Schema::string("The location")),
        ))
        .send()
        .await?;

    // read response stream:
    let mut tool_calls = vec![];
    while let Some(chunk) = response.next().await {
        match chunk? {
            Chunk::Text(text) => {
                eprint!("{text}");
            }
            Chunk::Tool(name, json_str) => {
                tool_calls.push((name, json_str));
            }
        }
    }
    println!();

    // handle tool calls:
    for (name, json_str) in tool_calls {
        match name.as_ref() {
            "weather" => {
                let location: LocationData = serde_json::from_str(&json_str)?;
                println!("{location:#?}");
            }
            _ => {}
        }
    }

    Ok(())
}
