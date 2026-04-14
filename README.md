[![github]](https://github.com/fuderis/anylm-rs)&ensp;
[![crates-io]](https://crates.io/crates/anylm)&ensp;
[![docs-rs]](https://docs.rs/anylm)

[github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs

# AnyLM - Universal API for Every AI

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
* **Is something missing?**: Write to me and I will add it too. (`Telegram`: [@fuderis](https://t.me/fuderis)).

## Examples:

### Cerebras:
```rust
use anylm::{Chunk, Completions, Proxy, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = std::env::var("CEREBRAS_API_KEY")?;

    // send request:
    let mut response = Completions::cerebras(api_key, "llama3.1-8b")
        //.proxy(Proxy::all("socks5://127.0.0.1:1080")?)
        .user_message(vec!["Hello, how are you doing?".into()])
        .send()
        .await?;

    // read response stream:
    while let Some(chunk) = response.next().await {
        if let Chunk::Text(text) = chunk? {
            eprint!("{text}");
        }
    }
    println!();

    Ok(())
}
```

### Claude:
```rust
use anylm::{Chunk, Completions, Proxy, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")?;

    // send request:
    let mut response = Completions::claude(api_key, "claude-opus-4-6")
        //.proxy(Proxy::all("socks5://127.0.0.1:1080")?)
        .user_message(vec!["Hello, how are you doing?".into()])
        .send()
        .await?;

    // read response stream:
    while let Some(chunk) = response.next().await {
        if let Chunk::Text(text) = chunk? {
            eprint!("{text}");
        }
    }
    println!();

    Ok(())
}
```

### ImageView:
```rust
use anylm::{Chunk, Completions, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    // send request:
    let mut response = Completions::lmstudio("", "qwen/qwen3-vl-4b")
        .server("http://localhost:1234")
        .user_message(vec![
            Path::new("test-image.png").into(),
            "What's on the picture?".into(),
        ])
        .send()
        .await?;

    // read response stream:
    while let Some(chunk) = response.next().await {
        if let Chunk::Text(text) = chunk? {
            eprint!("{text}");
        }
    }
    println!();

    Ok(())
}
```

### Structured Output (JSON):
```rust
use anylm::{Chunk, Completions, Schema, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    /// The person structure
    #[derive(Debug, serde::Deserialize)]
    struct Person {
        first_name: String,
        last_name: Option<String>,
        age: u8,
    }

    // send request:
    let mut response = Completions::lmstudio("", "mistralai/ministral-3-3b")
        .user_message(vec!["John Smith, 30 years old".into()])
        .schema(
            Schema::object("The user structure")
                .required_property("first_name", Schema::string("The user first name"))
                .optional_property("last_name", Schema::string("The user last name"))
                .required_property("age", Schema::integer("The user age")),
        )
        .send()
        .await?;

    // read response stream:
    let mut json_str = String::new();
    while let Some(chunk) = response.next().await {
        if let Chunk::Text(text) = chunk? {
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
use anylm::{Chunk, Completions, Schema, Tool, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    /// The weather tool data
    #[derive(Debug, serde::Deserialize)]
    struct LocationData {
        location: String,
    }

    // send request:
    let mut response = Completions::lmstudio("", "mistralai/ministral-3-3b")
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
```

### Embeddings:
```rust
use anylm::{Embeddings, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    // send request:
    let response = Embeddings::lmstudio("", "nomic-ai/nomic-embed-text-v1.5")
        .input("Hello, how are you doing?")
        .send()
        .await?;

    // print response:
    println!("Embeddings: {:?}", embeddings.data);

    Ok(())
}
```

> And etc., it all has the same logic..

## License & Credits:

* **License**: Distributed under the [*MIT*](https://github.com/fuderis/anylm-rs/blob/main/LICENSE.md) license.
* **Contacts**:
  [*GitHub*](https://github.com/fuderis),
  [*Behance*](https://behance.net/fuderis),
  [*Telegram*](https://t.me/fuderis),
  [*Telegram Channel*](https://t.me/fuderis_club),
  [*VKontakte*](https://vk.com/fuderis).
* **Donate**: `TON: UQBq2GVLt_nu6bF8ku0RneWDr_B0AdrBMXVPcRrNmTU6mz65`

> Thank you for your support! Don't forget to check out my other projects as well. [*GitHub*](https://github.com/fuderis)<br>
**P.s.**: This software is actively evolving, and your suggestions and feedback are always welcome!
