use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Float(f64),
    Int(i64),
    Bool(bool),
    String(String),
}

impl Value {
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(value) => Some(value),
            _ => None,
        }
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Int(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Data {
    Scalar(Value),
    Vector(Vec<Value>),
    Map(HashMap<String, Value>),
    None,
}

impl Data {
    pub fn as_scalar(&self) -> Option<Value> {
        match self {
            Data::Scalar(value) => Some(value.clone()),
            _ => None,
        }
    }

    pub fn as_vector(&self) -> Option<&Vec<Value>> {
        match self {
            Data::Vector(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Data::Map(value) => Some(value),
            _ => None,
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Data::None)
    }
}

pub struct DataSet {
    pub timestamp: u64,
    pub data: HashMap<String, Data>,
}

impl DataSet {
    pub fn new(timestamp: u64) -> Self {
        DataSet {
            timestamp,
            data: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: &str, data: Data) {
        self.data.insert(name.to_string(), data);
    }

    pub fn get(&self, name: &str) -> Option<&Data> {
        self.data.get(name)
    }
}

impl fmt::Debug for DataSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "DataSet {{")?;
        writeln!(f, "    timestamp: {},", self.timestamp)?;
        writeln!(f, "    data: {{")?;
        for (name, result) in &self.data {
            writeln!(f, "        {}: {:?},", name, result)?;
        }
        writeln!(f, "    }}")?;
        writeln!(f, "}}")?;
        Ok(())
    }
}
