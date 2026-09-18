use crate::prelude::*;

/// Embedding search optimization.
#[derive(Debug, Display, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Search {
    /// Uses for save context.
    #[display(fmt = "search_document")]
    Document,
    /// Uses for search context.
    #[display(fmt = "search_query")]
    Query,
}
