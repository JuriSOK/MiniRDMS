//use std::borrow::Borrow;
use crate::relation::Relation;

pub struct Database<'a> {
    nom: String,
    relations: Vec<Relation<'a>>,
}
impl<'a> Database<'a> {
    pub fn new(name: String) -> Self {
        Database {
            nom: String::from(name),
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
    pub fn get_nom(&self) -> &str {
        return &self.nom;
    }
    pub fn add_relation(&mut self, relation: Relation<'a>) {
        self.relations.push(relation);
    }
    pub fn remove_relation(&mut self, relation: &str) {
        if let Some(index) = self.relations.iter().position(|r| r.get_name() == relation) {
            self.relations.swap_remove(index); //complexité en O(1), met le dernier element à la place de l'element qu'on veut supprimer
        }
    }
}
