use crate::buffer_manager::BufferManager;
use crate::col_info::ColInfo;
use crate::condition::Condition;
use crate::db_manager::DBManager;
use crate::disk_manager::DiskManager;
use crate::operator::ProjectionOperator;
use crate::operator::RecordPrinter;
use crate::operator::RelationScanner;
use crate::operator::SelectOperator;
use crate::record::Record;
use crate::relation::Relation;
use crate::select::Select;
use crate::DBConfig;
use std::cell::RefCell;
use std::io::{stdin, stdout, Write};
use std::rc::Rc;

pub struct SGBD<'a> {
    dbconfig: &'a DBConfig,
    buffer_manager: Rc<RefCell<BufferManager<'a>>>,
    db_manager: RefCell<DBManager<'a>>,
}

impl<'a> SGBD<'a> {
    pub fn new(db: &'a DBConfig) -> Self {
        let mut dk = DiskManager::new(db);
        let _ = dk.load_state();
        let rc_bfm = Rc::new(RefCell::new(BufferManager::new(
            db,
            dk,
            db.get_bm_policy().to_string(),
        )));
        let mut dbm = DBManager::new(db, Rc::clone(&rc_bfm));
        let _ = dbm.load_state();

        SGBD {
            dbconfig: db,
            buffer_manager: Rc::clone(&rc_bfm),
            db_manager: RefCell::new(dbm), //DBManager::new(&db, Rc::clone(&rc_bfm))
        }
    }
    pub fn run(&mut self) {
        let mut saisie: String = String::from("");
        while saisie != "q".to_string() {
            print!(":");
            //code complètement emprunté sur le forum rust, ne pas me demander comment ça fonctionne
            let _ = stdout().flush();
            saisie = "".to_string();
            stdin()
                .read_line(&mut saisie)
                .expect("Did not enter a correct string");
            if let Some('\n') = saisie.chars().next_back() {
                saisie.pop();
            }
            if let Some('\r') = saisie.chars().next_back() {
                saisie.pop();
            }
            match saisie.as_str() {
                s if s.starts_with("QUIT") => {
                    self.process_quit_command(&saisie);
                    saisie = "q".to_string()
                }
                s if s.starts_with("CREATE DATABASE") => self.process_create_data_base_command(
                    &saisie.split_whitespace().next_back().unwrap().to_string(),
                ), //la on prend la chaine de caractere on la transforme en iterateur et on prend le dernier element, en esperant que ca soit le nom de la BDD
                s if s.starts_with("SET DATABASE") => self.process_set_data_base_command(
                    &saisie.split_whitespace().next_back().unwrap().to_string(),
                ),
                s if s.starts_with("DROP DATABASES") => self.process_drop_data_bases_command(
                    &saisie.split_whitespace().next_back().unwrap().to_string(),
                ),
                s if s.starts_with("DROP DATABASE") => self.process_drop_data_base_command(
                    &saisie.split_whitespace().next_back().unwrap().to_string(),
                ),
                s if s.starts_with("LIST DATABASES") => self.process_list_data_bases_command(
                    &saisie.split_whitespace().next_back().unwrap().to_string(),
                ),
                s if s.starts_with("CREATE TABLE") => {
                    let tmp = &saisie.split_whitespace().collect::<Vec<&str>>();
                    self.process_create_table_command(&tmp[tmp.len() - 2..].join(" "))
                } //certifié presque fait maison, si ca fonctionne faut pas toucher
                s if s.starts_with("DROP TABLES") => self.process_drop_tables_command(
                    &saisie.split_whitespace().next_back().unwrap().to_string(),
                ),
                s if s.starts_with("DROP TABLE") => self.process_drop_table_command(
                    &saisie.split_whitespace().next_back().unwrap().to_string(),
                ),
                s if s.starts_with("LIST TABLES") => self.process_list_tables_command(
                    &saisie.split_whitespace().next_back().unwrap().to_string(),
                ),
                s if s.starts_with("INSERT INTO") => {
                    let tmp = &saisie.split_whitespace().collect::<Vec<&str>>();
                    self.process_insert_command(&tmp[2..].join(" "));
                }
                s if s.starts_with("BULKINSERT INTO") => {
                    let tmp = &saisie.split_whitespace().collect::<Vec<&str>>();
                    self.process_bulk_insert_command(&tmp[2..].join(" "));
                }
                s if s.starts_with("SELECT") => {
                    self.process_select_command(&saisie); // Appeler la méthode de traitement pour SELECT
                }

                _ => println!("{} is not a command", saisie),
            }
        }
    }
    pub fn process_quit_command(&mut self, _commande: &String) {
        let _ = self.db_manager.borrow_mut().save_state();
        let dm = DiskManager::new(&self.dbconfig);
        let _ = dm.save_state();
        self.buffer_manager.borrow_mut().flush_buffers();
        println!("Goodbye.");
    }
    pub fn process_create_data_base_command(&mut self, commande: &String) {
        self.db_manager.borrow_mut().create_data_base(commande);
        println!("Database {} created.", commande);
    }
    pub fn process_set_data_base_command(&mut self, commande: &String) {
        self.db_manager.borrow_mut().set_current_data_base(commande);
        println!("Current database is now: {}", commande)
    }
    pub fn process_list_data_bases_command(&mut self, _commande: &String) {
        self.db_manager.borrow_mut().list_databases()
    }
    pub fn process_create_table_command(&mut self, commande: &String) {
        let mut dbm = self.db_manager.borrow_mut();
        let infos = commande.split_whitespace().collect::<Vec<&str>>();
        let name = infos[0].to_string();
        let mut table_char = infos[1].chars();
        let _ = table_char.next(); //on eneleve la premiere parenthese
        let _ = table_char.next_back(); //on eneleve la derniere parenthese fermante
        let table_infos = table_char.as_str().split([',']).collect::<Vec<&str>>();
        let mut vec_cols = Vec::new();

        for chaine in table_infos {
            /*
            let tmp = ColInfo::new(chaine.split(':').collect::<Vec<&str>>()[0], chaine.split(':').collect::<Vec<&str>>()[1]);
            vec_cols.push(tmp);
            */
            vec_cols.push(ColInfo::new(
                chaine.split(':').collect::<Vec<&str>>()[0].to_string(),
                chaine.split(':').collect::<Vec<&str>>()[1].to_string(),
            )); //pour faire un vec de col info
        }
        let rel = Relation::new(name, vec_cols, Rc::clone(&self.buffer_manager));
        dbm.add_table_to_current_data_base(rel);
    }
    pub fn process_drop_table_command(&mut self, commande: &String) {
        //desallouer toutes les pages de la table, header page + data page j'imagine
        let mut dbm = self.db_manager.borrow_mut();

        match dbm.get_bdd_courante() {
            Some(_database) => {
                let table_tmp = dbm.get_table_from_current_data_base(commande);
                if table_tmp.is_none() {
                    println!("Table does not exist.");
                    return;
                }
                let table = table_tmp.unwrap();
                let hp_id = table.get_header_page_id();
                let page_ids = table.get_data_pages();
                let bfm = self.buffer_manager.borrow_mut();
                let mut dm = bfm.get_disk_manager();
                dm.dealloc_page(hp_id.clone());
                for page_id in page_ids {
                    dm.dealloc_page(page_id);
                }
                dbm.remove_table_from_current_data_base(commande);
                println!("{} has been dropped.", commande);
            }
            _ => println!("No current database."),
        }
    }
    pub fn process_drop_tables_command(&mut self, _commande: &String) {
        //j'aurai pu utiliser process_drop_table c'est vrai
        let mut dbm = self.db_manager.borrow_mut();
        match dbm.get_bdd_courante() {
            Some(_database) => {
                let tables = dbm.get_bdd_courante().unwrap().get_relations();
                let mut page_ids = Vec::new();
                //let bfm = self.buffer_manager.borrow_mut();
                //let mut dm = bfm.get_disk_manager();
                for rel in tables {
                    page_ids.push(rel.get_header_page_id().clone());
                    page_ids.append(&mut rel.get_data_pages());
                }
                for page in page_ids {
                    let bfm = self.buffer_manager.borrow_mut();
                    let mut dm = bfm.get_disk_manager();
                    dm.dealloc_page(page);
                }
                dbm.remove_tables_from_current_data_base();
                println!("All tables dropped.");
            }
            _ => println!("No current database."),
        }
    }
    pub fn process_drop_data_bases_command(&mut self, commande: &String) {
        // Collecter les noms des bases de données en dehors de l'emprunt mutable
        let bdds: Vec<String> = {
            let dbm = self.db_manager.borrow_mut();
            dbm.get_basededonnees().keys().cloned().collect() // cloner pour éviter les références
        };

        for bdd in bdds {
            {
                // Emprunt temporaire et limité de db_manager pour modifier la base courante
                let mut dbm = self.db_manager.borrow_mut();
                dbm.set_current_data_base(&bdd);
            }
            // Maintenant, on peut emprunter `self` mutablement sans conflit
            self.process_drop_tables_command(commande);
        }

        // Dernier emprunt pour supprimer les bases de données
        self.db_manager.borrow_mut().remove_data_bases();
        println!("All databases dropped.");
    }

