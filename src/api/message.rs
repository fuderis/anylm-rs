use super::{Content, Role};
use crate::{prelude::*, utils};

use chrono::{DateTime, Utc};

/// The request message
#[derive(From, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[from(Bytes, "Message::user(vec![String::from_utf8_lossy(&value).into()])")]
#[from(String, "Message::user(vec![value.into()])")]
#[from(&str, "Message::user(vec![value.into()])")]
pub struct Message {
    pub role: Role,
    pub content: Vec<Content>,
    #[serde(default)]
    pub tokens_count: usize,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,
}

impl Message {
    /// Returns message tokens count
    fn count_tokens(content: &[Content]) -> usize {
        content
            .iter()
            .map(|c| match c {
                Content::Text { text } => utils::count_tokens(&text),
                Content::Image { detail, .. } => match detail.as_deref() {
                    Some("high") => 170,
                    Some("auto") => 110,
                    _ => 85, // low (by default)
                },
            })
            .sum::<usize>()
    }

    /// Creates a new message structure
    pub fn new(role: Role, content: Vec<Content>) -> Self {
        let tokens_count = Self::count_tokens(&content);

        Self {
            role,
            content,
            tokens_count,
            timestamp: Some(Utc::now()),
        }
    }

    /// The system prompt message
    pub fn system(content: Vec<Content>) -> Self {
        Self::new(Role::System, content)
    }

    /// The user prompt message
    pub fn user(content: Vec<Content>) -> Self {
        Self::new(Role::User, content)
    }

    /// The assistant prompt message
    pub fn assistant(content: Vec<Content>) -> Self {
        Self::new(Role::Assistant, content)
    }
}
