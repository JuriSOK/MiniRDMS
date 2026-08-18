//! Tuple container used by relations and query operators.

#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    record_tuple: Vec<String>,
}

impl Record {
    /// Creates a record from ordered string values.
    pub fn new(record_tuple: Vec<String>) -> Self {
        Self { record_tuple }
    }

    /// Returns a copy of all field values.
    pub fn get_tuple(&self) -> Vec<String> {
        return self.record_tuple.clone();
    }

    /// Replaces all field values after decoding a record from a page.
    pub fn set_tuple(&mut self, tuple: Vec<String>) {
        self.record_tuple = tuple;
    }

    /// Returns one field by column index.
    pub fn get_value(&self, index: usize) -> &String {
        &self.record_tuple[index]
    }
}
