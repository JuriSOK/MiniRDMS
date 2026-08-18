use crate::col_info::ColInfo;
use crate::condition::Condition;
use crate::record::Record;
use crate::select::Select;
use once_cell::sync::Lazy;
use std::rc::Rc;

pub trait IRecordIterator {
    fn get_next_record(&mut self) -> Option<Record>;
    fn close(&mut self);
    fn reset(&mut self);
}

pub struct RelationScanner {
    records: Vec<Record>, // Liste des enregistrements de la relation
    current_index: usize, // Index courant dans la liste
}

impl RelationScanner {
    pub fn new(records: Vec<Record>) -> Self {
        RelationScanner {
            records,
            current_index: 0,
        }
    }
}

impl IRecordIterator for RelationScanner {
    fn get_next_record(&mut self) -> Option<Record> {
        if self.current_index < self.records.len() {
            let record = self.records[self.current_index].clone();
            self.current_index += 1;
            Some(record)
        } else {
            None
        }
    }

    fn close(&mut self) {
        // Pas de ressources spécifiques à libérer dans cette version simplifiée
    }

    fn reset(&mut self) {
        self.current_index = 0; // Réinitialise l'index pour repartir du début
    }
}

pub struct SelectOperator {
    select: Select,                           // Conditions de sélection
    child_iterator: Box<dyn IRecordIterator>, // L'opérateur fils (RelationScanner)
    col_info: Rc<Vec<ColInfo>>,               // Les informations des colonnes
}

impl IRecordIterator for SelectOperator {
    fn get_next_record(&mut self) -> Option<Record> {
        /*
        Explication du loop : Sans loop, si le le tuple renvoyer par get_next_record() ne passe pas les conditions,
        on ne peut renvoyer un None, car le None signifie qu'on a plus de tuple donnée par le RelationScanner.
        Cela casserait donc les itérations.

        Le loop, permet donc de toujours renvoyer au moins 1 tuple qui passe la condition, si ce n'est pas le cas, on a juste plus de tuple.
         */
        loop {
            // Récupérer le prochain enregistrement de l'opérateur fils
            if let Some(record) = self.child_iterator.get_next_record() {
                // Appliquer les conditions de sélection à l'enregistrement
                if self.evaluate_conditions(&record) {
                    return Some(record); // Si l'enregistrement est valide, le retourner
                }
            } else {
                return None; // Si aucun enregistrement, terminer
            }
        }
    }

    fn close(&mut self) {
        // Fermer l'opérateur fils (peut être un fichier, une connexion, etc.)
        self.child_iterator.close();
    }

    fn reset(&mut self) {
        // Réinitialiser l'état de l'opérateur (recommencer l'itération depuis le début)
        self.child_iterator.reset();
    }
}

impl SelectOperator {
    // Crée une nouvelle instance de SelectOperator
    pub fn new(
        select: Select,
        child_iterator: Box<dyn IRecordIterator>,
        col_info: Rc<Vec<ColInfo>>,
    ) -> Self {
        SelectOperator {
            select,
            child_iterator,
            col_info,
        }
    }

    // Applique les conditions de sélection sur l'enregistrement
    fn evaluate_conditions(&self, record: &Record) -> bool {
        let liste_conditions: &Result<Vec<Condition>, String> =
            &self.select.get_list_conditions(&self.col_info, record);

        if liste_conditions.is_err() {
            return false;
        }
        let liste_conditions = liste_conditions.as_ref().unwrap();
        for condition in liste_conditions {
            // Passer les informations des colonnes à chaque condition pour l'évaluation
            if !condition.evaluate() {
                return false; // Si une condition échoue, l'enregistrement est rejeté
            }
        }
        true // Si toutes les conditions sont satisfaites
    }
}

pub struct ProjectionOperator {
    columns_to_project: Vec<String>, // Liste des colonnes à garder
    child_iterator: Box<dyn IRecordIterator>, // L'opérateur fils (par exemple, SelectOperator)
    col_info: Rc<Vec<ColInfo>>,      // Les informations des colonnes
}

impl ProjectionOperator {
    // Crée un opérateur de projection
    pub fn new(
        columns_to_project: Vec<String>,
        child_iterator: Box<dyn IRecordIterator>,
        col_info: Rc<Vec<ColInfo>>,
    ) -> Self {
        ProjectionOperator {
            columns_to_project,
            child_iterator,
            col_info,
        }
    }

