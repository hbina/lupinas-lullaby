use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Ref {
    #[serde(rename = "$ref")]
    pub ref_path: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum ObjectOrReference<T> {
    Ref(Ref),
    Object(T),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum BooleanObjectOrReference<T> {
    Boolean(bool),
    Object(T),
    Ref(Ref),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Spec3 {
    pub openapi: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Components>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Components {
    /// An object to hold reusable Schema Objects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<BTreeMap<String, ObjectOrReference<Schema>>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Schema {
    /// Properties.
    /// The following properties are taken directly from the [JSON Schema](https://tools.ietf.org/html/draft-wright-json-schema-00) definition and follow the same specification.
    /// TODO(hbina): Extend support to all of this.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "multipleOf", skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<Value>>,
    /// OpenAPI Specific Properties.
    /// The following properties are taken from the JSON Schema definition but their definitions were adjusted to the OpenAPI Specification.
    // TODO(hbina): Extend support to all of this.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub schema_type: Option<String>,
    // Inline or referenced schema MUST be of a [Schema Object](#schemaObject) and not a standard
    // JSON Schema.
    // [oneOf-anyOf-](https://swagger.io/docs/specification/data-models/oneof-anyof-allof-not/#oneof)
    #[serde(rename = "oneOf", skip_serializing_if = "Option::is_none")]
    pub one_of: Option<Vec<ObjectOrReference<Schema>>>,
    #[serde(rename = "allOf", skip_serializing_if = "Option::is_none")]
    pub all_of: Option<Vec<ObjectOrReference<Schema>>>,
    #[serde(rename = "anyOf", skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<ObjectOrReference<Schema>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not: Option<Vec<ObjectOrReference<Schema>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<ObjectOrReference<Schema>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<BTreeMap<String, ObjectOrReference<Schema>>>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "additionalProperties"
    )]
    pub additional_properties: Option<BooleanObjectOrReference<Box<Schema>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
}
