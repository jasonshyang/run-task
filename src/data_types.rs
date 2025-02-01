use std::collections::HashMap;
use std::fmt;

#[derive(Clone)]
pub struct DataSet<Output> {
    pub timestamp: u64,
    pub data: HashMap<String, Output>,
}

impl<Output> DataSet<Output> {
    pub fn new(timestamp: u64) -> Self {
        DataSet {
            timestamp,
            data: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: &str, data: Output) {
        self.data.insert(name.to_string(), data);
    }

    pub fn get(&self, name: &str) -> Option<&Output> {
        self.data.get(name)
    }

    pub fn take(&mut self, name: &str) -> Option<Output> {
        self.data.remove(name)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Output)> {
        self.data.iter()
    }

    pub fn into_iter(self) -> impl Iterator<Item = (String, Output)> {
        self.data.into_iter()
    }

    pub fn into_inner(self) -> HashMap<String, Output> {
        self.data
    }

}

impl<Input: fmt::Debug> fmt::Debug for DataSet<Input> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "DataSet @ {}", self.timestamp)?;
        writeln!(f, "├─ Items: {}", self.data.len())?;
        for (key, value) in &self.data {
            writeln!(f, "├─ {}: {:?}", key, value)?;
        }
        Ok(())
    }
}
