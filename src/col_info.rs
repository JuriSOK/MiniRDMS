#[derive(Debug, Clone)]
pub struct ColInfo {
    name: String,
    column_type: String,
}
impl ColInfo {
    //Le fait d'utiliser &str au lieu de string dans les arguments permet de pouvoir utiliser des chaînes littérales,
    //Exemple :
    // let column_name = "id";    column_name est de type &str
    // let column_type = "Int";  column_type est aussi de type &str
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
