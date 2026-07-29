use anylm::{
    api::Messages,
    completions::{Chunk, Completions},
};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    // prepare messages:
    let messages = Messages::new()
        .user(vec![
            Path::new("test-image.png").into(),
            "What's on the picture?".into(),
        ])
        .wrap();

    // send request:
    let mut response = Completions::openai()
        .base_url("http://127.0.0.1:1234")
        .model("qwen/qwen3-vl-4b")
        .send(messages)
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
