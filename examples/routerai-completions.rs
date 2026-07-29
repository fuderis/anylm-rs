use anylm::{
    api::Messages,
    completions::{Chunk, Completions},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

#[tokio::main]
async fn main() -> Result<()> {
    // prepare messages
    let messages = Messages::new()
        .user(vec!["Hello, how are you doing?".into()])
        .wrap();

    // send request
    let mut response = Completions::openai()
        .base_url("https://routerai.ru/api")
        .read_key("ROUTERAI_API_KEY")?
        .model("qwen/qwen3-coder-plus")
        .send(messages)
        .await?;

    // read response stream
    while let Some(chunk) = response.next().await {
        if let Chunk::Text(text) = chunk? {
            eprint!("{text}");
        }
    }
    println!();

    Ok(())
}
