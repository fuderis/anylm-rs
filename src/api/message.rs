use super::{Content, Role};
use crate::{api::ToolCall, prelude::*, utils};

use chrono::{DateTime, Utc};

/// The message visibility option
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Visibility {
    /// It's visible to everyone
    #[default]
    Public,

    /// Not to show to the user, but to send models
    Internal,

    /// For debugging purposes only
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
#[derive(From, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[from(Bytes, expr = Message::user(vec![String::from_utf8_lossy(&value).into()]))]
#[from(String, expr = Message::user(vec![value.into()]))]
#[from(&str, expr = Message::user(vec![value.into()]))]
pub struct Message {
    pub role: Role,
    pub content: Vec<Content>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub tool_call_id: String,
    #[serde(default)]
    pub tokens_count: usize,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,
    #[serde(default)]
    pub visibility: Visibility,
}

impl Message {
    /// Creates a new message structure
    pub fn new(role: Role, content: Vec<Content>) -> Self {
        let tokens_count = count_tokens(&content);

        Self {
            role,
            content,
            tokens_count,
            timestamp: Some(Utc::now()),
            tool_calls: vec![],
            tool_call_id: str!(),
            visibility: Visibility::Public,
        }
    }

    /// Creates the system prompt message
    pub fn system(content: Vec<Content>) -> Self {
        Self::new(Role::System, content)
    }

    /// Creates the user prompt message
    pub fn user(content: Vec<Content>) -> Self {
        Self::new(Role::User, content)
    }

    /// Creates the assistant response message
    pub fn assistant(content: Vec<Content>, tool_calls: Vec<ToolCall>) -> Self {
        let mut this = Self::new(Role::Assistant, content);
        this.tool_calls = tool_calls;
        this
    }

    /// Creates the tool response message
    pub fn tool(content: Vec<Content>, tool_call_id: String) -> Self {
        let mut this = Self::new(Role::Tool, content);
        this.tool_call_id = tool_call_id;
        this
    }

    /// Maps the message content
    pub fn map(&mut self, f: impl FnOnce(&mut Vec<Content>)) {
        f(&mut self.content);
        self.count_tokens();
    }

    /// Counts & updates the tokens count
    pub fn count_tokens(&mut self) -> usize {
        let count = count_tokens(&self.content);
        self.tokens_count = count;
        count
    }

    /// Sets the visibility option
    pub fn visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }
}

/// Returns the message tokens count
pub fn count_tokens(content: &[Content]) -> usize {
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
