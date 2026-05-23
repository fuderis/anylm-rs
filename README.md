[![github]](https://github.com/fuderis/anylm-rs)&ensp;
[![crates-io]](https://crates.io/crates/anylm)&ensp;
[![docs-rs]](https://docs.rs/anylm)

[github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs

# AnyLM: Universal API for Every AI

**Sick of juggling separate APIs for each AI model—wrestling with their quirky syntax and endless docs?**<br>

I was too. That's why I built `AnyLM`: learn one intuitive API once, then unleash it across any service—LLMs, embeddings, vision models, you name it. Seamless, powerful, done.

## Supported:

* **Standarts**: Supported `OpenAI` and `Anthropic` API standarts (what 90% of AI uses).
* **Services**: `LM Studio`, `ChatGPT`, `Cerebras`, `OpenRouter`, `Perplexity`, `Claude` and `Voyage`.
* **Stream Response**: Allows you to read the LM response in parts without waiting for the full completion.
* **Context Control**: Automatic trimming of the dialog context when exceeding the token limits.
* **Image View**: Image analysis support with reading from files and directly via `base64 url`.
* **Structured Output**: Structured AI-response in JSON format.
* **Tool Calls**: Calling handlers with arguments for smart AI agents.
* **Embeddings**: Text embeddings support for fast text analysis.
* **Proxy Support**: Support for using proxy/vpn request tunneling.
* **Is something missing?**: Write to me and I will add it too. (Contacts: [E-Mail](mailto:synapdrake@ya.ru)).

## Examples:

### Cerebras:
```rust
use anylm::{AiChunk, Completions, Messages, Proxy};

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = std::env::var("CEREBRAS_API_KEY")?;

    // prepare messages:
    let messages = Messages::new()
        .user(vec!["Hello, how are you doing?".into()])
        .wrap();

    // send request:
    let mut response = Completions::cerebras(api_key, "llama3.1-8b")
        .proxy(Proxy::all("socks5://127.0.0.1:1080")?)
        .send(messages)
        .await?;

    // read response stream:
    while let Some(chunk) = response.next().await {
        if let AiChunk::Text(text) = chunk? {
            eprint!("{text}");
        }
    }
    println!();

    Ok(())
}
```

### Claude:
```rust
use anylm::{AiChunk, Completions, Messages, Proxy};

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")?;

    // prepare messages:
    let messages = Messages::new()
        .user(vec!["Hello, how are you doing?".into()])
        .wrap();

    // send request:
    let mut response = Completions::anthropic(api_key, "claude-opus-4-6")
        .proxy(Proxy::all("socks5://127.0.0.1:1080")?)
        .send(messages)
        .await?;

    // read response stream:
    while let Some(chunk) = response.next().await {
        if let AiChunk::Text(text) = chunk? {
            eprint!("{text}");
        }
    }
    println!();

    Ok(())
}
```

### ImageView:
```rust
use anylm::{AiChunk, Completions, Messages};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    // prepare messages:
    let messages = Messages::new()
        .user(vec![
            Path::new("test-image.png").into(),
            "What's on the picture?".into(),
        ])
        .wrap();

    // send request:
    let mut response = Completions::lmstudio("", "qwen/qwen2.5-vl-7b")
        .send(messages)
        .await?;

    // read response stream:
    while let Some(chunk) = response.next().await {
        if let AiChunk::Text(text) = chunk? {
            eprint!("{text}");
        }
    }
    println!();

    Ok(())
}
```

### Structured Output (JSON):
```rust
use anylm::{AiChunk, Completions, Messages, Schema};

/// The person structure
#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct Person {
    first_name: String,
    last_name: Option<String>,
    age: u8,
}

#[tokio::main]
async fn main() -> Result<()> {
    // prepare messages:
    let messages = Messages::new()
        .user(vec!["John Smith, 30 years old".into()])
        .wrap();

    // send request:
    let mut response = Completions::lmstudio("", "qwen/qwen2.5-vl-7b")
        .schema(
            Schema::object("The user structure")
                .required_property("first_name", Schema::string("The user first name"))
                .optional_property("last_name", Schema::string("The user last name"))
                .required_property("age", Schema::integer("The user age")),
        )
        .send(messages)
        .await?;

    // read response stream:
    let mut json_str = String::new();
    while let Some(chunk) = response.next().await {
        if let AiChunk::Text(text) = chunk? {
            json_str.push_str(&text);
        }
    }

    // parse response as JSON:
    let person: Person = serde_json::from_str(&json_str)?;
    println!("{person:#?}");

    Ok(())
}
```

### Tool Calls:
```rust
use anylm::{AiChunk, Completions, Messages, Schema, Tool, ToolCall};

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
```

### Embeddings:
```rust
use anylm::{Embeddings, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    // send request:
    let embeddings = Embeddings::lmstudio("", "nomic-ai/nomic-embed-text-v1.5")
        .input("Hello, how are you doing?")
        .send()
        .await?;

    // print response:
    println!("Embeddings: {:?}", embeddings.data);

    Ok(())
}
```

> And etc., it all has the same logic..

## License & Feedback:

> This library distributed under the [MIT](https://github.com/fuderis/anylm-rs/blob/main/LICENSE.md) license.

You can contact me via [GitHub](https://github.com/fuderis) or send a message to my [E-Mail](mailto:synapdrake@ya.ru).
This library is actively evolving, and your suggestions and feedback are always welcome!
