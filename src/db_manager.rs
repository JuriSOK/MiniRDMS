use crate::buffer_manager::BufferManager;
use crate::col_info::ColInfo;
use crate::data_base::Database;
use crate::relation::Relation;
use crate::DBConfig;
use crate::PageId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::option::Option;
use std::rc::Rc;
pub struct DBManager<'a> {
    basededonnees: HashMap<String, Database<'a>>,
    dbconfig: &'a DBConfig,
    bdd_courante: Option<String>,
    buffer_manager: Rc<RefCell<BufferManager<'a>>>,
}

impl<'a> DBManager<'a> {
    pub fn new(db: &'a DBConfig, buffer_m: Rc<RefCell<BufferManager<'a>>>) -> Self {
        DBManager {
            basededonnees: HashMap::new(),
            dbconfig: db,
            bdd_courante: None::<String>,
            buffer_manager: buffer_m,
        }
    }

    pub fn get_bdd_courante(&mut self) -> Option<&mut Database<'a>> {
        if self.bdd_courante.is_some() {
            return self
                .basededonnees
                .get_mut(self.bdd_courante.as_ref().unwrap());
        } else {
            return None;
        }
    }

    pub fn get_basededonnees(&self) -> &HashMap<String, Database<'a>> {
        return &self.basededonnees;
    }

    pub fn get_dbconfig(&self) -> &'a DBConfig {
        return self.dbconfig;
    }

    pub fn create_data_base(&mut self, db: &str) {
        self.basededonnees
            .insert(db.to_string(), Database::new(db.to_string()));
    }

    pub fn set_current_data_base(&mut self, nom: &str) {
        if self.basededonnees.contains_key(nom) {
            self.bdd_courante = Some(nom.to_string());
        } else {
            //A voir, ca ne sert à rien en vrai car on suppose trjs que la BDD existe.
            println!("Database {} does not exist.", nom);
        }
    }

    pub fn add_table_to_current_data_base(&mut self, tab: Relation<'a>) {
        let nom = tab.get_name().clone();
        if self.bdd_courante.is_some() {
            self.get_bdd_courante().unwrap().add_relation(tab);
        }
        println!("Table {} created.", nom);
    }

    pub fn get_table_from_current_data_base(&mut self, nom_tab: &str) -> Option<&Relation<'a>> {
        let bdd = self.get_bdd_courante().unwrap();
        let rel_bdd = bdd.get_relations();
        let mut rel_result = None;
        for rel in rel_bdd.iter() {
            if rel.get_name() == nom_tab {
                rel_result = Some(rel);
            }
        }
        return rel_result;
    }

    pub fn remove_table_from_current_data_base(&mut self, nom_tab: &str) {
        self.get_bdd_courante().unwrap().remove_relation(nom_tab);
    }

    pub fn remove_data_base(&mut self, nom_bdd: &str) {
        if let Some(_db) = self.basededonnees.get(nom_bdd) {
            self.basededonnees
                .get_mut(nom_bdd)
                .unwrap()
                .set_relations(Vec::<Relation>::new());
            self.basededonnees.remove(nom_bdd);
            if !self.get_bdd_courante().is_none()
                && self.get_bdd_courante().unwrap().get_nom() == nom_bdd
            {
                self.bdd_courante = None;
            }
        }
    }

    pub fn remove_tables_from_current_data_base(&mut self) {
        match self.get_bdd_courante() {
            Some(_database) => self.get_bdd_courante().unwrap().set_relations(Vec::new()),
            _ => println!("No current database."),
        }
    }

    pub fn remove_data_bases(&mut self) {
        self.bdd_courante = None;
        self.basededonnees.clear();
    }

    pub fn list_databases(&mut self) {
        println!("Databases:");
        match self.get_bdd_courante() {
            Some(_database) => println!(
                "Current database: {}",
                self.get_bdd_courante().unwrap().get_nom()
            ),
            _ => println!("Current database: none."),
        }
        for bdd in self.basededonnees.keys() {
            println!("Database: {}", bdd)
        }
    }

    /* AU CAS OU L'AFFICHAGE N'EST PAS BON POUR L'AUTRE
    pub fn list_tables_in_current_data_base(&mut self){
        match self.get_bdd_courante(){
            Some(_database) => {let relations:&Vec<Relation> = self.get_bdd_courante().unwrap().get_relations();
                if relations.len() == 0{
                    println!("The current database does not contain any tables.");
                }
                for rel in relations {
                    println!("Table : {}, colonnes : ", rel.get_name());
                    let cols:Vec<ColInfo> = rel.get_columns();
                    for col in cols {
                        println!("nom : {}, type : {}", col.get_name(), col.get_columntype());
                    }
                }},
            => println!("Aucune bdd courante."),
        }
    } (modifié)
    */

    pub fn list_tables_in_current_data_base(&mut self) {
        match self.get_bdd_courante() {
            Some(database) => {
                let relations = database.get_relations();
                if relations.is_empty() {
                    println!("La base de données ne contient aucune table.");
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
        // Définir le fichier de sauvegarde
        let save_file = format!("{}/databases.json", self.dbconfig.get_dbpath());

        // Créer une structure de données pour la sauvegarde
        //Cela stock une BDD avec en format : <NOM_BDD, (NOM_REL,PageId,Noms cols, Types)>
        let mut sauvegarde: HashMap<String, Vec<(String, (u32, u32), Vec<String>, Vec<String>)>> =
            HashMap::new();

        // Pour chaque base de données
        for (nom_bdd, bdd) in &self.basededonnees {
            let mut relations: Vec<(String, (u32, u32), Vec<String>, Vec<String>)> = Vec::new();

            // Pour chaque relation dans la base de données
            for relation in bdd.get_relations() {
                let mut colonnes: Vec<String> = Vec::new();
                let mut types: Vec<String> = Vec::new();

                // Pour chaque colonne dans la relation, on ajoute son nom et son type
                for col in &relation.get_columns() {
                    colonnes.push(col.get_name().clone());
                    types.push(col.get_column_type().clone());
                }

                // Ajouter la relation avec son nom, son header page (file_idx, page_idx), et ses colonnes et types
                relations.push((
                    relation.get_name().to_string(),
                    (
                        relation.get_header_page_id().get_file_idx(),
                        relation.get_header_page_id().get_page_idx(),
                    ),
                    colonnes,
                    types,
                ));
            }
            //si la bdd est la bdd courante on ajoute un morceau a son nom pour pouvoir la reconnaitre plus tard
            if self.bdd_courante.clone().is_some()
                && self.bdd_courante.clone().unwrap().as_str() == nom_bdd.as_str()
            {
                sauvegarde.insert([nom_bdd, "BDD_COURANTE"].join(""), relations);
            } else {
                // Ajouter cette base de données et ses relations à la structure de sauvegarde
                sauvegarde.insert(nom_bdd.clone(), relations);
            }
        }

        // Sérialiser les données en JSON
        let json_data = serde_json::to_string_pretty(&sauvegarde)?;

        // Écrire les données sérialisées dans le fichier
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

        // Lire le contenu du fichier dans une chaîne de caractères
        let mut json_data = String::new();
        file.read_to_string(&mut json_data)?;

        // Désérialiser les données JSON en une HashMap qui contient les bases de données et leurs relations
        let sauvegarde: HashMap<String, Vec<(String, (u32, u32), Vec<String>, Vec<String>)>> =
            serde_json::from_str(&json_data)?;

        // Parcourir chaque base de données dans la sauvegarde
        for (mut nom_bdd, relations) in sauvegarde {
            //on regarde si la bdd est la bdd courante de la session precedente, si c'est le cas on l'ajoute comme bdd courante
            if nom_bdd.contains("BDD_COURANTE") {
                //ptet pas opti mais ca fonctionne
                let nouv_nom = nom_bdd
                    .to_string()
                    .drain(..nom_bdd.len() - 12)
                    .collect::<String>();
                nom_bdd = nouv_nom;
                self.bdd_courante = Some(nom_bdd.to_string());
            }

            let mut bdd = Database::new(nom_bdd.clone()); // Créer une nouvelle base de données

            // Pour chaque relation de cette base de données
            for (nom_relation, (file_idx, page_idx), colonnes, types) in relations {
                let mut cols: Vec<ColInfo> = Vec::new();

                // Créer les colonnes à partir des noms et types s
                for i in 0..colonnes.len() {
                    cols.push(ColInfo::new(colonnes[i].clone(), types[i].clone()));
                }

                // Créer le header page id pour la relation
                let header_page_id = PageId::new(file_idx, page_idx);

                // Créer la relation et l'ajouter à la base de données
                let relation = Relation::from_saved(
                    nom_relation,
                    cols,
                    header_page_id,
                    self.buffer_manager.clone(),
                );

                // Ajouter la relation à la base de données
                bdd.add_relation(relation);
            }

            // Ajouter la base de données restaurée au gestionnaire de bases de données
            self.basededonnees.insert(nom_bdd, bdd);
        }
        println!("LOAD STATE OK");
        // Retourner Ok après avoir terminé
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
    fn test_bdd_courante() {
        let s: String = String::from("config.json");
        let config = DBConfig::load_db_config(s);
        let dm = DiskManager::new(&config);
        let algo_lru = String::from("LRU");

        let buffer_manager1 = BufferManager::new(&config, dm, algo_lru);
        let rc_a = Rc::new(RefCell::new(buffer_manager1));

        let colinfo1: Vec<ColInfo> = vec![
            ColInfo::new("NOM".to_string(), "CHAR(10)".to_string()),
            ColInfo::new("AGE".to_string(), "INT".to_string()),
            ColInfo::new("PRENOM".to_string(), "VARCHAR(6)".to_string()),
        ];
        let relation1 = Relation::new("PERSONNE".to_string(), colinfo1.clone(), Rc::clone(&rc_a));

        let colinfo2: Vec<ColInfo> = vec![
            ColInfo::new("NOM".to_string(), "CHAR(20)".to_string()),
            ColInfo::new("ID".to_string(), "INT".to_string()),
            ColInfo::new("SALAIRE".to_string(), "FLOAT".to_string()),
        ];
        let relation2 = Relation::new("EMPLOI".to_string(), colinfo2.clone(), Rc::clone(&rc_a));

        let colinfo3: Vec<ColInfo> = vec![
            ColInfo::new("MARQUE".to_string(), "CHAR(20)".to_string()),
            ColInfo::new("MODELE".to_string(), "VARCHAR(10)".to_string()),
            ColInfo::new("ID".to_string(), "INT".to_string()),
            ColInfo::new("PUISSANCE".to_string(), "INT".to_string()),
            ColInfo::new("PRIX".to_string(), "FLOAT".to_string()),
        ];
        let relation3 = Relation::new("VOITURE".to_string(), colinfo3.clone(), Rc::clone(&rc_a));

        let colinfo4: Vec<ColInfo> = vec![
            ColInfo::new("MARQUE".to_string(), "CHAR(20)".to_string()),
            ColInfo::new("MODELE".to_string(), "VARCHAR(10)".to_string()),
            ColInfo::new("ID".to_string(), "INT".to_string()),
            ColInfo::new("PUISSANCE".to_string(), "INT".to_string()),
            ColInfo::new("CARBURANT".to_string(), "CHAR(10)".to_string()),
            ColInfo::new("PRIX".to_string(), "FLOAT".to_string()),
        ];
        let relation4 = Relation::new("TRACTEUR".to_string(), colinfo4.clone(), Rc::clone(&rc_a));

        let mut db_manager = DBManager::new(&config, Rc::clone(&rc_a));
        db_manager.create_data_base("carrefour");
        db_manager.set_current_data_base("carrefour");
        db_manager.add_table_to_current_data_base(relation1);
        db_manager.add_table_to_current_data_base(relation2);

        db_manager.list_databases();
        db_manager.list_tables_in_current_data_base();

        db_manager.create_data_base("concession");
        db_manager.set_current_data_base("concession");
        db_manager.add_table_to_current_data_base(relation3);
        db_manager.add_table_to_current_data_base(relation4);

        db_manager.list_databases();
        db_manager.list_tables_in_current_data_base();

        let _ = db_manager.save_state();
    }

    #[test]
    fn test_save_state_and_load_state() {
        let s: String = String::from("config.json");
        let config = DBConfig::load_db_config(s);
        let dm = DiskManager::new(&config);
        let algo_lru = String::from("LRU");

        let buffer_manager1 = BufferManager::new(&config, dm, algo_lru);
        let rc_a = Rc::new(RefCell::new(buffer_manager1));

        let mut db_manager = DBManager::new(&config, Rc::clone(&rc_a));
        let _ = db_manager.load_state();

        db_manager.set_current_data_base("concession");
        db_manager.list_databases();
        db_manager.list_tables_in_current_data_base();
    }
}
