use super::spec::{Schema, Spec2};
use std::fmt::Display;

// TODO: Validate type at root is object?
pub fn convert_schema_to_anonymous_object(schema: &Schema) -> JavaScriptType {
    if let Some(properties) = schema.properties.as_ref() {
        let required_names = schema.required.as_ref();
        let properties = properties
            .iter()
            .map(|(name, schema)| {
                let required = required_names.map(|x| x.contains(&name)).unwrap_or(false);
                let ttype = convert_schema_type_to_javascript_type(schema);
                RowTriplet {
                    name: String::from(name),
                    required,
                    ttype,
                }
            })
            .collect::<Vec<RowTriplet>>();
        JavaScriptType::AnonymousObject(properties)
    } else {
        JavaScriptType::AnonymousObject(vec![])
    }
}

pub fn convert_schema_type_to_javascript_type(schema: &Schema) -> JavaScriptType {
    if let Some(r) = schema.ref_path.as_ref() {
        JavaScriptType::Typename(parse_reference(r))
    } else if let Some(all_of) = schema.all_of.as_ref() {
        JavaScriptType::Product(
            all_of
                .iter()
                .map(convert_schema_type_to_javascript_type)
                .collect::<Vec<_>>(),
        )
    } else if let Some(ttype) = schema.schema_type.as_ref() {
        match ttype.as_str() {
            "integer" | "number" => JavaScriptType::typename("number"),
            "string" => {
                if let Some(enums) = schema.enum_values.as_ref() {
                    JavaScriptType::Sum(enums.clone())
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
                Some(child_schema) => JavaScriptType::Array(Box::new(
                    convert_schema_type_to_javascript_type(child_schema),
                )),
                None => JavaScriptType::typename("any"),
            },
            "object" => convert_schema_to_anonymous_object(schema),
            _ => JavaScriptType::typename("any"),
        }
    } else {
        JavaScriptType::AnonymousObject(vec![])
    }
}

pub fn parse_root_schema_object_to_javascript_construct(
    (name, schema): (&String, &Schema),
) -> String {
    let name = name.to_string();
    let ttype = convert_schema_type_to_javascript_type(schema);
    format!("export type {} = {}", name, ttype)
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
            .map(parse_root_schema_object_to_javascript_construct)
            .collect::<Vec<String>>()
            .join("\n"),
        None => String::new(),
    }
}

#[derive(Debug, Clone)]
pub enum JavaScriptType {
    Array(Box<JavaScriptType>),
    Product(Vec<JavaScriptType>),
    Sum(Vec<String>),
    Typename(String),
    AnonymousObject(Vec<RowTriplet>),
}

impl JavaScriptType {
    pub fn typename<T: Into<String>>(str: T) -> JavaScriptType {
        JavaScriptType::Typename(str.into())
    }
}

impl Display for JavaScriptType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                JavaScriptType::Array(o) => {
                    format!("{}[]", o)
                }
                JavaScriptType::AnonymousObject(o) => {
                    format!(
                        "{{{}}}",
                        o.iter().map(|r| { format!("{};", r) }).collect::<String>()
                    )
                }
                JavaScriptType::Product(p) => {
                    p.iter()
                        .map(|x| format!("{}", x))
                        .collect::<Vec<String>>()
                        .join("&")
                }
                JavaScriptType::Sum(names) => {
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
                JavaScriptType::Typename(o) => o.to_string(),
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct RowTriplet {
    name: String,
    required: bool,
    ttype: JavaScriptType,
}

impl Display for RowTriplet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\"{}\" {} : {}",
            self.name,
            if self.required { "" } else { "?" },
            self.ttype
        )
    }
}

#[derive(Debug)]
pub enum ParseError {
    InvalidReference(String),
}
