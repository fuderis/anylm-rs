use crate::api::Content;

/// Tokenizes text.
pub fn tokenize(text: &str) -> Vec<u32> {
    let tokenizer = tiktoken_rs::cl100k_base().expect("Failed to create tokenizer");
    tokenizer.encode_with_special_tokens(text)
}

/// Returns tokens count in string.
pub fn count_tokens(text: &str) -> usize {
    tokenize(text).len()
}

/// Helper method to return the content tokens count.
pub fn content_tokens(content: &[Content]) -> usize {
    content
        .iter()
        .map(|c| match c {
            Content::Text { text } => count_tokens(text),
            Content::Image { detail, .. } => match detail.as_deref() {
                Some("high") => 170,
                Some("auto") => 110,
                _ => 85, // low (by default)
            },
        })
        .sum::<usize>()
}
