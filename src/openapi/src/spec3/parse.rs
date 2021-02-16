use super::spec::{ObjectOrReference, Ref, Schema, Spec3};
use serde_yaml::Value;
use std::{fmt::Display, unimplemented};

pub fn parse_reference(reference: &Ref) -> String {
    let (prefix, name) = reference.ref_path.split_at("#/components/schemas/".len());
    if prefix == "#/components/schemas/" {
        String::from(name)
    } else {
        panic!("{:#?}", Spec3Error::InvalidReference(reference.clone()))
    }
}

fn parse_schema_object_to_javascript_arrays(schema: &Schema) -> JavaScriptType {
    if let Some(x) = schema.items.as_ref() {
        match x.as_ref() {
            ObjectOrReference::Object(o) => parse_schema_object_to_javascript_type(o),
            ObjectOrReference::Ref(s) => JavaScriptType::Typename(parse_reference(s)),
        }
    } else {
        panic!(
            "{:#?}",
            Spec3Error::CannotConvertSchemaToArray(Box::new(schema.clone()))
        )
    }
}

fn parse_schema_object_to_javascript_strings(schema: &Schema) -> JavaScriptType {
    if let Some(enums) = schema.enum_values.as_ref() {
        JavaScriptType::Sum(enums.clone())
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

fn parse_schema_object_to_javascript_row_triplets(schema: &Schema) -> Vec<RowTriplet> {
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
                    ObjectOrReference::Ref(r) => JavaScriptType::Typename(parse_reference(r)),
                    ObjectOrReference::Object(o) => parse_schema_object_to_javascript_type(o),
                };
                RowTriplet::from_triplet(name, row_required, ttype)
            })
            .collect::<Vec<RowTriplet>>();
        result
    } else {
        vec![]
    }
}

fn parse_schema_object_to_enum_values(schema: &Schema) -> Vec<Value> {
    if let Some(values) = schema.enum_values.as_ref() {
        values.clone()
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

pub fn parse_root_schema_object_to_javascript_construct(
    (name, schema): (&String, &ObjectOrReference<Schema>),
) -> JavaScriptConstruct {
    let name = name.to_string();
    match schema {
        ObjectOrReference::Ref(s) => {
            JavaScriptConstruct::Alias(name, JavaScriptType::Typename(parse_reference(s)))
        }
        ObjectOrReference::Object(v) => {
            let body = parse_schema_object_to_javascript_type(v);
            JavaScriptConstruct::Alias(name, body)
        }
    }
}

pub fn use_spec3(spec: &Spec3) -> String {
    if let Some(components) = spec.components.as_ref() {
        if let Some(schemas) = components.schemas.as_ref() {
            let result = schemas
                .iter()
                .map(parse_root_schema_object_to_javascript_construct)
                .collect::<Vec<_>>()
                .iter()
                .map(|x| format!("{}", x))
                .collect::<String>();
            result
        } else {
            String::new()
        }
    } else {
        String::new()
    }
}

#[derive(Debug)]
pub enum JavaScriptType {
    Sum(Vec<Value>),
    Array(Box<JavaScriptType>),
    Typename(String),
    AnonymousObject(Vec<RowTriplet>),
}

impl JavaScriptType {
    pub fn typename<T: Into<String>>(str: T) -> JavaScriptType {
        JavaScriptType::Typename(str.into())
    }
}

#[derive(Debug)]
pub enum JavaScriptConstruct {
    Alias(String, JavaScriptType),
}

impl Display for JavaScriptConstruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JavaScriptConstruct::Alias(name, ttype) => {
                writeln!(f, "export type {} = {};", name, ttype)
            }
        }
    }
}

#[derive(Debug)]
pub struct RowTriplet {
    name: String,
    required: bool,
    ttype: JavaScriptType,
}

impl RowTriplet {
    pub fn from_triplet(name: String, required: bool, ttype: JavaScriptType) -> RowTriplet {
        RowTriplet {
            name,
            required,
            ttype,
        }
    }
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

impl Display for JavaScriptType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JavaScriptType::Sum(s) => {
                let result = s
                    .iter()
                    .map(|v| match v {
                        Value::Null => "null".to_string(),
                        Value::Bool(b) => {
                            if *b {
                                "true".to_string()
                            } else {
                                "false".to_string()
                            }
                        }
                        Value::Number(n) => {
                            format!("{}", n)
                        }
                        Value::String(s) => {
                            format!("\"{}\"", s)
                        }
                        o => unimplemented!("Using {:#?} is not yet supported in Display", o),
                    })
                    .collect::<Vec<String>>()
                    .join("|");
                write!(f, "{}", result)
            }
            JavaScriptType::Array(a) => {
                write!(f, "({})[]", a)
            }
            JavaScriptType::Typename(n) => {
                write!(f, "{}", n)
            }
            JavaScriptType::AnonymousObject(o) => {
                write!(
                    f,
                    "{{{}}}",
                    o.iter().map(|r| { format!("{};", r) }).collect::<String>()
                )
            }
        }
    }
}

#[derive(Debug)]
pub enum Spec3Error {
    InvalidReference(Ref),
    CannotConvertSchemaToArray(Box<Schema>),
}
