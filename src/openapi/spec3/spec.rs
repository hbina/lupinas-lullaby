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
    // TODO(hbina): Deserialize as [semantic version number](https://semver.org/spec/v2.0.0.html)
    pub openapi: String,
    pub info: Info,
    pub paths: BTreeMap<String, Path>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Components>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Path {
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    ref_path: Option<ObjectOrReference<Box<Path>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    get: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    put: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    post: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delete: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    head: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    patch: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trace: Option<Operation>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Operation {
    #[serde(skip_serializing_if = "Option::is_none")]
    operation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<Vec<ObjectOrReference<Parameter>>>,
    responses: BTreeMap<String, ObjectOrReference<Response>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum ParameterLocation {
    #[serde(rename = "query")]
    Query,
    #[serde(rename = "header")]
    Header,
    #[serde(rename = "path")]
    Path,
    #[serde(rename = "cookie")]
    Cookie,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Parameter {
    name: String,
    #[serde(rename = "in")]
    location: ParameterLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deprecated: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Response {
    #[serde(skip_serializing_if = " Option::is_none")]
    headers: Option<BTreeMap<String, ObjectOrReference<Header>>>,
    #[serde(skip_serializing_if = " Option::is_none")]
    content: Option<BTreeMap<String, MediaType>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MediaType {
    #[serde(skip_serializing_if = " Option::is_none")]
    schema: Option<ObjectOrReference<Schema>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Header {
    #[serde(skip_serializing_if = " Option::is_none")]
    required: Option<bool>,
    #[serde(skip_serializing_if = " Option::is_none")]
    deprecated: Option<bool>,
    #[serde(skip_serializing_if = " Option::is_none")]
    allow_empty_value: Option<bool>,
    #[serde(skip_serializing_if = " Option::is_none")]
    schema: Option<ObjectOrReference<Schema>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Info {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "termsOfService", skip_serializing_if = "Option::is_none")]
    pub terms_of_service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<License>,
    pub version: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct License {
    name: String,
    url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Contact {
    name: Option<String>,
    url: Option<String>,
    email: Option<String>,
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
