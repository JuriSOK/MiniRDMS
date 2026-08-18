use std::fs::File;

pub struct DBConfig {
    dbpath: String,
    pagesize: u32,
    dm_maxfilesize: u32,
    bm_buffer_count: u32,
    bm_policy: String,
}

impl DBConfig {
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

    pub fn set_dbpath(&mut self, path: String) {
        self.dbpath = path;
    }

    pub fn get_dbpath(&self) -> &String {
        &self.dbpath
    }

    pub fn get_page_size(&self) -> u32 {
        self.pagesize
    }

    pub fn get_dm_maxfilesize(&self) -> u32 {
        self.dm_maxfilesize
    }

    pub fn get_bm_buffer_count(&self) -> u32 {
        self.bm_buffer_count
    }

    pub fn get_bm_policy(&self) -> &String {
        &self.bm_policy
    }

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
