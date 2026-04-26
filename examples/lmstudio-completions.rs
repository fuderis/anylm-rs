use anylm::{AiChunk, Completions};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

#[tokio::main]
async fn main() -> Result<()> {
    // send request:
    let mut response = Completions::lmstudio("", "qwen/qwen2.5-vl-7b")
        .host("http://localhost:1234")
        .user_message(vec!["Hello, how are you doing?".into()])
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
