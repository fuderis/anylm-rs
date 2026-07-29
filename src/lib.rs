#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
pub mod error;
pub mod options;
mod prelude;

pub mod utils;

pub mod image;

pub mod api;
pub mod chunk;

pub mod completions;
pub mod embeddings;

pub use bytes::{self, Bytes};
pub use reqwest::{self, Proxy};
