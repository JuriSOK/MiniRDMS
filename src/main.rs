mod buffer;
mod buffer_manager;
mod col_info;
mod condition;
mod config;
mod data_base;
mod db_manager;
mod disk_manager;
mod mini_rdbms;
mod operator;
mod page;
mod page_info;
mod record;
mod record_id;
mod relation;
mod select;
mod types;

use crate::mini_rdbms::MiniRdbms;
use crate::page::PageId;
use config::DBConfig;

fn main() {
    let path_json = ("config.json").to_string();
    let dbc = DBConfig::load_db_config(path_json);
    let mut mini_rdbms = MiniRdbms::new(&dbc);
    println!("MiniRDBMS - Mini Relational Database Management System");

    mini_rdbms.run();
}
