mod buffer;
mod buffer_manager;
mod col_info;
mod condition;
mod config;
mod data_base;
mod db_manager;
mod disk_manager;
mod operator;
mod page;
mod page_info;
mod record;
mod record_id;
mod relation;
mod select;
mod sgbd;
mod types;

use crate::page::PageId;
use crate::sgbd::SGBD;
use config::DBConfig;

fn main() {
    let chemin_json = ("config.json").to_string();
    let dbc = DBConfig::load_db_config(chemin_json);
    let mut sgbd = SGBD::new(&dbc);
    println!("MiniRDBMS - Mini Relational Database Management System");

    sgbd.run();
}
