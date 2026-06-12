use super::{Content, Image, Message, Role};
use crate::{ToolCall, prelude::*};

use std::{path::Path, sync::Arc};
use tokio::{fs, sync::Mutex};

/// The request messages
#[derive(Default, Debug, Clone, Eq, PartialEq)]
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

    /// Wraps into Arc<Mutex<_>>
    pub fn wrap(self) -> Arc<Mutex<Self>> {
        arc_mutex!(self)
    }
}

impl Messages {
    /// Finds the index of response message, or creates a new one
    fn find_or_create_index(&mut self, tool_call_id: Option<&str>) -> usize {
        let mut target_idx = None;

        match tool_call_id {
            // assistant response:
            None => {
                if let Some(last_msg) = self.messages.last() {
                    match last_msg.role {
                        Role::User => {}
                        Role::Assistant => {
                            target_idx = Some(self.messages.len() - 1);
                        }
                        Role::Tool | Role::System => {
                            if let Some((idx, _)) = self
                                .messages
                                .iter()
                                .enumerate()
                                .rev()
                                .find(|(_, m)| m.role == Role::Assistant)
                            {
                                target_idx = Some(idx);
                            }
                        }
                    }
                }
            }

            // tool call response:
            Some(id) => {
                if let Some((idx, _)) = self
                    .messages
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(_, m)| m.role == Role::Tool && m.tool_call_id == id)
                {
                    target_idx = Some(idx);
                }
            }
        }

        // return index or create a new one:
        if let Some(idx) = target_idx {
            idx
        } else {
            let new_msg = match tool_call_id {
                Some(id) => Message::tool(vec![], id.to_string()),
                None => Message::assistant(vec![], vec![]),
            };
            self.add_message(new_msg);
            self.messages.len() - 1
        }
    }

    /// Push a text content part into last message (assistant/tool)
    pub fn push_str(&mut self, tool_call_id: Option<&str>, text_part: &str) {
        if text_part.is_empty() {
            return;
        }

        let idx = self.find_or_create_index(tool_call_id);
        let msg = &mut self.messages[idx];
        let old_msg_tokens = msg.tokens_count;

        if let Some(Content::Text { text }) = msg.content.last_mut() {
            text.push_str(text_part);
        } else {
            msg.content.push(Content::Text {
                text: text_part.to_string(),
            });
        }

        self.tokens_count -= old_msg_tokens;
        self.tokens_count += msg.count_tokens();
    }

    /// Push a new content into last message (assistant/tool)
    pub fn push_content(&mut self, tool_call_id: Option<&str>, content: impl Into<Content>) {
        let content = content.into();
        if content.is_empty() {
            return;
        }

        let idx = self.find_or_create_index(tool_call_id);
        let msg = &mut self.messages[idx];
        let old_msg_tokens = msg.tokens_count;

        msg.content.push(content);

        self.tokens_count -= old_msg_tokens;
        self.tokens_count += msg.count_tokens();
    }

    /// Push a new image content into last message (assistant/tool)
    pub fn push_image(&mut self, tool_call_id: Option<&str>, image: Image, detail: Option<String>) {
        let content = Content::Image { image, detail };

        let idx = self.find_or_create_index(tool_call_id);
        let msg = &mut self.messages[idx];
        let old_msg_tokens = msg.tokens_count;

        msg.content.push(content);

        self.tokens_count -= old_msg_tokens;
        self.tokens_count += msg.count_tokens();
    }

    /// Removes messages from the context and returns them [> 0 - from start, < 0 - from end]
    pub fn slice(&mut self, pairs_count: isize) -> Vec<Message> {
        if pairs_count == 0 || self.messages.is_empty() {
            return vec![];
        }

        let target_pairs = pairs_count.abs() as usize;
        let mut keep_flags = vec![true; self.messages.len()];
        let mut found_pairs = 0;

        if pairs_count > 0 {
            // slice from start:
            let mut inside_pair = false;
            for (idx, msg) in self.messages.iter().enumerate() {
                if msg.role == Role::System {
                    continue;
                }

                if msg.role == Role::User {
                    inside_pair = true;
                }
                if inside_pair {
                    keep_flags[idx] = false;
                }
                if msg.role == Role::Assistant && inside_pair {
                    found_pairs += 1;
                    inside_pair = false;
                    if found_pairs >= target_pairs {
                        break;
                    }
                }
            }
        } else {
            // slice from end:
            let mut inside_pair = false;
            for (idx, msg) in self.messages.iter().enumerate().rev() {
                if msg.role == Role::System {
                    continue;
                }

                if msg.role == Role::Assistant {
                    inside_pair = true;
                }
                if inside_pair {
                    keep_flags[idx] = false;
                }
                if msg.role == Role::User && inside_pair {
                    found_pairs += 1;
                    inside_pair = false;
                    if found_pairs >= target_pairs {
                        break;
                    }
                }
            }
        }

        // collect messages:
        let mut retained = Vec::with_capacity(self.messages.len());
        let mut extracted = Vec::with_capacity(target_pairs * 2);

        let original_messages = std::mem::take(&mut self.messages);
        for (idx, msg) in original_messages.into_iter().enumerate() {
            if keep_flags[idx] {
                retained.push(msg);
            } else {
                extracted.push(msg);
            }
        }

        self.messages = retained;
        self.count_tokens();

        extracted
    }

    /// Counts & updates the total tokens count
    pub fn count_tokens(&mut self) -> usize {
        let mut total = 0;
        for msg in &mut self.messages {
            total += msg.count_tokens();
        }
        self.tokens_count = total;
        total
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
