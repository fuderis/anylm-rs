use anylm::{
    api::{Messages, Schema},
    completions::{Chunk, Completions},
};

/// The person structure
#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct Person {
    first_name: String,
    last_name: Option<String>,
    age: u8,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    // prepare messages:
    let messages = Messages::new()
        .user(vec!["John Smith, 30 years old".into()])
        .wrap();

    // send request:
    let mut response = Completions::openai()
        .base_url("http://127.0.0.1:1234")
        .model("qwen/qwen3-vl-4b")
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
        if let Chunk::Text(text) = chunk? {
            json_str.push_str(&text);
        }
    }

    // parse response as JSON:
    let person: Person = serde_json::from_str(&json_str)?;
    println!("{person:#?}");

    Ok(())
}
