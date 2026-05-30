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
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    properties: HashMap<String, Schema>,
}

impl Tool {
    /// Creates a new tool schema
    pub fn new(name: impl Into<String>, descr: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: match descr.into() {
                s if !s.is_empty() => Some(s),
                _ => None,
            },
            properties: HashMap::new(),
        }
    }

    /// Adds an argument
    pub fn property(mut self, name: impl Into<String>, mut schema: Schema, required: bool) -> Self {
        schema.optional = Some(!required);
        self.properties.insert(name.into(), schema);
        self
    }

    /// Adds a required argument
    pub fn required_property(self, name: impl Into<String>, schema: Schema) -> Self {
        self.property(name, schema, true)
    }

    /// Adds an optional argument
    pub fn optional_property(self, name: impl Into<String>, schema: Schema) -> Self {
        self.property(name, schema, false)
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
        let mut parameters_schema = Schema::object("");

        for (name, schema) in &self.properties {
            let is_required = !schema.optional.unwrap_or(true);
            parameters_schema = parameters_schema.property(name, schema.clone(), is_required);
        }

        // plug for empty parameters:
        let has_props = parameters_schema
            .properties
            .as_ref()
            .map(|props| !props.is_empty())
            .unwrap_or(false);

        if !has_props {
            parameters_schema = parameters_schema.optional_property("_", Schema::null(""));
        }

        // serializing raw tool:
        let mut v = serde_json::to_value(self)?;

        // removing properties field:
        if let Some(obj) = v.as_object_mut() {
            obj.remove("properties");

            // serializing parameters & insert it:
            let mut params_json = serde_json::to_value(parameters_schema)?;
            Schema::sanitize_json_schema(&mut params_json);
            obj.insert("parameters".to_string(), params_json);
        }

        Schema::sanitize_json_schema(&mut v);
        Ok(v)
    }
}
