use crate::prelude::*;

/// The JSON-schema kind
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum JsonSchemaKind {
    Object,
    Array,
    String,
    Number,
    Integer,
    Boolean,
    Null,
}

impl JsonSchemaKind {
    /// Returns true if this is `object` schema
    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object)
    }

    /// Returns true if this is `array` schema
    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array)
    }

    /// Returns true if this is `string` schema
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String)
    }

    /// Returns true if this is `integer` schema
    pub fn is_integer(&self) -> bool {
        matches!(self, Self::Integer)
    }

    /// Returns true if this is `boolean` schema
    pub fn is_boolean(&self) -> bool {
        matches!(self, Self::Boolean)
    }

    /// Returns true if this is `null` schema
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

impl Default for JsonSchemaKind {
    fn default() -> Self {
        Self::Object
    }
}

/// The JSON-schema property
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct JsonSchema {
    /// The schema type
    #[serde(rename = "type")]
    pub kind: JsonSchemaKind,
    /// The schema description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The string value variants
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(alias = "enum")]
    pub variants: Option<HashSet<String>>,
    /// The minimum value for number
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,
    /// The maximum value for number
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
    /// The array items type
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<JsonSchema>>,
    /// The object properties
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, Box<JsonSchema>>>,
    /// The required object properties
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<HashSet<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub optional: Option<bool>,
    #[serde(rename = "additionalProperties")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<bool>,
}

impl JsonSchema {
    /// Creates a new schema
    pub fn new(kind: JsonSchemaKind, descr: impl Into<String>) -> Self {
        Self {
            kind,
            description: match descr.into() {
                s if !s.is_empty() => Some(s),
                _ => None,
            },
            ..Default::default()
        }
    }

    /// Creates an object schema
    pub fn object(descr: impl Into<String>) -> Self {
        Self::new(JsonSchemaKind::Object, descr)
    }

    /// Creates an array schema
    pub fn array(descr: impl Into<String>) -> Self {
        Self::new(JsonSchemaKind::Array, descr)
    }

    /// Creates a string schema
    pub fn string(descr: impl Into<String>) -> Self {
        Self::new(JsonSchemaKind::String, descr)
    }

    /// Creates a number schema
    pub fn number(descr: impl Into<String>) -> Self {
        Self::new(JsonSchemaKind::Number, descr)
    }

    /// Creates a integer schema
    pub fn integer(descr: impl Into<String>) -> Self {
        Self::new(JsonSchemaKind::Integer, descr)
    }

    /// Creates a boolean schema
    pub fn boolean(descr: impl Into<String>) -> Self {
        Self::new(JsonSchemaKind::Boolean, descr)
    }

    /// Creates a null schema
    pub fn null(descr: impl Into<String>) -> Self {
        Self::new(JsonSchemaKind::Null, descr)
    }

    /// Sets schema description
    pub fn description(mut self, descr: impl Into<String>) -> Self {
        self.description.replace(descr.into());
        self
    }

    /// Adds string value variants
    pub fn variants(mut self, vars: HashSet<String>) -> Self {
        self.variants.get_or_insert_default().extend(vars);
        self
    }

    /// Adds string value variant
    pub fn variant(mut self, var: impl Into<String>) -> Self {
        self.variants.get_or_insert_default().insert(var.into());
        self
    }

    /// Sets the minimum number value
    pub fn minimum(mut self, min: f64) -> Self {
        self.minimum.replace(min);
        self
    }

    /// Sets the maximum number value
    pub fn maximum(mut self, max: f64) -> Self {
        self.maximum.replace(max);
        self
    }

    /// Sets the array items schema
    pub fn items(mut self, schema: JsonSchema) -> Self {
        self.items.replace(Box::new(schema));
        self
    }

    /// Adds the object properties schema
    pub fn properties(mut self, props: HashMap<impl Into<String>, Box<JsonSchema>>) -> Self {
        self.properties
            .get_or_insert_default()
            .extend(props.into_iter().map(|(k, v)| (k.into(), v)));
        self
    }

    /// Adds the object property schema
    pub fn property(mut self, name: impl Into<String>, schema: JsonSchema, required: bool) -> Self {
        let name = name.into();
        let reqs = self.required.get_or_insert_default();

        if required {
            reqs.insert(name.clone());
        }

        self.properties
            .get_or_insert_default()
            .insert(name, Box::new(schema));
        self
    }

    /// Adds the required property schema
    pub fn required_property(self, name: impl Into<String>, schema: JsonSchema) -> Self {
        self.property(name, schema, true)
    }

    /// Adds the optional property schema
    pub fn optional_property(self, name: impl Into<String>, schema: JsonSchema) -> Self {
        self.property(name, schema, false)
    }

