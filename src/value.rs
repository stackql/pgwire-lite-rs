use std::fmt;

/// Value represents a PostgreSQL value from a query result.
#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "NULL"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "{}", s),
            Value::Bytes(b) => write!(f, "{:?}", b),
        }
    }
}

// Default implementation for Value is Null
impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

// Implement From traits for common types
impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Integer(i)
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Value::Integer(i as i64)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Float(f)
    }
}

impl From<Vec<u8>> for Value {
    fn from(b: Vec<u8>) -> Self {
        Value::Bytes(b)
    }
}

// Try-conversion traits for getting values out
impl Value {
    /// Try to get a string value
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get a bool value
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            Value::String(s) => match s.to_lowercase().as_str() {
                "true" | "t" | "yes" | "y" | "1" => Some(true),
                "false" | "f" | "no" | "n" | "0" => Some(false),
                _ => None,
            },
            _ => None,
        }
    }

    /// Try to get an integer value
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Integer(i) => Some(*i),
            Value::Float(f) => Some(*f as i64),
            Value::String(s) => s.parse::<i64>().ok(),
            _ => None,
        }
    }

    /// Try to get a float value
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Integer(i) => Some(*i as f64),
            Value::String(s) => s.parse::<f64>().ok(),
            _ => None,
        }
    }

    /// Check if value is null
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}
