use anylm::{Embeddings, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    // send request:
    let embeddings = Embeddings::lmstudio("", "nomic-ai/nomic-embed-text-v1.5")
        .input("Hello, how are you doing?")
        .send()
        .await?;

    // print response:
    println!("Embeddings: {:?}", embeddings.data);

    Ok(())
}
