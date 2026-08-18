#[derive(Debug, Clone)]
pub struct ColInfo {
    name: String,
    column_type: String,
}
impl ColInfo {
    pub fn new(name: String, column_type: String) -> Self {
        ColInfo {
            name: String::from(name),
            column_type: String::from(column_type),
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_column_type(&self) -> &String {
        &self.column_type
    }
}
