use super::spec::{Schema, Spec2};
use std::fmt::Display;

// TODO: Validate type at root is object?
pub fn convert_schema_to_interface((name, schema): (&String, &Schema)) -> String {
    let properties = schema
        .properties
        .as_ref()
        .map(|x| x.iter().map(convert_property_to_member).collect::<Vec<_>>())
        .unwrap_or_else(|| vec![]);
    let members = properties
        .iter()
        .map(|(name, js_type)| {
            let required = schema
                .required
                .as_ref()
                .map(|y| y.contains(name))
                .map(|y| if y { "?" } else { "" })
                .unwrap_or("");
            format!("\t{} {} : {};\n", name, required, js_type)
        })
        .collect::<String>();
    format!(r##"export interface {} {{{}}}"##, name, members)
}

pub fn convert_property_to_member((name, schema): (&String, &Schema)) -> (String, JavaScriptType) {
    let js_type = convert_schema_type_to_javascript_type(schema);
    (name.clone(), js_type)
}

pub fn convert_schema_type_to_javascript_type(schema: &Schema) -> JavaScriptType {
    if let Some(r) = schema.ref_path.as_ref() {
        JavaScriptType::Object(parse_reference(r))
    } else if let Some(ttype) = schema.schema_type.as_ref() {
        match ttype.as_str() {
            "integer" => JavaScriptType::Number,
            "string" => {
                if let Some(enums) = schema.enum_values.as_ref() {
                    JavaScriptType::Enum(enums.clone())
                } else if let Some(format) = schema.format.as_ref() {
                    match format.as_str() {
                        "date-time" => JavaScriptType::Object("Date".to_string()),
                        _ => JavaScriptType::String,
                    }
                } else {
                    JavaScriptType::String
                }
            }
            "boolean" => JavaScriptType::Boolean,
            "array" => match schema.items.as_ref() {
                Some(child_schema) => convert_schema_type_to_javascript_type(child_schema),
                None => JavaScriptType::Any,
            },
            _ => JavaScriptType::Any,
        }
    } else {
        panic!(
            "Cannot perform schema type => javascript type for schema:{:#?}",
            schema
        );
    }
}

pub fn parse_reference(reference: &str) -> String {
    let (prefix, name) = reference.split_at("#/definitions/".len());
    if prefix == "#/definitions/" {
        String::from(name)
    } else {
        panic!(
            "{:#?}",
            ParseError::InvalidReference(String::from(reference))
        )
    }
}

pub fn use_spec2(spec: &Spec2) -> String {
    match spec.definitions.as_ref() {
        Some(definitions) => definitions
            .iter()
            .map(convert_schema_to_interface)
            .collect::<Vec<_>>()
            .iter()
            .map(|x| x.to_string())
            .collect::<String>(),
        None => String::new(),
    }
}

#[derive(Debug)]
pub enum JavaScriptType {
    Any,
    String,
    Number,
    Boolean,
    Enum(Vec<String>),
    Object(String),
}

impl Display for JavaScriptType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                JavaScriptType::Any => "any".into(),
                JavaScriptType::String => "string".into(),
                JavaScriptType::Number => "number".into(),
                JavaScriptType::Boolean => "boolean".into(),
                JavaScriptType::Enum(names) => {
                    let count = names.iter().len();
                    names
                        .iter()
                        .enumerate()
                        .map(|(idx, s)| {
                            let last_element = count == idx + 1;
                            let div_operator = if last_element { "" } else { "|" };
                            format!("\"{}\" {}", s, div_operator)
                        })
                        .collect::<String>()
                }
                JavaScriptType::Object(o) => o.to_string(),
            }
        )
    }
}

#[derive(Debug)]
pub enum ParseError {
    InvalidReference(String),
}
