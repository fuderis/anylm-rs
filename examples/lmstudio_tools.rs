use anylm::{
    api::{Messages, Schema, Tool, ToolCall},
    completions::{Chunk, Completions},
};

/// The weather tool data
#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct LocationData {
    location: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    // create messages:
    let messages = Messages::new()
        .system(vec!["You are a helpful assistant.".into()])
        .user(vec!["What's the weather like in London?".into()])
        .wrap();

    // send request:
    let mut response = Completions::openai()
        .base_url("http://127.0.0.1:1234")
        .model("qwen/qwen3-vl-4b")
        .tool(
            Tool::new("weather", "Search weather by location")
                .required_property("location", Schema::string("The location")),
        )
        .send(messages.clone())
        .await?;

    let mut tool_calls = vec![];

    // read response chunks:
    while let Some(chunk) = response.next().await {
        match chunk? {
            Chunk::Text(text_part) => {
                eprint!("{text_part}");
            }
            Chunk::Tool(tool_call) => {
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
