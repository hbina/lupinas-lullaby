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
    pub info: InfoObj,
    pub servers: Vec<Server>,
    pub paths: BTreeMap<String, PathObj>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<ComponentObj>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Server {
    pub url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PathObj {
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub ref_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<OperationObj>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<OperationObj>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<OperationObj>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<OperationObj>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<OperationObj>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<OperationObj>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<OperationObj>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<OperationObj>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct OperationObj {
    #[serde(rename = "operationId", skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<ObjectOrReference<ParameterObj>>>,
    pub responses: BTreeMap<String, ObjectOrReference<ResponseObj>>,
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
pub struct ParameterObj {
    pub name: String,
    #[serde(rename = "in")]
    pub location: ParameterLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<ObjectOrReference<SchemaObj>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ResponseObj {
    pub description: String,
    #[serde(skip_serializing_if = " Option::is_none")]
    pub headers: Option<BTreeMap<String, ObjectOrReference<HeaderObj>>>,
    #[serde(skip_serializing_if = " Option::is_none")]
    pub content: Option<BTreeMap<String, MediaTypeObj>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MediaTypeObj {
    #[serde(skip_serializing_if = " Option::is_none")]
    pub schema: Option<ObjectOrReference<SchemaObj>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct HeaderObj {
    #[serde(skip_serializing_if = " Option::is_none")]
    pub required: Option<bool>,
    #[serde(skip_serializing_if = " Option::is_none")]
    pub deprecated: Option<bool>,
    #[serde(skip_serializing_if = " Option::is_none")]
    pub allow_empty_value: Option<bool>,
    #[serde(skip_serializing_if = " Option::is_none")]
    pub schema: Option<ObjectOrReference<SchemaObj>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct InfoObj {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "termsOfService", skip_serializing_if = "Option::is_none")]
    pub terms_of_service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<ContactObj>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<LicenseObj>,
    pub version: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct LicenseObj {
    pub name: String,
    pub url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ContactObj {
    pub name: Option<String>,
    pub url: Option<String>,
    pub email: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ComponentObj {
    /// An object to hold reusable Schema Objects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<BTreeMap<String, ObjectOrReference<SchemaObj>>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SchemaObj {
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
    pub one_of: Option<Vec<ObjectOrReference<SchemaObj>>>,
    #[serde(rename = "allOf", skip_serializing_if = "Option::is_none")]
    pub all_of: Option<Vec<ObjectOrReference<SchemaObj>>>,
    #[serde(rename = "anyOf", skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<ObjectOrReference<SchemaObj>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not: Option<Vec<ObjectOrReference<SchemaObj>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<ObjectOrReference<SchemaObj>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<BTreeMap<String, ObjectOrReference<SchemaObj>>>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "additionalProperties"
    )]
    pub additional_properties: Option<BooleanObjectOrReference<Box<SchemaObj>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
}
