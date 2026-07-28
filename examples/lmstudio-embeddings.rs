use anylm::Embeddings;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Indexing: Save user music preferences into the vector database
    let doc_vector = Embeddings::lmstudio("", "nomic-ai/nomic-embed-text-v1.5")
        .input("Loves classical piano, ambient, and Ludovico Einaudi.")
        .document() // storage optimization
        .send()
        .await?;

    println!(
        "Document vector generated (dims: {})",
        doc_vector.data[0].embedding.len()
    );

    // 2. Retrieval: Search context when user requests music playback
    let query_vector = Embeddings::lmstudio("", "nomic-ai/nomic-embed-text-v1.5")
        .input("Play my favorite music!")
        .query() // search optimization
        .send()
        .await?;

    println!(
        "Query vector generated (dims: {})",
        query_vector.data[0].embedding.len()
    );

    Ok(())
}
