use std::collections::HashMap;
use std::fmt;

pub struct DataSet<D> {
    pub timestamp: u64,
    pub data: HashMap<String, D>,
}

impl<D> DataSet<D> {
    pub fn new(timestamp: u64) -> Self {
        DataSet {
            timestamp,
            data: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: &str, data: D) {
        self.data.insert(name.to_string(), data);
    }

    pub fn get(&self, name: &str) -> Option<&D> {
        self.data.get(name)
    }

    pub fn take(&mut self, name: &str) -> Option<D> {
        self.data.remove(name)
    }

}

impl<T: fmt::Debug> fmt::Debug for DataSet<T> {
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
