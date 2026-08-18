//! Loads database settings from JSON and exposes them to storage managers.

use std::fs::File;
pub struct DBConfig {
    dbpath: String,
    pagesize: u32,
    dm_maxfilesize: u32,
    bm_buffer_count: u32,
    bm_policy: String,
}

impl DBConfig {
    /// Creates a configuration object shared by the disk, buffer, and catalog managers.
    pub fn new(
        path: String,
        pagesize: u32,
        dm_maxfilesize: u32,
        bm_buffer_count: u32,
        bm_policy: String,
    ) -> Self {
        Self {
            dbpath: path,
            pagesize: pagesize,
            dm_maxfilesize: dm_maxfilesize,
            bm_buffer_count: bm_buffer_count,
            bm_policy: bm_policy,
        }
    }

    /// Returns the base directory used for database files.
    pub fn get_dbpath(&self) -> &String {
        &self.dbpath
    }

    /// Returns the fixed size of one page in bytes.
    pub fn get_page_size(&self) -> u32 {
        self.pagesize
    }

    /// Returns the maximum size allowed for one data file.
    pub fn get_dm_maxfilesize(&self) -> u32 {
        self.dm_maxfilesize
    }

    /// Returns how many pages the buffer manager can hold.
    pub fn get_bm_buffer_count(&self) -> u32 {
        self.bm_buffer_count
    }

    /// Returns the configured replacement policy, currently `LRU` or `MRU`.
    pub fn get_bm_policy(&self) -> &String {
        &self.bm_policy
    }

    /// Reads `config.json` and converts string values into strongly typed settings.
    pub fn load_db_config(file_config: String) -> DBConfig {
        let file = File::open(file_config).expect("file should open read only test");
        let value: serde_json::Value =
            serde_json::from_reader(file).expect("file should be proper JSON");

        let dbpath: String = value["dbpath"].as_str().unwrap().to_string();
        let pagesize: u32 = value["pagesize"]
            .as_str()
            .unwrap()
            .to_string()
            .parse()
            .expect("Not a number");
        let dm_maxfilesize: u32 = value["dm_maxfilesize"]
            .as_str()
            .unwrap()
            .to_string()
            .parse()
            .expect("Not a number");
        let bm_buffer_count: u32 = value["bm_buffer_count"]
            .as_str()
            .unwrap()
            .to_string()
            .parse()
            .expect("");
        let bm_policy: String = value["bm_policy"].as_str().unwrap().to_string();
        return DBConfig::new(dbpath, pagesize, dm_maxfilesize, bm_buffer_count, bm_policy);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_constructor() {
        let s = String::from("res/dbpath");
        let ps_test: u32 = 32;
        let dm_max_test: u32 = 64;
        let bm_buffer_count: u32 = 4;
        let bm_policy: String = String::from("LRU");

        let config = DBConfig::new(s, ps_test, dm_max_test, bm_buffer_count, bm_policy);
        assert_eq!(config.dbpath, "res/dbpath");
        assert_eq!(config.pagesize, 32);
        assert_eq!(config.dm_maxfilesize, 64);
        assert_eq!(config.bm_buffer_count, 4);
        assert_eq!(config.bm_policy, "LRU".to_string());
    }

    #[test]
    fn test_load_db_config() {
        let path_json = String::from("config.json");
        let config = DBConfig::load_db_config(path_json);
        assert_eq!(config.dbpath, "res/dbpath");
        assert_eq!(config.pagesize, 4096);
        assert_eq!(config.dm_maxfilesize, 65536);
        assert_eq!(config.bm_buffer_count, 4);
        assert_eq!(config.bm_policy, "LRU".to_string());
    }
}
