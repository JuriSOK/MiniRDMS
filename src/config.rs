use std::fs::File;
//use serde_json::{Result, Value};

pub struct DBConfig {
    dbpath: String,
    pagesize: u32,
    dm_maxfilesize: u32,
    bm_buffer_count: u32,
    bm_policy: String,
}

impl DBConfig {
    //permet d'implémenter la structure (en gros c'est la classe en elle même et struct c'est juste pour mettre les valeurs je pense)
    pub fn new(
        chemin: String,
        pagesize: u32,
        dm_maxfilesize: u32,
        bm_buffer_count: u32,
        bm_policy: String,
    ) -> Self {
        //Constructeur de la classe
        Self {
            //dans ce scope on met les attributs de la classe
            dbpath: chemin,
            pagesize: pagesize,
            dm_maxfilesize: dm_maxfilesize,
            bm_buffer_count: bm_buffer_count,
            bm_policy: bm_policy,
        }
    }

    pub fn set_dbpath(&mut self, chemin: String) {
        self.dbpath = chemin;
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

    pub fn load_db_config(fichier_config: String) -> DBConfig {
        //println!("{}", fichier_config);
        let file = File::open(fichier_config).expect("file should open read only test"); //OUVRE LE FICHIER
        let valeur: serde_json::Value =
            serde_json::from_reader(file).expect("file should be proper JSON"); // RECUPERE LE CONTENUE

        let dbpath: String = valeur["dbpath"].as_str().unwrap().to_string();
        let pagesize: u32 = valeur["pagesize"]
            .as_str()
            .unwrap()
            .to_string()
            .parse()
            .expect("Not a number");
        let dm_maxfilesize: u32 = valeur["dm_maxfilesize"]
            .as_str()
            .unwrap()
            .to_string()
            .parse()
            .expect("Not a number");
        let bm_buffer_count: u32 = valeur["bm_buffer_count"]
            .as_str()
            .unwrap()
            .to_string()
            .parse()
            .expect("");
        let bm_policy: String = valeur["bm_policy"].as_str().unwrap().to_string();
        return DBConfig::new(dbpath, pagesize, dm_maxfilesize, bm_buffer_count, bm_policy);
        // Sans to_string ça renvoie un truc bizarre
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_constructeur() {
        let s = String::from("res/dbpath");
        let ps_test: u32 = 32;
        let dm_max_test: u32 = 64;
        let bm_buffer_count: u32 = 4;
        let bm_policy: String = String::from("LRU");

        let classe = DBConfig::new(s, ps_test, dm_max_test, bm_buffer_count, bm_policy);
        assert_eq!(classe.dbpath, "res/dbpath");
        assert_eq!(classe.pagesize, 32);
        assert_eq!(classe.dm_maxfilesize, 64);
        assert_eq!(classe.bm_buffer_count, 4);
        assert_eq!(classe.bm_policy, "LRU".to_string());
    }

    #[test]
    fn test_load_db_config() {
        let chemin_json = String::from("config.json");
        let classe = DBConfig::load_db_config(chemin_json);
        assert_eq!(classe.dbpath, "res/dbpath");
        assert_eq!(classe.pagesize, 4096);
        assert_eq!(classe.dm_maxfilesize, 65536);
        assert_eq!(classe.bm_buffer_count, 4);
        assert_eq!(classe.bm_policy, "LRU".to_string());
    }
}
