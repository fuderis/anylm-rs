use anylm::{Chunk, Completions, prelude::*};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    // send request:
    let mut response = Completions::lmstudio(1234, "qwen/qwen3-vl-4b")
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
