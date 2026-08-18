//! In-memory representation of one database and its relations.

use crate::relation::Relation;

pub struct Database<'a> {
    name: String,
    relations: Vec<Relation<'a>>,
}
impl<'a> Database<'a> {
    /// Creates an empty database catalog entry.
    pub fn new(name: String) -> Self {
        Database {
            name: String::from(name),
            relations: Vec::new(),
        }
    }
    /// Replaces all relations, mainly used when dropping or loading tables.
    pub fn set_relations(&mut self, relations: Vec<Relation<'a>>) {
        self.relations = relations;
    }
    /// Returns all relations without allowing mutation.
    pub fn get_relations(&self) -> &Vec<Relation<'a>> {
        return &self.relations;
    }
    /// Returns all relations with mutation access for inserts and table changes.
    pub fn get_relations_mut(&mut self) -> &mut Vec<Relation<'a>> {
        &mut self.relations
    }
    /// Returns the database name.
    pub fn get_name(&self) -> &str {
        return &self.name;
    }
    /// Adds a table relation to the database.
    pub fn add_relation(&mut self, relation: Relation<'a>) {
        self.relations.push(relation);
    }
    /// Removes one relation by table name.
    pub fn remove_relation(&mut self, relation: &str) {
        if let Some(index) = self.relations.iter().position(|r| r.get_name() == relation) {
            self.relations.swap_remove(index);
        }
    }
}
