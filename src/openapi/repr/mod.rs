use std::ops::Not;

#[derive(Debug, Clone)]
pub enum JavaScriptValue {
    Null,
    String(String),
    Boolean(bool),
    Number(f64),
    Array(Vec<JavaScriptValue>),
    Object(Vec<(JavaScriptValue, JavaScriptValue)>),
}

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
    AnonymousObject(Vec<RowTriplet>),
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
                        "{{{}}}",
                        o.iter()
                            .map(|r| { format!("{};", r) })
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

impl std::fmt::Display for RowTriplet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\"{}\"{}:{}",
            self.name,
            if self.required { "" } else { "?" },
            self.ttype
        )
    }
}

pub fn filter_empty_types(tt: JavaScriptType) -> Option<JavaScriptType> {
    match tt {
        JavaScriptType::Array(t) => {
            filter_empty_types(*t).map(|t| JavaScriptType::Array(Box::new(t)))
        }
        JavaScriptType::Product(p) => {
            let result = p
                .iter()
                .filter_map(|v| filter_empty_types(v.clone()))
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
                .filter_map(|v| filter_empty_types(v.clone()))
                .collect::<Vec<JavaScriptType>>();
            if result.is_empty() {
                None
            } else {
                Some(JavaScriptType::Sum(result))
            }
        }
        JavaScriptType::AnonymousObject(o) => {
            if o.is_empty() {
                None
            } else {
                Some(JavaScriptType::AnonymousObject(
                    o.into_iter()
                        .filter_map(|v| {
                            filter_empty_types(v.ttype.clone())
                                .map(|tt| RowTriplet::from_triplet(v.name, v.required, tt))
                        })
                        .collect(),
                ))
            }
        }
        JavaScriptType::Value(v) => Some(JavaScriptType::Value(v)),
        JavaScriptType::Typename(t) => Some(JavaScriptType::Typename(t)),
    }
}

pub fn filter_unwanted_types(tt: JavaScriptType, skip_types: &Vec<&str>) -> Option<JavaScriptType> {
    match tt {
        JavaScriptType::Array(t) => {
            filter_unwanted_types(*t, skip_types).map(|t| JavaScriptType::Array(Box::new(t)))
        }
        JavaScriptType::Product(p) => {
            let result = p
                .iter()
                .cloned()
                .filter_map(|v| filter_unwanted_types(v, skip_types))
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
                .filter_map(|v| filter_unwanted_types(v, skip_types))
                .collect::<Vec<JavaScriptType>>();
            if result.is_empty() {
                None
            } else {
                Some(JavaScriptType::Sum(result))
            }
        }
        JavaScriptType::AnonymousObject(o) => {
            if o.is_empty() {
                None
            } else {
                Some(JavaScriptType::AnonymousObject(
                    o.into_iter()
                        .filter_map(|v| {
                            filter_unwanted_types(v.ttype.clone(), skip_types)
                                .map(|tt| RowTriplet::from_triplet(v.name, v.required, tt))
                        })
                        .collect(),
                ))
            }
        }
        JavaScriptType::Value(v) => Some(JavaScriptType::Value(v)),
        JavaScriptType::Typename(t) => skip_types
            .contains(&t.as_str())
            .not()
            .then(|| JavaScriptType::Typename(t)),
    }
}
