use std::collections::HashMap;

use super::spec::{Schema, Spec2};
use crate::repr::{JavaScriptType, JavaScriptValue, ObjectRow};

// TODO: Validate type at root is object?
pub fn convert_schema_to_js_object_rows(schema: &Schema) -> HashMap<String, ObjectRow> {
    if let Some(properties) = schema.properties.as_ref() {
        let required_names = schema.required.as_ref();
        let properties = properties
            .iter()
            .map(|(name, schema)| {
                let required = required_names.map(|x| x.contains(&name)).unwrap_or(false);
                let ttype = convert_schema_type_to_js_type(schema);
                (name.clone(), ObjectRow::from_data(required, ttype))
            })
            .collect::<HashMap<_, ObjectRow>>();
        properties
    } else {
        HashMap::new()
    }
}

pub fn convert_schema_type_to_js_type(schema: &Schema) -> JavaScriptType {
    if let Some(r) = schema.ref_path.as_ref() {
        JavaScriptType::Typename(parse_reference(r))
    } else if let Some(all_of) = schema.all_of.as_ref() {
        JavaScriptType::Product(
            all_of
                .iter()
                .map(convert_schema_type_to_js_type)
                .collect::<Vec<_>>(),
        )
    } else if let Some(ttype) = schema.schema_type.as_ref() {
        match ttype.as_str() {
            "integer" | "number" => JavaScriptType::typename("number"),
            "string" => {
                if let Some(enums) = schema.enum_values.as_ref() {
                    JavaScriptType::Sum(
                        enums
                            .iter()
                            .map(|v| {
                                JavaScriptType::Value(Box::new(JavaScriptValue::String(
                                    v.to_string(),
                                )))
                            })
                            .collect(),
                    )
                } else if let Some(format) = schema.format.as_ref() {
                    match format.as_str() {
                        "date-time" => JavaScriptType::typename("Date"),
                        _ => JavaScriptType::typename("string"),
                    }
                } else {
                    JavaScriptType::typename("string")
                }
            }
            "boolean" => JavaScriptType::typename("boolean"),
            "array" => match schema.items.as_ref() {
                Some(child_schema) => {
                    JavaScriptType::Array(Box::new(convert_schema_type_to_js_type(child_schema)))
                }
                None => JavaScriptType::typename("any"),
            },
            "object" => JavaScriptType::AnonymousObject(convert_schema_to_js_object_rows(schema)),
            _ => JavaScriptType::typename("any"),
        }
    } else {
        JavaScriptType::AnonymousObject(HashMap::new())
    }
}

pub fn parse_schema((name, schema): (&String, &Schema)) -> (String, JavaScriptType) {
    let name = name.to_string();
    let ttype = convert_schema_type_to_js_type(schema);
    (name, ttype)
}

pub fn parse_reference(reference: &str) -> String {
    let (prefix, name) = reference.split_at("#/definitions/".len());
    if prefix == "#/definitions/" {
        String::from(name)
    } else {
        let result = reference.to_string();
        panic!("Unable to parse reference:'{}'", result);
    }
}

pub fn use_spec2(spec: &Spec2) -> Vec<(String, JavaScriptType)> {
    match spec.definitions.as_ref() {
        Some(definitions) => definitions.iter().map(parse_schema).collect(),
        None => vec![],
    }
}