    pub fn process_drop_data_base_command(&mut self, commande: &String) {
        // Collecter les noms des bases de données en dehors de l'emprunt mutable
        let bdds: Vec<String> = {
            let dbm = self.db_manager.borrow_mut();
            dbm.get_basededonnees().keys().cloned().collect() // cloner pour éviter les références
        };
        if bdds.contains(commande) {
            let mut dbm = self.db_manager.borrow_mut();
            dbm.remove_data_base(commande);
        } else {
            println!("Database {} does not exist.", commande);
        }
    }

    pub fn process_list_tables_command(&mut self, _commande: &String) {
        let mut dbm = self.db_manager.borrow_mut();
        dbm.list_tables_in_current_data_base();
    }

    pub fn process_insert_command(&mut self, commande: &String) {
        let mut all_bdd = self.db_manager.borrow_mut();

        let infos = commande.split_whitespace().collect::<Vec<&str>>();

        let nom_relation = infos[0].to_string();
        let mut values_chars = infos[2].chars();
        let _ = values_chars.next();
        let _ = values_chars.next_back();

        let values_info = values_chars.as_str().split(',').collect::<Vec<&str>>();

        let mut valeurs: Vec<String> = Vec::new();

        for val in values_info {
            if (val.starts_with('"')) || (val.starts_with('“')) || (val.starts_with('ʺ')) {
                let mut chars = val.chars();
                chars.next();
                chars.next_back();
                let resultat = chars.as_str();
                valeurs.push(resultat.to_string());
            } else {
                valeurs.push(val.to_string());
            }
        }
        let bdd_courant = all_bdd.get_bdd_courante().unwrap();
        let relations = bdd_courant.get_relations_mut();

        for rel in relations {
            if rel.get_name().as_str() == nom_relation {
                rel.insert_record(Record::new(valeurs));
                break;
            }
        }
        println!("INSERT completed.");
    }

