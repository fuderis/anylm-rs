use super::{Content, Role};
use crate::{
    api::{Image, ToolCall},
    prelude::*,
    utils,
};

use chrono::{DateTime, Utc};

/// Message visibility option.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Visibility {
    /// Visible to everyone.
    #[default]
    Public,

    /// Hidden from the user, but sent to LLM models.
    Internal,

    /// For debugging purposes only.
    Debug,
}

impl Visibility {
    pub fn is_public(&self) -> bool {
        matches!(self, Self::Public)
    }

    pub fn is_internal(&self) -> bool {
        matches!(self, Self::Internal)
    }

    pub fn is_debug(&self) -> bool {
        matches!(self, Self::Debug)
    }
}

/// The request message
#[derive(Default, From, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[from(Bytes, expr = Message::user(vec![String::from_utf8_lossy(&value).into()]))]
#[from(String, expr = Message::user(vec![value.into()]))]
#[from(&str, expr = Message::user(vec![value.into()]))]
pub struct Message {
    /// Message role [system|user|assistant|tool].
    pub role: Role,
    /// Message content [text|image].
    pub content: Vec<Content>,
    /// Tool calls list (for assistant message).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
    /// Tool call id (for tool message [tool result]).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub tool_call_id: String,
    /// Total token count for this message.
    #[serde(default)]
    pub tokens_count: usize,
    /// Message date and time in UTC format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,
    /// Message visibility (for debugging or visibility logic).
    #[serde(default)]
    pub visibility: Visibility,
}

impl Message {
    /// Creates new message structure.
    pub fn new(role: Role, content: Vec<Content>) -> Self {
        let tokens_count = utils::content_tokens(&content);

        Self {
            role,
            content,
            tokens_count,
            timestamp: Some(Utc::now()),
            ..Default::default()
        }
    }

    /// Creates new system prompt message.
    pub fn system(content: Vec<Content>) -> Self {
        Self::new(Role::System, content)
    }

    /// Creates new user prompt message.
    pub fn user(content: Vec<Content>) -> Self {
        Self::new(Role::User, content)
    }

    /// Creates new assistant response message.
    pub fn assistant(content: Vec<Content>, tool_calls: Vec<ToolCall>) -> Self {
        Self {
            tool_calls,
            ..Self::new(Role::Assistant, content)
        }
    }

    /// Creates new tool response message.
    pub fn tool(content: Vec<Content>, tool_call_id: String) -> Self {
        Self {
            tool_call_id,
            ..Self::new(Role::Tool, content)
        }
    }

    /// Maps message content.
    pub fn map_content(&mut self, f: impl FnOnce(&mut Vec<Content>)) {
        f(&mut self.content);
        self.count_tokens();
    }

    /// Extracts text parts from message content.
    pub fn extract_texts(&self) -> Vec<&str> {
        self.content
            .iter()
            .filter_map(|c| match c {
                Content::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
    }

    /// Extracts images from message content.
    pub fn extract_images(&self) -> Vec<&Image> {
        self.content
            .iter()
            .filter_map(|c| match c {
                Content::Image { image, .. } => Some(image),
                _ => None,
            })
            .collect::<Vec<_>>()
    }

    /// Recounts & updates the total tokens count.
    pub fn count_tokens(&mut self) -> usize {
        let count = utils::content_tokens(&self.content);
        self.tokens_count = count;
        count
    }

    /// Sets message visibility option.
    pub fn visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }
}
