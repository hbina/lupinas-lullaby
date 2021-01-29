use super::spec::{Ref, Schema};
use serde_yaml::Value;
use std::fmt::Display;

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
            "{} {} : {}",
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
                        Value::Null => format!("null"),
                        Value::Bool(b) => {
                            if *b {
                                format!("true")
                            } else {
                                format!("false")
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
    CannotConvertSchemaToArray(Schema),
}