    pub fn process_bulk_insert_command(&mut self, commande: &String) {
        let mut all_bdd = self.db_manager.borrow_mut();

        let infos = commande.split_whitespace().collect::<Vec<&str>>();

        let nom_relation = infos[0].to_string();
        let nom_fichier = infos[1].to_string();

        // Lire le fichier CSV
        let file_content = std::fs::read_to_string(&nom_fichier).unwrap();

        // Diviser le fichier en lignes
        let lines = file_content.lines();

        let bdd_courant = all_bdd.get_bdd_courante().unwrap();
        let relations = bdd_courant.get_relations_mut();

        for rel in relations {
            if rel.get_name().as_str() == nom_relation {
                for line in lines.clone() {
                    let values_info = line.split(',').collect::<Vec<&str>>();
                    let mut valeurs: Vec<String> = Vec::new();

                    for val in values_info {
                        if (val.starts_with('"'))
                            || (val.starts_with('“'))
                            || (val.starts_with('ʺ'))
                        {
                            let mut chars = val.chars();
                            chars.next();
                            chars.next_back();
                            let resultat = chars.as_str();
                            valeurs.push(resultat.to_string());
                        } else {
                            valeurs.push(val.to_string());
                        }
                    }
                    rel.insert_record(Record::new(valeurs));
                }
                break; // Si problème, l'enlever.
            }
        }
        println!("BULKINSERT completed.");
    }

    pub fn process_select_command(&mut self, commande: &String) {
        let mut dbm = self.db_manager.borrow_mut();

        let select = Select::new(commande.as_str());

        let select_result: Select;

        if select.is_err() {
            println!("{}", select.err().unwrap());
            return;
        } else {
            select_result = select.unwrap();
        }

        let column_names = select_result.get_colonnes().clone();

        if select_result.get_tables().len() > 1 {
            println!("Error: multiple tables are not supported yet.");
            return;
        }

        let mut colomn_names_res: Vec<String> = Vec::new();
        if !select_result.get_colonnes()[0].eq("*") {
            for col_name in column_names {
                let tmp = Condition::split_colonne(col_name.as_str());
                if tmp.is_ok() {
                    let (_tmp2, tmp3) = tmp.unwrap();
                    colomn_names_res.push(tmp3.to_string());
                }
            }
        }

        match dbm.get_bdd_courante() {
            Some(_database) => {
                let tables: &mut Vec<Relation<'_>> =
                    dbm.get_bdd_courante().unwrap().get_relations_mut();

                let mut table_tmp: Option<&mut Relation> = None;
                for tableiter in tables {
                    let tab = select_result.get_tables()[0]
                        .as_str()
                        .split(' ')
                        .collect::<Vec<&str>>();
                    if tab.len() == 2 && tableiter.get_name().eq(tab[0]) {
                        table_tmp = Some(tableiter);
                    }
                }

                if table_tmp.is_none() {
                    println!("Table does not exist.");
                    return;
                }
                let table = table_tmp.unwrap();
                let table_clone = table.get_all_records();
                let iterateur = SelectOperator::new(
                    select_result.clone(),
                    Box::new(RelationScanner::new(table_clone)),
                    Rc::new(table.get_columns()),
                );

                let project_oper: ProjectionOperator;

                let tmp_column_names = table.get_columns().clone();
                let mut tmp_tmp_column_names: Vec<String> = Vec::new();
                if select_result.get_colonnes()[0].eq("*") {
                    for col_name in tmp_column_names {
                        tmp_tmp_column_names.push(col_name.get_name().to_string());
                    }
                    project_oper = ProjectionOperator::new(
                        tmp_tmp_column_names.clone(),
                        Box::new(iterateur),
                        Rc::new(table.get_columns()),
                    );
                } else {
                    project_oper = ProjectionOperator::new(
                        colomn_names_res.clone(),
                        Box::new(iterateur),
                        Rc::new(table.get_columns()),
                    );
                }

                let mut record_printer = RecordPrinter::new(Box::new(project_oper));
                record_printer.print_records();

                //let mut record_printer = RecordPrinter::new(Rc::new(table.get_columns()));
                //TODO:Continuer dans la mÃªme logique
            }
            _ => {
                println!("No current database.")
            }
        }
    }
}
