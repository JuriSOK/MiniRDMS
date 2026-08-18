//! Interactive MiniRDBMS shell and command dispatcher.

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

pub struct MiniRdbms<'a> {
    dbconfig: &'a DBConfig,
    buffer_manager: Rc<RefCell<BufferManager<'a>>>,
    db_manager: RefCell<DBManager<'a>>,
}

impl<'a> MiniRdbms<'a> {
    /// Wires the disk, buffer, and catalog managers from the loaded configuration.
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

        MiniRdbms {
            dbconfig: db,
            buffer_manager: Rc::clone(&rc_bfm),
            db_manager: RefCell::new(dbm),
        }
    }

    /// Reads CLI commands until QUIT and dispatches each command to the matching handler.
    pub fn run(&mut self) {
        let mut input: String = String::from("");
        while input != "q".to_string() {
            print!(":");
            let _ = stdout().flush();
            input = "".to_string();
            stdin()
                .read_line(&mut input)
                .expect("Did not enter a correct string");
            if let Some('\n') = input.chars().next_back() {
                input.pop();
            }
            if let Some('\r') = input.chars().next_back() {
                input.pop();
            }
            match input.as_str() {
                s if s.starts_with("QUIT") => {
                    self.process_quit_command(&input);
                    input = "q".to_string()
                }
                s if s.starts_with("CREATE DATABASE") => self.process_create_database_command(
                    &input.split_whitespace().next_back().unwrap().to_string(),
                ),
                s if s.starts_with("SET DATABASE") => self.process_set_database_command(
                    &input.split_whitespace().next_back().unwrap().to_string(),
                ),
                s if s.starts_with("DROP DATABASES") => self.process_drop_databases_command(
                    &input.split_whitespace().next_back().unwrap().to_string(),
                ),
                s if s.starts_with("DROP DATABASE") => self.process_drop_database_command(
                    &input.split_whitespace().next_back().unwrap().to_string(),
                ),
                s if s.starts_with("LIST DATABASES") => self.process_list_databases_command(
                    &input.split_whitespace().next_back().unwrap().to_string(),
                ),
                s if s.starts_with("CREATE TABLE") => {
                    let parts = &input.split_whitespace().collect::<Vec<&str>>();
                    self.process_create_table_command(&parts[parts.len() - 2..].join(" "))
                }
                s if s.starts_with("DROP TABLES") => self.process_drop_tables_command(
                    &input.split_whitespace().next_back().unwrap().to_string(),
                ),
                s if s.starts_with("DROP TABLE") => self.process_drop_table_command(
                    &input.split_whitespace().next_back().unwrap().to_string(),
                ),
                s if s.starts_with("LIST TABLES") => self.process_list_tables_command(
                    &input.split_whitespace().next_back().unwrap().to_string(),
                ),
                s if s.starts_with("INSERT INTO") => {
                    let parts = &input.split_whitespace().collect::<Vec<&str>>();
                    self.process_insert_command(&parts[2..].join(" "));
                }
                s if s.starts_with("BULKINSERT INTO") => {
                    let parts = &input.split_whitespace().collect::<Vec<&str>>();
                    self.process_bulk_insert_command(&parts[2..].join(" "));
                }
                s if s.starts_with("SELECT") => {
                    self.process_select_command(&input);
                }

                _ => println!("{} is not a command", input),
            }
        }
    }
    /// Persists catalog and disk state before leaving the shell.
    pub fn process_quit_command(&mut self, _command: &String) {
        let _ = self.db_manager.borrow_mut().save_state();
        let dm = DiskManager::new(&self.dbconfig);
        let _ = dm.save_state();
        self.buffer_manager.borrow_mut().flush_buffers();
        println!("Goodbye.");
    }
    /// Handles `CREATE DATABASE <name>`.
    pub fn process_create_database_command(&mut self, command: &String) {
        self.db_manager.borrow_mut().create_database(command);
        println!("Database {} created.", command);
    }

    /// Handles `SET DATABASE <name>`.
    pub fn process_set_database_command(&mut self, command: &String) {
        self.db_manager.borrow_mut().set_current_database(command);
        println!("Current database is now: {}", command)
    }

    /// Handles `LIST DATABASES`.
    pub fn process_list_databases_command(&mut self, _command: &String) {
        self.db_manager.borrow_mut().list_databases()
    }

    /// Handles `CREATE TABLE <name> (<column>:<type>,...)`.
    pub fn process_create_table_command(&mut self, command: &String) {
        let mut dbm = self.db_manager.borrow_mut();
        let parts = command.split_whitespace().collect::<Vec<&str>>();
        let name = parts[0].to_string();
        let mut table_char = parts[1].chars();
        let _ = table_char.next();
        let _ = table_char.next_back();
        let table_parts = table_char.as_str().split([',']).collect::<Vec<&str>>();
        let mut columns = Vec::new();

        for column_definition in table_parts {
            let column_parts = column_definition.split(':').collect::<Vec<&str>>();
            columns.push(ColInfo::new(
                column_parts[0].to_string(),
                column_parts[1].to_string(),
            ));
        }
        let relation = Relation::new(name, columns, Rc::clone(&self.buffer_manager));
        dbm.add_table_to_current_database(relation);
    }

    /// Handles `DROP TABLE <name>` and releases the table pages.
    pub fn process_drop_table_command(&mut self, command: &String) {
        let mut dbm = self.db_manager.borrow_mut();

        match dbm.get_current_database() {
            Some(_database) => {
                let table_candidate = dbm.get_table_from_current_database(command);
                if table_candidate.is_none() {
                    println!("Table does not exist.");
                    return;
                }
                let table = table_candidate.unwrap();
                let hp_id = table.get_header_page_id();
                let page_ids = table.get_data_pages();
                let bfm = self.buffer_manager.borrow_mut();
                let mut dm = bfm.get_disk_manager();
                dm.dealloc_page(hp_id.clone());
                for page_id in page_ids {
                    dm.dealloc_page(page_id);
                }
                dbm.remove_table_from_current_database(command);
                println!("{} has been dropped.", command);
            }
            _ => println!("No current database."),
        }
    }

    /// Handles `DROP TABLES` for the current database.
    pub fn process_drop_tables_command(&mut self, _command: &String) {
        let mut dbm = self.db_manager.borrow_mut();
        match dbm.get_current_database() {
            Some(_database) => {
                let tables = dbm.get_current_database().unwrap().get_relations();
                let mut page_ids = Vec::new();
                for rel in tables {
                    page_ids.push(rel.get_header_page_id().clone());
                    page_ids.append(&mut rel.get_data_pages());
                }
                for page in page_ids {
                    let bfm = self.buffer_manager.borrow_mut();
                    let mut dm = bfm.get_disk_manager();
                    dm.dealloc_page(page);
                }
                dbm.remove_tables_from_current_database();
                println!("All tables dropped.");
            }
            _ => println!("No current database."),
        }
    }

    /// Handles `DROP DATABASES` by dropping each database and its tables.
    pub fn process_drop_databases_command(&mut self, command: &String) {
        let database_names: Vec<String> = {
            let dbm = self.db_manager.borrow_mut();
            dbm.get_databases().keys().cloned().collect()
        };

        for database in database_names {
            {
                let mut dbm = self.db_manager.borrow_mut();
                dbm.set_current_database(&database);
            }
            self.process_drop_tables_command(command);
        }

        self.db_manager.borrow_mut().remove_databases();
        println!("All databases dropped.");
    }

    /// Handles `DROP DATABASE <name>`.
    pub fn process_drop_database_command(&mut self, command: &String) {
        let database_names: Vec<String> = {
            let dbm = self.db_manager.borrow_mut();
            dbm.get_databases().keys().cloned().collect()
        };
        if database_names.contains(command) {
            let mut dbm = self.db_manager.borrow_mut();
            dbm.remove_database(command);
        } else {
            println!("Database {} does not exist.", command);
        }
    }

    /// Handles `LIST TABLES` for the current database.
    pub fn process_list_tables_command(&mut self, _command: &String) {
        let mut dbm = self.db_manager.borrow_mut();
        dbm.list_tables_in_current_database();
    }

    /// Handles `INSERT INTO <table> VALUES (...)`.
    pub fn process_insert_command(&mut self, command: &String) {
        let mut database_manager = self.db_manager.borrow_mut();

        let parts = command.split_whitespace().collect::<Vec<&str>>();

        let relation_name = parts[0].to_string();
        let mut values_chars = parts[2].chars();
        let _ = values_chars.next();
        let _ = values_chars.next_back();

        let values_info = values_chars.as_str().split(',').collect::<Vec<&str>>();

        let mut values: Vec<String> = Vec::new();

        for val in values_info {
            if (val.starts_with('"')) || (val.starts_with('“')) || (val.starts_with('ʺ')) {
                let mut chars = val.chars();
                chars.next();
                chars.next_back();
                let result = chars.as_str();
                values.push(result.to_string());
            } else {
                values.push(val.to_string());
            }
        }
        let current_database = database_manager.get_current_database().unwrap();
        let relations = current_database.get_relations_mut();

        for rel in relations {
            if rel.get_name().as_str() == relation_name {
                let record_id = rel.insert_record(Record::new(values));
                let _insert_location = (record_id.page_id, record_id.slot_idx);
                break;
            }
        }
        println!("INSERT completed.");
    }

    /// Handles `BULKINSERT INTO <table> <file>` by inserting each CSV line.
    pub fn process_bulk_insert_command(&mut self, command: &String) {
        let mut database_manager = self.db_manager.borrow_mut();

        let parts = command.split_whitespace().collect::<Vec<&str>>();

        let relation_name = parts[0].to_string();
        let file_name = parts[1].to_string();

        let file_content = std::fs::read_to_string(&file_name).unwrap();
        let lines = file_content.lines();

        let current_database = database_manager.get_current_database().unwrap();
        let relations = current_database.get_relations_mut();

        for rel in relations {
            if rel.get_name().as_str() == relation_name {
                for line in lines.clone() {
                    let values_info = line.split(',').collect::<Vec<&str>>();
                    let mut values: Vec<String> = Vec::new();

                    for val in values_info {
                        if (val.starts_with('"'))
                            || (val.starts_with('“'))
                            || (val.starts_with('ʺ'))
                        {
                            let mut chars = val.chars();
                            chars.next();
                            chars.next_back();
                            let result = chars.as_str();
                            values.push(result.to_string());
                        } else {
                            values.push(val.to_string());
                        }
                    }
                    let record_id = rel.insert_record(Record::new(values));
                    let _insert_location = (record_id.page_id, record_id.slot_idx);
                }
                break;
            }
        }
        println!("BULKINSERT completed.");
    }

    /// Handles single-table `SELECT` queries through scan, filter, project, and print operators.
    pub fn process_select_command(&mut self, command: &String) {
        let mut dbm = self.db_manager.borrow_mut();

        let select = Select::new(command.as_str());

        let select_result: Select;

        if select.is_err() {
            println!("{}", select.err().unwrap());
            return;
        } else {
            select_result = select.unwrap();
        }

        let column_names = select_result.get_columns().clone();

        if select_result.get_tables().len() > 1 {
            println!("Error: multiple tables are not supported yet.");
            return;
        }

        let mut projected_column_names: Vec<String> = Vec::new();
        if !select_result.get_columns()[0].eq("*") {
            for col_name in column_names {
                let tmp = Condition::split_column(col_name.as_str());
                if tmp.is_ok() {
                    let (_alias, column_name) = tmp.unwrap();
                    projected_column_names.push(column_name.to_string());
                }
            }
        }

        match dbm.get_current_database() {
            Some(_database) => {
                let tables: &mut Vec<Relation<'_>> =
                    dbm.get_current_database().unwrap().get_relations_mut();

                let mut table_candidate: Option<&mut Relation> = None;
                for table_iter in tables {
                    let table_parts = select_result.get_tables()[0]
                        .as_str()
                        .split(' ')
                        .collect::<Vec<&str>>();
                    if table_parts.len() == 2 && table_iter.get_name().eq(table_parts[0]) {
                        table_candidate = Some(table_iter);
                    }
                }

                if table_candidate.is_none() {
                    println!("Table does not exist.");
                    return;
                }
                let table = table_candidate.unwrap();
                let table_clone = table.get_all_records();
                let iterator = SelectOperator::new(
                    select_result.clone(),
                    Box::new(RelationScanner::new(table_clone)),
                    Rc::new(table.get_columns()),
                );

                let project_operator: ProjectionOperator;

                let table_columns = table.get_columns().clone();
                let mut tmp_table_columns: Vec<String> = Vec::new();
                if select_result.get_columns()[0].eq("*") {
                    for col_name in table_columns {
                        tmp_table_columns.push(col_name.get_name().to_string());
                    }
                    project_operator = ProjectionOperator::new(
                        tmp_table_columns.clone(),
                        Box::new(iterator),
                        Rc::new(table.get_columns()),
                    );
                } else {
                    project_operator = ProjectionOperator::new(
                        projected_column_names.clone(),
                        Box::new(iterator),
                        Rc::new(table.get_columns()),
                    );
                }

                let mut record_printer = RecordPrinter::new(Box::new(project_operator));
                record_printer.print_records();
            }
            _ => {
                println!("No current database.")
            }
        }
    }
}
