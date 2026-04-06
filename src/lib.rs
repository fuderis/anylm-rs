#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
pub mod error;
pub use error::Error;
pub mod prelude;
pub mod utils;
pub use utils::*;

pub mod image;

pub mod api;
pub use api::{
    ApiKind, Chunk, Completions, Content, Embedding, Embeddings, EmbeddingsData, Message, Role,
    Schema, SchemaKind, Stream, Tool, Usage,
};

pub use bytes::{self, Bytes};
pub use reqwest::{self, Proxy};
