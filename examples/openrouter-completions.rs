use anylm::{Chunk, Completions, Proxy, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = std::env::var("OPENROUTER_API_KEY")?;

    // send request:
    let mut response = Completions::openrouter(api_key, "openai/gpt-4")
        .proxy(Proxy::all("socks5://127.0.0.1:1080")?)
        .user_message(vec!["Hello, how are you doing?".into()])
        .send()
        .await?;

    // read response stream:
    while let Some(chunk) = response.next().await {
        match chunk {
            Ok(Chunk { text }) => eprint!("{text}"),
            Err(e) => eprintln!("\n{e}"),
        };
    }
    println!();

    Ok(())
}
