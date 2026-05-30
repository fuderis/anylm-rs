use anylm::{AiChunk, Completions, Messages};
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

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
    let mut response = Completions::lmstudio("", "qwen/qwen3-vl-4b")
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
