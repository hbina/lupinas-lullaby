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
                            "{} : {} ",
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
                        "{{\n{}\n}}",
                        o.iter()
                            .map(|r| { format!("\t{};", r) })
                            .collect::<Vec<String>>()
                            .join("\n")
                    )
                }
                JavaScriptType::Product(p) => {
                    p.iter()
                        .map(|x| format!("{}", x))
                        .collect::<Vec<String>>()
                        .join(" & ")
                }
                JavaScriptType::Sum(names) => {
                    names
                        .iter()
                        .map(|s| format!("{}", s))
                        .collect::<Vec<String>>()
                        .join(" | ")
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
            "\"{}\" {} : {}",
            self.name,
            if self.required { "" } else { "?" },
            self.ttype
        )
    }
}