    /// Adds the required object properties
    pub fn required(mut self, reqs: Vec<impl Into<String>>) -> Self {
        self.required
            .get_or_insert_default()
            .extend(reqs.into_iter().map(Into::into));
        self
    }

    /// Adds the required object property
    pub fn add_required(mut self, name: impl Into<String>) -> Self {
        self.required.get_or_insert_default().insert(name.into());
        self
    }
}

impl JsonSchema {
    /// Converts into `OpenAI` format
    pub fn to_openai_format(&self) -> Result<JsonValue> {
        let mut schema_json = self.to_json_schema()?;

        // OpenAI Strict requires `additionalProperties: false`, `properties`, and `required` on ALL object levels
        Self::apply_openai_strict_rules(&mut schema_json);

        Ok(json!({
            "type": "json_schema",
            "json_schema": {
                "name": "response",
                "schema": schema_json,
                "strict": true
            }
        }))
    }

    /// Recursively applies OpenAI Strict Mode rules to all levels of the schema
    fn apply_openai_strict_rules(value: &mut JsonValue) {
        if let Some(obj) = value.as_object_mut() {
            if obj.get("type").and_then(|t| t.as_str()) == Some("object") {
                // 1. Every object MUST have additionalProperties: false
                obj.insert("additionalProperties".to_string(), json!(false));

                // 2. Every object MUST have "properties"
                if !obj.contains_key("properties") {
                    obj.insert("properties".to_string(), json!({}));
                }

                // 3. Every object MUST have "required" (even if empty)
                if !obj.contains_key("required") {
                    obj.insert("required".to_string(), json!([]));
                }
            }

            // Recursively process object properties
            if let Some(props) = obj.get_mut("properties").and_then(|p| p.as_object_mut()) {
                for prop in props.values_mut() {
                    Self::apply_openai_strict_rules(prop);
                }
            }

            // Recursively process array items
            if let Some(items) = obj.get_mut("items") {
                Self::apply_openai_strict_rules(items);
            }
        }
    }

    /// Converts into `Anthropic` (and others) format (output_config)
    pub fn to_anthropic_format(&self) -> Result<JsonValue> {
        let mut schema_json = self.to_json_schema()?;

        // For most APIs, explicitly prohibit additional properties:
        if let Some(obj) = schema_json.as_object_mut() {
            obj.insert("additionalProperties".to_string(), json!(false));
        }

        Ok(json!({
            "format": {
                "type": "json_schema",
                "schema": schema_json
            }
        }))
    }

    /// Converts into `Google` format
    pub fn to_google_format(&self) -> Result<JsonValue> {
        let schema_json = self.to_json_schema()?;

        // Google requires a MIME type to activate JSON mode:
        Ok(json!({
            "response_mime_type": "application/json",
            "response_schema": schema_json
        }))
    }
}

impl JsonSchema {
    /// Converts into valid JSON-format
    pub fn to_json_schema(&self) -> Result<JsonValue> {
        let mut v = serde_json::to_value(self)?;

        Self::sanitize_json_schema(&mut v);
        Ok(v)
    }

    /// Recursively purging the `optional` field and filling in the `required` field
    pub fn sanitize_json_schema(value: &mut JsonValue) {
        if let Some(obj) = value.as_object_mut() {
            // renaming `variants` field into `enum`:
            if let Some(variants_value) = obj.remove("variants") {
                obj.insert("enum".to_string(), variants_value);
            }

            // handling nesting in items (for arrays):
            if let Some(items) = obj.get_mut("items") {
                Self::sanitize_json_schema(items);
            }

            // extracting the required list or creating a new one:
            let mut required_set: HashSet<String> = obj
                .get("required")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            // processing object properties:
            if let Some(props) = obj.get_mut("properties").and_then(|p| p.as_object_mut()) {
                for (name, prop) in props.iter_mut() {
                    // RECURSION: cleaning nested objects before looking at their flag:
                    Self::sanitize_json_schema(prop);

                    if let Some(prop_obj) = prop.as_object_mut() {
                        // removing the optional flag:
                        let is_optional = prop_obj
                            .remove("optional")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                        if !is_optional {
                            required_set.insert(name.clone());
                        }
                    }
                }

                // write the updated required back to the object:
                let mut final_required: Vec<_> = required_set.into_iter().collect();
                final_required.sort();
                obj.insert("required".to_string(), serde_json::json!(final_required));
            } else if obj.get("type").and_then(|t| t.as_str()) == Some("object") {
                if !obj.contains_key("required") {
                    obj.insert("required".to_string(), json!([]));
                }
            }
        }
    }
}
