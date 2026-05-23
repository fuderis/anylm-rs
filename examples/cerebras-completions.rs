use anylm::{AiChunk, Completions, Messages, Proxy};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

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
