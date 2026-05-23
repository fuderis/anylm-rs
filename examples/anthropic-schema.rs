use anylm::{AiChunk, Completions, Messages, Proxy, Schema};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")?;

    /// The person structure
    #[allow(dead_code)]
    #[derive(Debug, serde::Deserialize)]
    struct Person {
        first_name: String,
        last_name: Option<String>,
        age: u8,
    }

    // prepare messages:
    let messages = Messages::new()
        .user(vec!["John Smith, 30 years old".into()])
        .wrap();

    // send request:
    let mut response = Completions::anthropic(api_key, "claude-opus-4-6")
        .proxy(Proxy::all("socks5://127.0.0.1:1080")?)
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
