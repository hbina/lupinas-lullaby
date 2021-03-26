use std::{collections::HashMap, ops::Not};

#[derive(Debug, Clone)]
pub enum JavaScriptValue {
    Null,
    String(String),
    Boolean(bool),
    Number(f64),
    Array(Vec<JavaScriptValue>),
    Object(HashMap<String, JavaScriptValue>),
}

impl PartialEq for JavaScriptValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (JavaScriptValue::Array(l), JavaScriptValue::Array(r)) => l.eq(r),
            (JavaScriptValue::Boolean(l), JavaScriptValue::Boolean(r)) => l.eq(r),
            (JavaScriptValue::Null, JavaScriptValue::Null) => true,
            (JavaScriptValue::Number(l), JavaScriptValue::Number(r)) => l.eq(r),
            (JavaScriptValue::String(l), JavaScriptValue::String(r)) => l.eq(r),
            (JavaScriptValue::Object(l), JavaScriptValue::Object(r)) => l
                .iter()
                .zip(r.iter())
                .all(|((l1, l2), (r1, r2))| l1.eq(r1) && l2.eq(r2)),
            _ => false,
        }
    }
}

impl Eq for JavaScriptValue {}

impl From<&str> for JavaScriptValue {
    fn from(v: &str) -> Self {
        JavaScriptValue::String(v.to_string())
    }
}

impl From<bool> for JavaScriptValue {
    fn from(v: bool) -> Self {
        JavaScriptValue::Boolean(v)
    }
}

impl From<&bool> for JavaScriptValue {
    fn from(v: &bool) -> Self {
        JavaScriptValue::Boolean(*v)
    }
}

impl From<f64> for JavaScriptValue {
    fn from(v: f64) -> Self {
        JavaScriptValue::Number(v)
    }
}

impl From<&f64> for JavaScriptValue {
    fn from(v: &f64) -> Self {
        JavaScriptValue::Number(*v)
    }
}

impl From<&JavaScriptValue> for String {
    fn from(v: &JavaScriptValue) -> Self {
        match v {
            JavaScriptValue::Null => "null".to_string(),
            JavaScriptValue::String(s) => {
                format!("'{}'", s)
            }
            JavaScriptValue::Boolean(b) => if *b { "true" } else { "false" }.to_string(),
            JavaScriptValue::Number(n) => {
                format!("{}", n)
            }
            JavaScriptValue::Array(v) => {
                format!(
                    "[{}]",
                    v.iter()
                        .map(String::from)
                        .collect::<Vec<String>>()
                        .join(",")
                )
            }
            JavaScriptValue::Object(o) => {
                format!(
                    "{{{}}}",
                    o.iter()
                        .map(|(name, value)| format!(
                            "\"{}\":{}",
                            String::from(name),
                            String::from(value)
                        ))
                        .collect::<Vec<String>>()
                        .join(",")
                )
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum JavaScriptType {
    Array(Box<JavaScriptType>),
    Product(Vec<JavaScriptType>),
    Sum(Vec<JavaScriptType>),
    Typename(String),
    AnonymousObject(HashMap<String, ObjectRow>),
    Value(Box<JavaScriptValue>),
}

impl JavaScriptType {
    pub fn typename<T: Into<String>>(str: T) -> JavaScriptType {
        JavaScriptType::Typename(str.into())
    }
}

impl std::fmt::Display for JavaScriptType {
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
                        "{{\n{}\n}}",
                        o.iter()
                            .map(|(k, v)| {
                                format!(
                                    "\t{} {} : {};",
                                    k,
                                    if v.required { "" } else { "?" },
                                    v.ttype
                                )
                            })
                            .collect::<Vec<String>>()
                            .join("\n")
                    )
                }
                JavaScriptType::Product(p) => {
                    p.iter()
                        .map(|x| format!("{}", x))
                        .collect::<Vec<String>>()
                        .join("&")
                }
                JavaScriptType::Sum(s) => {
                    s.iter()
                        .map(|s| format!("{}", s))
                        .collect::<Vec<String>>()
                        .join("|")
                }
                JavaScriptType::Typename(o) => o.to_string(),
                JavaScriptType::Value(v) => {
                    String::from(v.as_ref())
                }
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct ObjectRow {
    pub required: bool,
    pub ttype: JavaScriptType,
}

impl ObjectRow {
    pub fn from_data(required: bool, ttype: JavaScriptType) -> ObjectRow {
        ObjectRow { required, ttype }
    }
}

pub fn filter_empty_types(tt: &JavaScriptType) -> Option<JavaScriptType> {
    match tt {
        JavaScriptType::Array(t) => {
            filter_empty_types(&*t).map(|t| JavaScriptType::Array(Box::new(t)))
        }
        JavaScriptType::Product(p) => {
            let result = p
                .iter()
                .filter_map(|v| filter_empty_types(v))
                .collect::<Vec<JavaScriptType>>();
            if result.is_empty() {
                None
            } else {
                Some(JavaScriptType::Product(result))
            }
        }
        JavaScriptType::Sum(s) => {
            let result = s
                .iter()
                .filter_map(|v| filter_empty_types(v))
                .collect::<Vec<JavaScriptType>>();
            if result.is_empty() {
                None
            } else {
                Some(JavaScriptType::Sum(result))
            }
        }
        JavaScriptType::AnonymousObject(o) => {
            let result = o
                .iter()
                .filter_map(|(k, v)| {
                    filter_empty_types(&v.ttype)
                        .map(|tt| (k.clone(), ObjectRow::from_data(v.required, tt)))
                })
                .collect::<HashMap<_, _>>();
            if result.is_empty() {
                None
            } else {
                Some(JavaScriptType::AnonymousObject(result))
            }
        }
        JavaScriptType::Value(v) => Some(JavaScriptType::Value(v.clone())),
        JavaScriptType::Typename(t) => Some(JavaScriptType::Typename(t.clone())),
    }
}

pub fn filter_unwanted_types(tt: &JavaScriptType, skip_types: &[&str]) -> Option<JavaScriptType> {
    match tt {
        JavaScriptType::Array(t) => {
            filter_unwanted_types(&*t, skip_types).map(|t| JavaScriptType::Array(Box::new(t)))
        }
        JavaScriptType::Product(p) => {
            let result = p
                .iter()
                .cloned()
                .filter_map(|v| filter_unwanted_types(&v, skip_types))
                .collect::<Vec<JavaScriptType>>();
            if result.is_empty() {
                None
            } else {
                Some(JavaScriptType::Product(result))
            }
        }
        JavaScriptType::Sum(s) => {
            let result = s
                .iter()
                .cloned()
                .filter_map(|v| filter_unwanted_types(&v, skip_types))
                .collect::<Vec<JavaScriptType>>();
            if result.is_empty() {
                None
            } else {
                Some(JavaScriptType::Sum(result))
            }
        }
        JavaScriptType::AnonymousObject(o) => {
            let result = o
                .iter()
                .filter_map(|(k, v)| {
                    filter_unwanted_types(&v.ttype, skip_types)
                        .map(|tt| (k.clone(), ObjectRow::from_data(v.required, tt)))
                })
                .collect::<HashMap<_, _>>();
            if result.is_empty() {
                None
            } else {
                Some(JavaScriptType::AnonymousObject(result))
            }
        }
        JavaScriptType::Value(v) => Some(JavaScriptType::Value(v.clone())),
        JavaScriptType::Typename(t) => skip_types
            .contains(&t.as_str())
            .not()
            .then(|| JavaScriptType::Typename(t.clone())),
    }
}
