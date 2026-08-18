#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    record_tuple: Vec<String>,
}

impl Record {
    pub fn new(record_tuple: Vec<String>) -> Self {
        Self { record_tuple }
    }

    pub fn get_tuple(&self) -> Vec<String> {
        return self.record_tuple.clone();
    }

    pub fn set_tuple(&mut self, tuple: Vec<String>) {
        self.record_tuple = tuple;
    }

    pub fn get_value(&self, index: usize) -> &String {
        &self.record_tuple[index]
    }
}
