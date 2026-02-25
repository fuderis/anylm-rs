use crate::prelude::*;

/// The JSON-schema kind
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum SchemaKind {
    Object,
    Array,
    String,
    Number,
    Integer,
    Boolean,
    Null,
}

impl Default for SchemaKind {
    fn default() -> Self {
        Self::Object
    }
}

/// The JSON-schema property
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// The schema type
    #[serde(rename = "type")]
    pub kind: SchemaKind,
    /// The schema description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The string value variants
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "enum")]
    pub variants: Option<HashSet<String>>,
    /// The minimum value for number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,
    /// The maximum value for number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
    /// The array items type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Schema>>,
    /// The object properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, Box<Schema>>>,
    /// The required object properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<HashSet<String>>,
    #[serde(rename = "additionalProperties")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<bool>,
}

impl Schema {
    /// Creates a new schema
    pub fn new(kind: SchemaKind, descr: impl Into<String>) -> Self {
        Self {
            kind,
            description: match descr.into() {
                s if !s.is_empty() => Some(s),
                _ => None,
            },
            additional_properties: Some(false),
            ..Default::default()
        }
    }

    /// Creates an object schema
    pub fn object(descr: impl Into<String>) -> Self {
        Self::new(SchemaKind::Object, descr)
    }

    /// Creates an array schema
    pub fn array(descr: impl Into<String>) -> Self {
        Self::new(SchemaKind::Array, descr)
    }

    /// Creates a string schema
    pub fn string(descr: impl Into<String>) -> Self {
        Self::new(SchemaKind::String, descr)
    }

    /// Creates a number schema
    pub fn number(descr: impl Into<String>) -> Self {
        Self::new(SchemaKind::Number, descr)
    }

    /// Creates a integer schema
    pub fn integer(descr: impl Into<String>) -> Self {
        Self::new(SchemaKind::Integer, descr)
    }

    /// Creates a boolean schema
    pub fn boolean(descr: impl Into<String>) -> Self {
        Self::new(SchemaKind::Boolean, descr)
    }

    /// Creates a null schema
    pub fn null(descr: impl Into<String>) -> Self {
        Self::new(SchemaKind::Null, descr)
    }

    /// Sets schema description
    pub fn set_description(&mut self, descr: impl Into<String>) {
        self.description.replace(descr.into());
    }
    /// Sets schema description
    pub fn description(mut self, descr: impl Into<String>) -> Self {
        self.set_description(descr);
        self
    }

    /// Adds string value variants
    pub fn set_variants(&mut self, vars: HashSet<String>) {
        self.variants.get_or_insert_default().extend(vars);
    }
    /// Adds string value variants
    pub fn variants(mut self, vars: HashSet<String>) -> Self {
        self.set_variants(vars);
        self
    }

    /// Adds string value variant
    pub fn set_variant(&mut self, var: impl Into<String>) {
        self.variants.get_or_insert_default().insert(var.into());
    }
    /// Adds string value variant
    pub fn variant(mut self, var: impl Into<String>) -> Self {
        self.set_variant(var);
        self
    }

    /// Sets the minimum number value
    pub fn set_minimum(&mut self, min: f64) {
        self.minimum.replace(min);
    }
    /// Sets the minimum number value
    pub fn minimum(mut self, min: f64) -> Self {
        self.set_minimum(min);
        self
    }

    /// Sets the maximum number value
    pub fn set_maximum(&mut self, max: f64) {
        self.maximum.replace(max);
    }
    /// Sets the maximum number value
    pub fn maximum(mut self, max: f64) -> Self {
        self.set_maximum(max);
        self
    }

    /// Sets the array items schema
    pub fn set_items(&mut self, schema: Schema) {
        self.items.replace(Box::new(schema));
    }
    /// Sets the array items schema
    pub fn items(mut self, schema: Schema) -> Self {
        self.set_items(schema);
        self
    }

    /// Adds the object properties schema
    pub fn set_properties(&mut self, props: HashMap<impl Into<String>, Box<Schema>>) {
        self.properties
            .get_or_insert_default()
            .extend(props.into_iter().map(|(k, v)| (k.into(), v)));
    }
    /// Adds the object properties schema
    pub fn properties(mut self, props: HashMap<impl Into<String>, Box<Schema>>) -> Self {
        self.set_properties(props);
        self
    }

    /// Adds the object property schema
    pub fn set_property(&mut self, name: impl Into<String>, schema: Schema, required: bool) {
        let name = name.into();
        let reqs = self.required.get_or_insert_default();

        if required {
            reqs.insert(name.clone());
        }
        self.properties
            .get_or_insert_default()
            .insert(name, Box::new(schema));
    }
    /// Adds the object property schema
    pub fn property(mut self, name: impl Into<String>, schema: Schema, required: bool) -> Self {
        self.set_property(name, schema, required);
        self
    }

    /// Adds the required property schema
    pub fn set_required_property(&mut self, name: impl Into<String>, schema: Schema) {
        self.set_property(name, schema, true);
    }
    /// Adds the required property schema
    pub fn required_property(self, name: impl Into<String>, schema: Schema) -> Self {
        self.property(name, schema, true)
    }

    /// Adds the optional property schema
    pub fn set_optional_property(&mut self, name: impl Into<String>, schema: Schema) {
        self.set_property(name, schema, false);
    }
    /// Adds the optional property schema
    pub fn optional_property(self, name: impl Into<String>, schema: Schema) -> Self {
        self.property(name, schema, false)
    }

    /// Adds the required object properties
    pub fn set_required(&mut self, reqs: Vec<impl Into<String>>) {
        self.required
            .get_or_insert_default()
            .extend(reqs.into_iter().map(Into::into));
    }
    /// Adds the required object properties
    pub fn required(mut self, reqs: Vec<impl Into<String>>) -> Self {
        self.set_required(reqs);
        self
    }

    /// Adds the required object property
    pub fn set_require(&mut self, name: impl Into<String>) {
        self.required.get_or_insert_default().insert(name.into());
    }
    /// Adds the required object property
    pub fn require(mut self, name: impl Into<String>) -> Self {
        self.set_require(name);
        self
    }
}
