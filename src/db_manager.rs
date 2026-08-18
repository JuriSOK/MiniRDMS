use crate::buffer_manager::BufferManager;
use crate::col_info::ColInfo;
use crate::data_base::Database;
use crate::relation::Relation;
use crate::DBConfig;
use crate::PageId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::rc::Rc;

const CURRENT_DATABASE_SUFFIX: &str = "CURRENT_DATABASE";
const LEGACY_CURRENT_DATABASE_SUFFIX: &str = "BDD_COURANTE";

pub struct DBManager<'a> {
    databases: HashMap<String, Database<'a>>,
    dbconfig: &'a DBConfig,
    current_database: Option<String>,
    buffer_manager: Rc<RefCell<BufferManager<'a>>>,
}

impl<'a> DBManager<'a> {
    pub fn new(db: &'a DBConfig, buffer_m: Rc<RefCell<BufferManager<'a>>>) -> Self {
        DBManager {
            databases: HashMap::new(),
            dbconfig: db,
            current_database: None::<String>,
            buffer_manager: buffer_m,
        }
    }

    pub fn get_current_database(&mut self) -> Option<&mut Database<'a>> {
        if self.current_database.is_some() {
            return self
                .databases
                .get_mut(self.current_database.as_ref().unwrap());
        } else {
            return None;
        }
    }

    pub fn get_databases(&self) -> &HashMap<String, Database<'a>> {
        return &self.databases;
    }

    pub fn get_db_config(&self) -> &'a DBConfig {
        return self.dbconfig;
    }

    pub fn create_database(&mut self, db: &str) {
        self.databases
            .insert(db.to_string(), Database::new(db.to_string()));
    }

    pub fn set_current_database(&mut self, name: &str) {
        if self.databases.contains_key(name) {
            self.current_database = Some(name.to_string());
        } else {
            println!("Database {} does not exist.", name);
        }
    }

    pub fn add_table_to_current_database(&mut self, table: Relation<'a>) {
        let name = table.get_name().clone();
        if self.current_database.is_some() {
            self.get_current_database().unwrap().add_relation(table);
        }
        println!("Table {} created.", name);
    }

    pub fn get_table_from_current_database(&mut self, table_name: &str) -> Option<&Relation<'a>> {
        let database = self.get_current_database().unwrap();
        let database_relations = database.get_relations();
        let mut rel_result = None;
        for rel in database_relations.iter() {
            if rel.get_name() == table_name {
                rel_result = Some(rel);
            }
        }
        return rel_result;
    }

    pub fn remove_table_from_current_database(&mut self, table_name: &str) {
        self.get_current_database()
            .unwrap()
            .remove_relation(table_name);
    }

    pub fn remove_database(&mut self, database_name: &str) {
        if let Some(_db) = self.databases.get(database_name) {
            self.databases
                .get_mut(database_name)
                .unwrap()
                .set_relations(Vec::<Relation>::new());
            self.databases.remove(database_name);
            if !self.get_current_database().is_none()
                && self.get_current_database().unwrap().get_name() == database_name
            {
                self.current_database = None;
            }
        }
    }

    pub fn remove_tables_from_current_database(&mut self) {
        match self.get_current_database() {
            Some(_database) => self
                .get_current_database()
                .unwrap()
                .set_relations(Vec::new()),
            _ => println!("No current database."),
        }
    }

    pub fn remove_databases(&mut self) {
        self.current_database = None;
        self.databases.clear();
    }

    pub fn list_databases(&mut self) {
        println!("Databases:");
        match self.get_current_database() {
            Some(_database) => println!(
                "Current database: {}",
                self.get_current_database().unwrap().get_name()
            ),
            _ => println!("Current database: none."),
        }
        for database in self.databases.keys() {
            println!("Database: {}", database)
        }
    }

    pub fn list_tables_in_current_database(&mut self) {
        match self.get_current_database() {
            Some(database) => {
                let relations = database.get_relations();
                if relations.is_empty() {
                    println!("The current database does not contain any tables.");
                    return;
                }

                for rel in relations {
                    println!("Table : {}", rel.get_name());
                    println!("+---------------------------+---------------------------+");
                    println!("| Name                      | Type                      |");
                    println!("+---------------------------+---------------------------+");

                    for col in rel.get_columns() {
                        println!("| {:<25} | {:<25} |", col.get_name(), col.get_column_type());
                    }

                    println!("+---------------------------+---------------------------+\n");
                }
            }
            None => println!("No current database."),
        }
    }

    pub fn save_state(&self) -> Result<(), std::io::Error> {
        let save_file = format!("{}/databases.json", self.dbconfig.get_dbpath());
        let mut snapshot: HashMap<String, Vec<(String, (u32, u32), Vec<String>, Vec<String>)>> =
            HashMap::new();

        for (database_name, database) in &self.databases {
            let mut relations: Vec<(String, (u32, u32), Vec<String>, Vec<String>)> = Vec::new();

            for relation in database.get_relations() {
                let mut columns: Vec<String> = Vec::new();
                let mut types: Vec<String> = Vec::new();

                for col in &relation.get_columns() {
                    columns.push(col.get_name().clone());
                    types.push(col.get_column_type().clone());
                }

                relations.push((
                    relation.get_name().to_string(),
                    (
                        relation.get_header_page_id().get_file_idx(),
                        relation.get_header_page_id().get_page_idx(),
                    ),
                    columns,
                    types,
                ));
            }

            if self.current_database.clone().is_some()
                && self.current_database.clone().unwrap().as_str() == database_name.as_str()
            {
                snapshot.insert([database_name, CURRENT_DATABASE_SUFFIX].join(""), relations);
            } else {
                snapshot.insert(database_name.clone(), relations);
            }
        }

        let json_data = serde_json::to_string_pretty(&snapshot)?;

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(save_file)?;
        file.write_all(json_data.as_bytes())?;
        println!("SAVE STATE OK");
        Ok(())
    }

    pub fn load_state(&mut self) -> Result<(), std::io::Error> {
        let save_file = format!("{}/databases.json", self.dbconfig.get_dbpath());

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(save_file)
            .expect("Could not open or create databases.json");

        let mut json_data = String::new();
        file.read_to_string(&mut json_data)?;

        let snapshot: HashMap<String, Vec<(String, (u32, u32), Vec<String>, Vec<String>)>> =
            serde_json::from_str(&json_data)?;

        for (mut database_name, relations) in snapshot {
            if database_name.contains(CURRENT_DATABASE_SUFFIX) {
                let new_name = database_name
                    .to_string()
                    .drain(..database_name.len() - CURRENT_DATABASE_SUFFIX.len())
                    .collect::<String>();
                database_name = new_name;
                self.current_database = Some(database_name.to_string());
            } else if database_name.contains(LEGACY_CURRENT_DATABASE_SUFFIX) {
                let new_name = database_name
                    .to_string()
                    .drain(..database_name.len() - LEGACY_CURRENT_DATABASE_SUFFIX.len())
                    .collect::<String>();
                database_name = new_name;
                self.current_database = Some(database_name.to_string());
            }

            let mut database = Database::new(database_name.clone());

            for (relation_name, (file_idx, page_idx), columns, types) in relations {
                let mut cols: Vec<ColInfo> = Vec::new();

                for i in 0..columns.len() {
                    cols.push(ColInfo::new(columns[i].clone(), types[i].clone()));
                }

                let header_page_id = PageId::new(file_idx, page_idx);
                let relation = Relation::from_saved(
                    relation_name,
                    cols,
                    header_page_id,
                    self.buffer_manager.clone(),
                );

                database.add_relation(relation);
            }

            self.databases.insert(database_name, database);
        }
        println!("LOAD STATE OK");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer_manager::BufferManager;
    use crate::disk_manager::DiskManager;
    use crate::DBConfig;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_current_database() {
        let s: String = String::from("config.json");
        let config = DBConfig::load_db_config(s);
        let dm = DiskManager::new(&config);
        let lru_policy = String::from("LRU");

        let buffer_manager1 = BufferManager::new(&config, dm, lru_policy);
        let rc_a = Rc::new(RefCell::new(buffer_manager1));

        let column_info1: Vec<ColInfo> = vec![
            ColInfo::new("NOM".to_string(), "CHAR(10)".to_string()),
            ColInfo::new("AGE".to_string(), "INT".to_string()),
            ColInfo::new("PRENOM".to_string(), "VARCHAR(6)".to_string()),
        ];
        let relation1 = Relation::new(
            "PERSONNE".to_string(),
            column_info1.clone(),
            Rc::clone(&rc_a),
        );

        let column_info2: Vec<ColInfo> = vec![
            ColInfo::new("NOM".to_string(), "CHAR(20)".to_string()),
            ColInfo::new("ID".to_string(), "INT".to_string()),
            ColInfo::new("SALAIRE".to_string(), "FLOAT".to_string()),
        ];
        let relation2 = Relation::new("EMPLOI".to_string(), column_info2.clone(), Rc::clone(&rc_a));

        let column_info3: Vec<ColInfo> = vec![
            ColInfo::new("MARQUE".to_string(), "CHAR(20)".to_string()),
            ColInfo::new("MODELE".to_string(), "VARCHAR(10)".to_string()),
            ColInfo::new("ID".to_string(), "INT".to_string()),
            ColInfo::new("PUISSANCE".to_string(), "INT".to_string()),
            ColInfo::new("PRIX".to_string(), "FLOAT".to_string()),
        ];
        let relation3 = Relation::new(
            "VOITURE".to_string(),
            column_info3.clone(),
            Rc::clone(&rc_a),
        );

        let column_info4: Vec<ColInfo> = vec![
            ColInfo::new("MARQUE".to_string(), "CHAR(20)".to_string()),
            ColInfo::new("MODELE".to_string(), "VARCHAR(10)".to_string()),
            ColInfo::new("ID".to_string(), "INT".to_string()),
            ColInfo::new("PUISSANCE".to_string(), "INT".to_string()),
            ColInfo::new("CARBURANT".to_string(), "CHAR(10)".to_string()),
            ColInfo::new("PRIX".to_string(), "FLOAT".to_string()),
        ];
        let relation4 = Relation::new(
            "TRACTEUR".to_string(),
            column_info4.clone(),
            Rc::clone(&rc_a),
        );

        let mut db_manager = DBManager::new(&config, Rc::clone(&rc_a));
        db_manager.create_database("carrefour");
        db_manager.set_current_database("carrefour");
        db_manager.add_table_to_current_database(relation1);
        db_manager.add_table_to_current_database(relation2);

        db_manager.list_databases();
        db_manager.list_tables_in_current_database();

        db_manager.create_database("concession");
        db_manager.set_current_database("concession");
        db_manager.add_table_to_current_database(relation3);
        db_manager.add_table_to_current_database(relation4);

        db_manager.list_databases();
        db_manager.list_tables_in_current_database();

        let _ = db_manager.save_state();
    }

    #[test]
    fn test_save_state_and_load_state() {
        let s: String = String::from("config.json");
        let config = DBConfig::load_db_config(s);
        let dm = DiskManager::new(&config);
        let lru_policy = String::from("LRU");

        let buffer_manager1 = BufferManager::new(&config, dm, lru_policy);
        let rc_a = Rc::new(RefCell::new(buffer_manager1));

        let mut db_manager = DBManager::new(&config, Rc::clone(&rc_a));
        let _ = db_manager.load_state();

        db_manager.set_current_database("concession");
        db_manager.list_databases();
        db_manager.list_tables_in_current_database();
    }
}
