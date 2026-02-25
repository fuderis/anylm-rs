use anylm::{Chunk, Completions, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    // send request:
    let mut response = Completions::lmstudio("", "qwen2.5-coder-3b-instruct")
        .server("http://localhost:1234")
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
