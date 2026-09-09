use crate::prelude::*;

/// Message role [system|user|assistant|tool].
#[derive(Default, Clone, Copy, Debug, Display, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System prompt.
    System,
    /// User message.
    #[default]
    User,
    /// Assistant response.
    Assistant,
    /// Tool call response.
    Tool,
}

impl Role {
    /// Returns true if it's the system prompt message.
    pub fn is_system(&self) -> bool {
        matches!(self, Self::System)
    }

    /// Returns true if it's the user message.
    pub fn is_user(&self) -> bool {
        matches!(self, Self::User)
    }

    /// Returns true if it's the assistant message.
    pub fn is_assistant(&self) -> bool {
        matches!(self, Self::Assistant)
    }

    /// Returns true if it's the tool message.
    pub fn is_tool(&self) -> bool {
        matches!(self, Self::Tool)
    }
}
