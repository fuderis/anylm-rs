use anylm::{AiChunk, Completions};
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

#[tokio::main]
async fn main() -> Result<()> {
    // send request:
    let mut response = Completions::lmstudio("", "qwen/qwen2.5-vl-7b")
        .user_message(vec![
            Path::new("test-image.png").into(),
            "What's on the picture?".into(),
        ])
        .send()
        .await?;

    // read response stream:
    while let Some(chunk) = response.next().await {
        if let AiChunk::Text { text } = chunk? {
            eprint!("{text}");
        }
    }
    println!();

    Ok(())
}
