use crate::prelude::*;

/// The message role
#[derive(Clone, Debug, Display, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

impl Role {
    /// Returns true if it's the system prompt message
    pub fn is_system(&self) -> bool {
        matches!(self, Self::System)
    }

    /// Returns true if it's the user message
    pub fn is_user(&self) -> bool {
        matches!(self, Self::User)
    }

    /// Returns true if it's the assistant message
    pub fn is_assistant(&self) -> bool {
        matches!(self, Self::Assistant)
    }

    /// Returns true if it's the tool message
    pub fn is_tool(&self) -> bool {
        matches!(self, Self::Tool)
    }
}
