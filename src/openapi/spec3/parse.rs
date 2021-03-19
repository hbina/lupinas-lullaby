use std::collections::HashMap;

use super::spec::{ObjectOrReference, Schema, Spec3};
use crate::repr::{JavaScriptType, JavaScriptValue, ObjectRow};

pub fn parse_reference(reference: &str) -> String {
    let (prefix, name) = reference.split_at("#/components/schemas/".len());
    if prefix == "#/components/schemas/" {
        String::from(name)
    } else {
        let result = reference.to_string();
        panic!("Unable to parse reference:'{}'", result);
    }
}

fn parse_schema_object_to_javascript_arrays(schema: &Schema) -> JavaScriptType {
    if let Some(x) = schema.items.as_ref() {
        match x.as_ref() {
            ObjectOrReference::Object(o) => parse_schema_object_to_javascript_type(o),
            ObjectOrReference::Ref(s) => JavaScriptType::Typename(parse_reference(&s.ref_path)),
        }
    } else {
        panic!("Unable to convert schema to javascript array")
    }
}

fn parse_schema_object_to_javascript_strings(schema: &Schema) -> JavaScriptType {
    if let Some(enums) = schema.enum_values.as_ref() {
        JavaScriptType::Sum(
            enums
                .iter()
                .map(|v| JavaScriptType::Value(Box::new(parse_json_value_to_javascript_type(v))))
                .collect(),
        )
    } else {
        // TODO(hbina): Handle the case for enums
        if let Some(format) = schema.format.as_ref() {
            match format.as_str() {
                "date" | "date-time" => JavaScriptType::typename("Date"),
                _ => JavaScriptType::typename("string"),
            }
        } else {
            JavaScriptType::typename("string")
        }
    }
}

fn parse_schema_object_to_javascript_row_triplets(schema: &Schema) -> HashMap<String, ObjectRow> {
    // 1. Find the required properties.
    // 2. Iterate through properties.
    // 3. Parse each rows type, creating a triplet of (name, required, type)
    if let Some(properties) = schema.properties.as_ref() {
        let required = schema.required.as_ref();
        let result = properties
            .iter()
            .map(|(name, object)| {
                let name = name.to_string();
                let row_required = required.map(|r| r.contains(&name)).unwrap_or(false);
                let ttype = match object {
                    ObjectOrReference::Ref(r) => {
                        JavaScriptType::Typename(parse_reference(&r.ref_path))
                    }
                    ObjectOrReference::Object(o) => parse_schema_object_to_javascript_type(o),
                };
                (name.clone(), ObjectRow::from_data(row_required, ttype))
            })
            .collect::<HashMap<_, _>>();
        result
    } else {
        HashMap::new()
    }
}

fn parse_schema_object_to_enum_values(schema: &Schema) -> Vec<JavaScriptType> {
    if let Some(values) = schema.enum_values.as_ref() {
        values
            .iter()
            .map(|v| JavaScriptType::Value(Box::new(parse_json_value_to_javascript_type(v))))
            .collect()
    } else {
        vec![]
    }
}

// TODO(hbina): Reimplement this to return an intermediate object so we can log the transformation.
pub fn parse_schema_object_to_javascript_type(schema: &Schema) -> JavaScriptType {
    // 1. Determine the schema type
    // 2. Call the corresponding functions

    if let Some(ty) = schema.schema_type.as_ref() {
        match ty.as_str() {
            "array" => {
                JavaScriptType::Array(Box::new(parse_schema_object_to_javascript_arrays(schema)))
            }
            "string" => parse_schema_object_to_javascript_strings(schema),
            "object" => JavaScriptType::AnonymousObject(
                parse_schema_object_to_javascript_row_triplets(schema),
            ),
            // TODO(hbina): Narrow down the exact type later.
            "integer" | "number" => JavaScriptType::typename("number"),
            "boolean" => JavaScriptType::typename("boolean"),
            "unknown" => JavaScriptType::typename("unknown"),
            "enum" => JavaScriptType::Sum(parse_schema_object_to_enum_values(schema)),
            // TODO(hbina): It is entirely possile type of a schema object to just be a string.
            // I should think.
            // Actually, this case should not even be possible because `types` can take a limited set of values.
            // Rework `openapi` to make this unrepresentable.
            _ => unimplemented!(
                r##"attempting to parse schema with unknown type. schema:{:#?}"##,
                schema
            ),
        }
    } else {
        // TODO(hbina): Revisit this case.
        // The specification does not say anything about the absent of this value.
        // It might be inherited from JSON SchemaObject. Look it up.
        JavaScriptType::typename("any")
    }
}

pub fn parse_root_schema(
    (name, schema): (&String, &ObjectOrReference<Schema>),
) -> (String, JavaScriptType) {
    let name = name.to_string();
    let ttype = match schema {
        ObjectOrReference::Ref(s) => JavaScriptType::Typename(parse_reference(&s.ref_path)),
        ObjectOrReference::Object(v) => parse_schema_object_to_javascript_type(v),
    };
    (name, ttype)
}

pub fn use_spec3(spec: &Spec3) -> Vec<(String, JavaScriptType)> {
    if let Some(components) = spec.components.as_ref() {
        if let Some(schemas) = components.schemas.as_ref() {
            schemas.iter().map(parse_root_schema).collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    }
}

const INVALID_KEY_TYPE_ERROR : &str = "Although YAML technically support having non-string keys. Only strings are valid keys in a JavaScript object.";

fn parse_json_value_to_javascript_type(v: &serde_yaml::Value) -> JavaScriptValue {
    match v {
        serde_yaml::Value::Null => JavaScriptValue::Null,
        serde_yaml::Value::Bool(b) => JavaScriptValue::from(b),
        serde_yaml::Value::Number(n) => JavaScriptValue::from(n.as_f64().unwrap()),
        serde_yaml::Value::String(s) => JavaScriptValue::from(s.as_str()),
        serde_yaml::Value::Sequence(v) => {
            JavaScriptValue::Array(v.iter().map(parse_json_value_to_javascript_type).collect())
        }
        serde_yaml::Value::Mapping(o) => JavaScriptValue::Object(
            o.iter()
                .filter_map(|(k, v)| match k {
                    serde_yaml::Value::String(s) => Some((s, v)),
                    _ => {
                        eprintln!("error:\n{}value:\n{:#?}", INVALID_KEY_TYPE_ERROR, k);
                        return None;
                    }
                })
                .map(|(name, value)| (name.clone(), parse_json_value_to_javascript_type(value)))
                .collect(),
        ),
    }
}
