//! Column metadata stored in table schemas.

#[derive(Debug, Clone)]
pub struct ColInfo {
    name: String,
    column_type: String,
}
impl ColInfo {
    /// Creates a column definition with a name and SQL-like type string.
    pub fn new(name: String, column_type: String) -> Self {
        ColInfo {
            name: String::from(name),
            column_type: String::from(column_type),
        }
    }

    /// Returns the column name used by records and SELECT projection.
    pub fn get_name(&self) -> &String {
        &self.name
    }

    /// Returns the persisted type descriptor, such as `INT` or `VARCHAR(20)`.
    pub fn get_column_type(&self) -> &String {
        &self.column_type
    }
}
