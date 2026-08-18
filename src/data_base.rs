use crate::relation::Relation;

pub struct Database<'a> {
    name: String,
    relations: Vec<Relation<'a>>,
}
impl<'a> Database<'a> {
    pub fn new(name: String) -> Self {
        Database {
            name: String::from(name),
            relations: Vec::new(),
        }
    }
    pub fn set_relations(&mut self, relations: Vec<Relation<'a>>) {
        self.relations = relations;
    }
    pub fn get_relations(&self) -> &Vec<Relation<'a>> {
        return &self.relations;
    }
    pub fn get_relations_mut(&mut self) -> &mut Vec<Relation<'a>> {
        &mut self.relations
    }
    pub fn get_name(&self) -> &str {
        return &self.name;
    }
    pub fn add_relation(&mut self, relation: Relation<'a>) {
        self.relations.push(relation);
    }
    pub fn remove_relation(&mut self, relation: &str) {
        if let Some(index) = self.relations.iter().position(|r| r.get_name() == relation) {
            self.relations.swap_remove(index);
        }
    }
}
