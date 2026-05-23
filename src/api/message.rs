use super::{Content, Role};
use crate::{ToolCall, prelude::*, utils};

use chrono::{DateTime, Utc};
use std::{path::Path, sync::Arc};
use tokio::{fs, sync::Mutex};

/// The request messages
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Messages {
    pub messages: Vec<Message>,
    pub tokens_count: usize,
}

impl Messages {
    /// Creates an empty messages list
    pub fn new() -> Self {
        Self {
            messages: vec![],
            tokens_count: 0,
        }
    }

    /// Reads the messages from file
    pub async fn read(path: impl AsRef<Path>) -> Result<Self> {
        let contents = fs::read_to_string(path.as_ref()).await?;
        let mut messages = Vec::new();

        // read file lines:
        for line in contents.lines() {
            let line = line.trim();
            if !line.is_empty() {
                let msg: Message = json::from_str(line)?;
                messages.push(msg);
            }
        }

        let mut this = Self {
            messages,
            tokens_count: 0,
        };
        this.count_tokens();

        Ok(this)
    }

    /// Saves the messages to file
    pub async fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let mut contents = String::new();

        // write file lines:
        for msg in &self.messages {
            let json_line = json::to_string(msg)?;
            contents.push_str(&json_line);
            contents.push('\n');
        }

        fs::write(path.as_ref(), contents).await?;
        Ok(())
    }

    /// Saves the last message to file
    pub async fn save_last(&self, path: impl AsRef<Path>) -> Result<()> {
        let Some(last_msg) = self.messages.last() else {
            return Ok(());
        };

        let mut json_line = json::to_string(last_msg)?;
        json_line.push('\n');

        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path.as_ref())
            .await?;

        use tokio::io::AsyncWriteExt;
        file.write_all(json_line.as_bytes()).await?;

        Ok(())
    }

    /// Saves the last N messages to file
    pub async fn save_last_n(&self, path: impl AsRef<Path>, count: usize) -> Result<()> {
        if count == 0 || self.messages.is_empty() {
            return Ok(());
        }

        let skip_amount = self.messages.len().saturating_sub(count);
        let new_messages = &self.messages[skip_amount..];

        let mut contents = String::new();
        for msg in new_messages {
            let json_line = json::to_string(msg)?;
            contents.push_str(&json_line);
            contents.push('\n');
        }

        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path.as_ref())
            .await?;

        use tokio::io::AsyncWriteExt;
        file.write_all(contents.as_bytes()).await?;

        Ok(())
    }

    /// Adds a message to request
    pub fn message(mut self, msg: Message) -> Self {
        self.tokens_count += msg.tokens_count;
        self.messages.push(msg);
        self
    }
    /// Adds a message to request
    pub fn add_message(&mut self, msg: Message) {
        self.tokens_count += msg.tokens_count;
        self.messages.push(msg);
    }

    /// Adds a messages to request
    pub fn messages(mut self, msgs: Vec<Message>) -> Self {
        for msg in msgs {
            self.tokens_count += msg.tokens_count;
            self.messages.push(msg);
        }
        self
    }
    /// Adds a messages to request
    pub fn add_messages(&mut self, msgs: Vec<Message>) {
        for msg in msgs {
            self.tokens_count += msg.tokens_count;
            self.messages.push(msg);
        }
    }

    /// Adds the system message to request
    pub fn system(self, content: Vec<Content>) -> Self {
        self.message(Message::system(content))
    }
    /// Adds the user message to request
    pub fn user(self, content: Vec<Content>) -> Self {
        self.message(Message::user(content))
    }
    /// Adds the assistant message to request
    pub fn assistant(self, content: Vec<Content>, tool_calls: Vec<ToolCall>) -> Self {
        self.message(Message::assistant(content, tool_calls))
    }
    /// Adds the tool message to request
    pub fn tool(self, tool_call_id: String, content: Vec<Content>) -> Self {
        self.message(Message::tool(content, tool_call_id))
    }

    /// Adds the system message to request
    pub fn add_system(&mut self, content: Vec<Content>) {
        self.add_message(Message::system(content));
    }
    /// Adds the user message to request
    pub fn add_user(&mut self, content: Vec<Content>) {
        self.add_message(Message::user(content));
    }
    /// Adds the assistant message to request
    pub fn add_assistant(&mut self, content: Vec<Content>, tool_calls: Vec<ToolCall>) {
        self.add_message(Message::assistant(content, tool_calls));
    }
    /// Adds the tool message to request
    pub fn add_tool(&mut self, tool_call_id: String, content: Vec<Content>) {
        self.add_message(Message::tool(content, tool_call_id));
    }

    /// Counts & updates the tokens count
    pub fn count_tokens(&mut self) {
        let mut total = 0;
        for msg in &mut self.messages {
            msg.count_tokens();
            total += msg.tokens_count;
        }
        self.tokens_count = total;
    }

    /// Wraps into Arc<Mutex<_>>
    pub fn wrap(self) -> Arc<Mutex<Self>> {
        arc_mutex!(self)
    }
}

impl Serialize for Messages {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.messages.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Messages {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let messages = Vec::<Message>::deserialize(deserializer)?;
        let mut this = Self {
            messages,
            tokens_count: 0,
        };
        this.count_tokens();

        Ok(this)
    }
}

impl From<Vec<Message>> for Messages {
    fn from(messages: Vec<Message>) -> Self {
        let mut this = Self {
            messages,
            tokens_count: 0,
        };

        this.count_tokens();
        this
    }
}

/// The request message
#[derive(From, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[from(Bytes, "Message::user(vec![String::from_utf8_lossy(&value).into()])")]
#[from(String, "Message::user(vec![value.into()])")]
#[from(&str, "Message::user(vec![value.into()])")]
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

    /// Updates the number of used tokens
    pub fn count_tokens(&mut self) {
        self.tokens_count = count_tokens(&self.content);
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
