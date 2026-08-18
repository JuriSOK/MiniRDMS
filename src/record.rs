#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    //le tuple de la ligne, représenté par un vecteur de chaines de caractères
    record_tuple: Vec<String>,
}

impl Record {
    //constructeur
    pub fn new(record_tuple: Vec<String>) -> Self {
        Self { record_tuple }
    }

    //get, indispensable si on veut récupérer le tuple dans un module externe
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