    // Applique la projection sur un enregistrement
    fn project_columns(&self, record: &Record) -> Record {
        let mut projected_tuple = Vec::new(); // Un nouvel enregistrement pour les colonnes projetées

        for col_name in &self.columns_to_project {
            // Récupérer l'index de la colonne via les informations des colonnes
            if let Some(index) = self
                .col_info
                .iter()
                .position(|col| col.get_name() == col_name)
            {
                // Utiliser l'index pour obtenir la valeur de la colonne dans le record
                projected_tuple.push(record.get_value(index).clone());
            }
        }

        Record::new(projected_tuple)
    }
}

impl IRecordIterator for ProjectionOperator {
    fn get_next_record(&mut self) -> Option<Record> {
        if let Some(record) = self.child_iterator.get_next_record() {
            Some(self.project_columns(&record)) // Projeter les colonnes et retourner le nouvel enregistrement
        } else {
            None
        }
    }

    fn close(&mut self) {
        self.child_iterator.close();
    }

    fn reset(&mut self) {
        self.child_iterator.reset();
    }
}

pub struct RecordPrinter<'a> {
    iterator: Box<dyn IRecordIterator + 'a>, // L'itérateur qui fournit les enregistrements
    total: usize,
}

pub static mut ERREURS: Lazy<Vec<String>> = Lazy::new(|| Vec::new());

impl<'a> RecordPrinter<'a> {
    // Constructeur de RecordPrinter qui prend un IRecordIterator
    pub fn new(iterator: Box<dyn IRecordIterator + 'a>) -> Self {
        RecordPrinter { iterator, total: 0 }
    }

    // Affiche les enregistrements un par un
    pub fn print_records(&mut self) {
        // Tant qu'il y a un enregistrement à afficher
        while let Some(record) = self.iterator.get_next_record() {
            // Afficher le tuple du record
            self.print_record(&record);
            self.total += 1;
        }

        if (unsafe { ERREURS.len() } > 0) {
            println!("\nError(s): {}", unsafe { ERREURS.get(0).unwrap() });
            unsafe {
                ERREURS.clear();
            }
        } else {
            println!("\nTotal records =  {}", self.total);
        }
    }

    // Méthode pour afficher un enregistrement sous forme de chaîne
    fn print_record(&self, record: &Record) {
        // Afficher les valeurs des colonnes
        let tuple = record.get_tuple();
        //let formatted_record: Vec<String> = tuple.iter().map(|value| format!("{}", value)).collect();
        println!("{}", tuple.join(" ; "));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relation_scanner() {
        let record1 = Record::new(vec!["1".to_string(), "John".to_string()]);
        let record2 = Record::new(vec!["2".to_string(), "Jane".to_string()]);
        let records = vec![record1.clone(), record2.clone()];
        let mut scanner = RelationScanner::new(records);

        // Test de l'itérateur pour obtenir le premier enregistrement
        let result = scanner.get_next_record();
        assert_eq!(
            result,
            Some(record1),
            "Le premier enregistrement devrait être correct."
        );

        // Test de l'itérateur pour obtenir le second enregistrement
        let result = scanner.get_next_record();
        assert_eq!(
            result,
            Some(record2),
            "Le deuxième enregistrement devrait être correct."
        );

        // Test de la fin de l'itérateur
        let result = scanner.get_next_record();
        assert_eq!(result, None, "Il ne devrait plus y avoir d'enregistrement.");
    }

    #[test]

    fn test_record_printer() {
        let record1 = Record::new(vec!["1".to_string(), "John".to_string()]);
        let record2 = Record::new(vec!["2".to_string(), "Jane".to_string()]);
        let records = vec![record1.clone(), record2.clone()];
        let col_info = Rc::new(vec![
            ColInfo::new("id".to_string(), "INT".to_string()),
            ColInfo::new("name".to_string(), "VARCHAR".to_string()),
        ]);

        let scanner = Box::new(RelationScanner::new(records));
        let projection_operator = ProjectionOperator::new(
            vec!["id".to_string(), "name".to_string()],
            scanner,
            col_info,
        );
        let mut printer = RecordPrinter::new(Box::new(projection_operator));

        // On ne peut pas vérifier directement avec println! sans capture du résultat.
        // Pour un test d'affichage, il faudrait rediriger la sortie stdout.
        printer.print_records(); // On s'assure que la méthode fonctionne sans erreurs
    }
}
