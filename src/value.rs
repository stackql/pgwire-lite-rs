// src/value.rs

use std::fmt;

/// Represents a value from a PostgreSQL query result.
///
/// This enum provides type-safe access to various PostgreSQL data types
/// and includes conversion methods for common Rust types.
#[derive(Debug, Clone)]
#[derive(Default)]
pub enum Value {
    #[default]
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
    /// Try to get the value as a string reference.
    ///
    /// Returns `None` if the value is not a String.
    ///
    /// # Example
    ///
    /// ```
    /// use pgwire_lite::Value;
    ///
    /// let val = Value::from("hello");
    /// assert_eq!(val.as_str(), Some("hello"));
    ///
    /// let val = Value::Integer(42);
    /// assert_eq!(val.as_str(), None);
    /// ```
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get the value as a boolean.
    ///
    /// Returns `Some(bool)` if the value is a boolean or a string that can be
    /// interpreted as a boolean (e.g., "true", "yes", "1").
    /// Returns `None` for other types or strings that cannot be parsed as booleans.
    ///
    /// # Example
    ///
    /// ```
    /// use pgwire_lite::Value;
    ///
    /// let val = Value::Bool(true);
    /// assert_eq!(val.as_bool(), Some(true));
    ///
    /// let val = Value::from("yes");
    /// assert_eq!(val.as_bool(), Some(true));
    ///
    /// let val = Value::from("0");
    /// assert_eq!(val.as_bool(), Some(false));
    ///
    /// let val = Value::from("invalid");
    /// assert_eq!(val.as_bool(), None);
    /// ```
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

    /// Try to get the value as a 64-bit signed integer.
    ///
    /// Returns `Some(i64)` if the value is an integer, a float that can be
    /// converted to an integer, or a string that can be parsed as an integer.
    /// Returns `None` for other types or values that cannot be converted.
    ///
    /// # Example
    ///
    /// ```
    /// use pgwire_lite::Value;
    ///
    /// let val = Value::Integer(42);
    /// assert_eq!(val.as_i64(), Some(42));
    ///
    /// let val = Value::Float(42.0);
    /// assert_eq!(val.as_i64(), Some(42));
    ///
    /// let val = Value::from("42");
    /// assert_eq!(val.as_i64(), Some(42));
    ///
    /// let val = Value::from("invalid");
    /// assert_eq!(val.as_i64(), None);
    /// ```
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Integer(i) => Some(*i),
            Value::Float(f) => Some(*f as i64),
            Value::String(s) => s.parse::<i64>().ok(),
            _ => None,
        }
    }

    /// Try to get the value as a 64-bit floating point number.
    ///
    /// Returns `Some(f64)` if the value is a float, an integer, or a string
    /// that can be parsed as a float.
    /// Returns `None` for other types or values that cannot be converted.
    ///
    /// # Example
    ///
    /// ```
    /// use pgwire_lite::Value;
    ///
    /// let val = Value::Float(3.14);
    /// assert_eq!(val.as_f64(), Some(3.14));
    ///
    /// let val = Value::Integer(42);
    /// assert_eq!(val.as_f64(), Some(42.0));
    ///
    /// let val = Value::from("3.14");
    /// assert_eq!(val.as_f64(), Some(3.14));
    ///
    /// let val = Value::from("invalid");
    /// assert_eq!(val.as_f64(), None);
    /// ```
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Integer(i) => Some(*i as f64),
            Value::String(s) => s.parse::<f64>().ok(),
            _ => None,
        }
    }

    /// Check if the value is NULL.
    ///
    /// # Example
    ///
    /// ```
    /// use pgwire_lite::Value;
    ///
    /// let val = Value::Null;
    /// assert!(val.is_null());
    ///
    /// let val = Value::Integer(42);
    /// assert!(!val.is_null());
    /// ```
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}
