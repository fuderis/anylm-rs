use anylm::embeddings::Embeddings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    // 1. Indexing: Save user music preferences into the vector database
    let doc_vector = Embeddings::openai()
        .base_url("http://127.0.0.1:1234")
        .model("nomic-ai/nomic-embed-text-v1.5")
        .input("Loves classical piano music by Ludovico Einaudi.")
        .document() // storage optimization
        .send()
        .await?;

    println!(
        "Document vector generated (dims: {})",
        doc_vector.data[0].embedding.len()
    );

    // 2. Retrieval: Search context when user requests music playback
    let query_vector = Embeddings::openai()
        .base_url("http://127.0.0.1:1234")
        .model("nomic-ai/nomic-embed-text-v1.5")
        .input("Play my lovely music!")
        .query() // search optimization
        .send()
        .await?;

    println!(
        "Query vector generated (dims: {})",
        query_vector.data[0].embedding.len()
    );

    Ok(())
}
