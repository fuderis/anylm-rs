use super::Schema;
use crate::prelude::*;

/// The tool call function
#[derive(From, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct ToolCallFunction {
    pub name: String,
    #[serde(rename = "arguments")]
    pub json_str: String,
}

impl ToolCallFunction {
    /// Parses the function arguments
    pub fn parse_args<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        Ok(serde_json::from_str(&self.json_str)?)
    }
}

/// The tool call
#[derive(From, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct ToolCall {
    #[serde(default)]
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(rename = "function")]
    pub func: ToolCallFunction,
}

impl ToolCall {
    /// Parses the function arguments
    pub fn parse_args<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        Ok(serde_json::from_str(&self.func.json_str)?)
    }
}

/// The tool call structure
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Tool {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(default = "Tool::default_schema")]
    parameters: Schema,
    #[serde(default)]
    properties: HashMap<String, Schema>,
}

impl Tool {
    fn default_schema() -> Schema {
        Schema::object("")
    }

    /// Creates a new tool call
    pub fn new(name: impl Into<String>, descr: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: match descr.into() {
                s if !s.is_empty() => Some(s),
                _ => None,
            },
            parameters: Schema::object(""),
            properties: HashMap::new(),
        }
    }

    /// Adds an argument
    pub fn property(mut self, name: impl Into<String>, schema: Schema, required: bool) -> Self {
        self.parameters = self.parameters.property(name, schema, required);
        self
    }

    /// Adds a required argument
    pub fn required_property(mut self, name: impl Into<String>, schema: Schema) -> Self {
        self.parameters = self.parameters.required_property(name, schema);
        self
    }

    /// Adds an optional argument
    pub fn optional_property(mut self, name: impl Into<String>, schema: Schema) -> Self {
        self.parameters = self.parameters.optional_property(name, schema);
        self
    }
}

impl Tool {
    /// Converts into `OpenAI` format: {"type": "function", "function": {...}}
    pub fn to_openai_format(&self) -> Result<JsonValue> {
        let tool_json = self.to_json_tool()?;

        Ok(json!({
            "type": "function",
            "function": tool_json
        }))
    }

    /// Converts into `Anthropic` format: replace "parameters" to "input_schema"
    pub fn to_anthropic_format(&self) -> Result<JsonValue> {
        let mut tool_json = self.to_json_tool()?;

        if let Some(obj) = tool_json.as_object_mut() {
            if let Some(params) = obj.remove("parameters") {
                obj.insert("input_schema".to_string(), params);
            }
        }

        Ok(tool_json)
    }

    /// Converts into `Google` format: {"function_declarations": [...]}
    pub fn to_google_format(&self) -> Result<JsonValue> {
        let tool_json = self.to_json_tool()?;

        Ok(json!({
            "function_declarations": [ tool_json ]
        }))
    }
}

impl Tool {
    /// Converts into valid JSON-format
    pub fn to_json_tool(&self) -> Result<JsonValue> {
        let mut this = self.clone();

        // push tool properties into schema:
        for (name, schema) in &self.properties {
            this = this.property(name, schema.clone(), !schema.optional);
        }
        this.properties.clear();

        // fix schema properties format:
        if this
            .parameters
            .properties
            .as_ref()
            .map(|props| props.is_empty())
            .unwrap_or(true)
        {
            this = this.optional_property("_", Schema::null(""));
        }

        let mut v = serde_json::to_value(this)?;
        if let Some(params) = v.get_mut("parameters") {
            Schema::sanitize_json_schema(params);
        }

        Schema::sanitize_json_schema(&mut v);
        Ok(v)
    }
}
