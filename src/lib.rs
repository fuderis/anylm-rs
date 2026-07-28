#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
pub mod error;
mod prelude;

pub mod utils;

pub mod image;

pub mod options;
pub use options::AiOptions;

pub mod chunk;

pub mod api;
pub use api::{
    AiChunk, AiStream, ApiKind, Completions, Content, Embedding, EmbeddingSearch, Embeddings,
    EmbeddingsData, Message, Messages, Role, Schema, SchemaKind, Tool, ToolCall, ToolCallFunction,
    Usage, count_tokens,
};

pub use bytes::{self, Bytes};
pub use reqwest::{self, Proxy};
