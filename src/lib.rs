#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
pub mod error;
mod prelude;
pub mod utils;

pub mod options;
pub use options::Options;

pub mod image;
pub use image::Image;

pub mod api;
pub use api::{Content, Message, Messages};

pub mod completions;
pub use completions::{Chunk, Completions};

pub mod embeddings;
pub use embeddings::{Embedding, Embeddings, EmbeddingsData, Search};

pub mod chunk;

pub use bytes::{self, Bytes};
pub use reqwest::{self, Proxy};
