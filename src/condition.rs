use crate::col_info::ColInfo;
use crate::record::Record;
use crate::types::{Chars, Number, Operande};
use fancy_regex::Regex;
use once_cell::sync::Lazy;
use std::error::Error;

#[derive(Debug)]
pub struct PatternError {
    pub message: String,
}
impl PatternError {
    pub fn new(message: &str) -> Self {
        PatternError {
            message: message.to_string(),
        }
    }
}
//Pour l'affichage en mode Type { attr1 : ... , attr2 : ... etc. }
impl std::fmt::Display for PatternError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl Error for PatternError {}

#[derive(Debug)]
pub enum Operateur {
    EQUAL,
    LESSTHAN,
    GREATERTHAN,
    LESSEQUAL,
    GREATEREQUAL,
    NOTEQUAL,
}

pub struct Condition {
    oper_gauche: Box<dyn Operande>,
    operateur: Operateur,
    oper_droite: Box<dyn Operande>,
}

pub static AUCUN_CONST: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([a-zA-Z_][a-zA-Z0-9_.-]*\.[a-zA-Z0-9_.-]+)\s*(=|<>|<=|>=|<|>)\s*([a-zA-Z_][a-zA-Z0-9_.-]*\.[a-zA-Z0-9_.-]+)$")
        .expect("Erreur création du regex AUCUN_CONST")
});

pub static CHAR_CONST_GAUCHE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(['"ʺ])([a-zA-Z0-9_-]+)\1\s*(=|<>|<=|>=|<|>)\s*([a-zA-Z_][a-zA-Z0-9_.-]*\.[a-zA-Z0-9_.-]+)$"#)
        .expect("Erreur création du regex CHAR_CONST_GAUCHE")
});

pub static CHAR_CONST_DROITE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^([a-zA-Z_][a-zA-Z0-9_.-]*\.[a-zA-Z0-9_.-]+)\s*(=|<>|<=|>=|<|>)\s*(['"ʺ])([a-zA-Z0-9_-]+)\3$"#)
        .expect("Erreur création du regex CHAR_CONST_DROITE")
});

pub static NUMBER_CONST_GAUCHE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(-?[0-9]+(\.[0-9]+)?)\s*(=|<>|<=|>=|<|>)\s*([a-zA-Z_][a-zA-Z0-9_.-]*\.[a-zA-Z0-9_.-]+)$"#)
        .expect("Erreur création du regex NUMBER_CONST_GAUCHE")
});

pub static NUMBER_CONST_DROITE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^([a-zA-Z_][a-zA-Z0-9_.-]*\.[a-zA-Z0-9_.-]+)\s*(=|<>|<=|>=|<|>)\s*(-?[0-9]+(\.[0-9]+)?)$"#)
        .expect("Erreur création du regex NUMBER_CONST_DROITE")
});

pub static DEUX_CHAR_CONST: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(['"ʺ])([a-zA-Z0-9_-]+)\1\s*(=|<>|<=|>=|<|>)\s*(['"ʺ])([a-zA-Z0-9_-]+)\4$"#)
        .expect("Erreur création du regex DEUX_CHAR_CONST")
});

/*pub static DEUX_NUMBER_CONST: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(-?[0-9]+(\.[0-9]+)?)\s*(=|<>|<=|>=|<|>)\s*(-?[0-9]+(\.[0-9]+)?)$"#)
        .expect("Erreur création du regex DEUX_NUMBER_CONST")
});*/
pub static DEUX_NUMBER_CONST: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^\s*(-?[0-9]+(\.[0-9]+)?)\s*(=|<>|<=|>=|<|>)\s*(-?[0-9]+(\.[0-9]+)?)\s*$"#)
        .expect("Erreur création du regex DEUX_NUMBER_CONST")
});

impl Condition {
    fn new(gauche: Box<dyn Operande>, operateur: Operateur, droite: Box<dyn Operande>) -> Self {
        Condition {
            oper_gauche: gauche,
            operateur,
            oper_droite: droite,
        }
    }

    //TODO:On aura peut être besoin de refcell si ca ne marche pas car on ne peut pas utiliser 2 self sur la même ligne

    pub fn evaluate(&self) -> bool {
        let copie_oper_droite = self.oper_droite.clone_box();

        match self.operateur {
            Operateur::EQUAL => {
                return self.oper_gauche.compare(copie_oper_droite) == 0;
            }
            Operateur::LESSTHAN => {
                return self.oper_gauche.compare(copie_oper_droite) == -1;
            }
            Operateur::GREATERTHAN => {
                return self.oper_gauche.compare(copie_oper_droite) == 1;
            }
            Operateur::LESSEQUAL => {
                return self.oper_gauche.compare(copie_oper_droite) <= 0;
            }
            Operateur::GREATEREQUAL => {
                return self.oper_gauche.compare(copie_oper_droite) >= 0;
            }
            Operateur::NOTEQUAL => {
                return self.oper_gauche.compare(copie_oper_droite) != 0;
            }
        }
    }

    fn choisir_operande(
        colonnes: &Vec<ColInfo>,
        nom_colonne: &str,
        record: &Record,
    ) -> Result<Box<dyn Operande>, PatternError> {
        let index = colonnes
            .iter()
            .position(|col| col.get_name().eq(nom_colonne))
            .ok_or_else(|| PatternError::new("Unknown column"))?;

        if colonnes[index].get_column_type().eq("INT")
            || colonnes[index].get_column_type().eq("REAL")
        {
            Ok(Box::new(Number::new(record.get_tuple()[index].as_str())))
        } else {
            Ok(Box::new(Chars::new(record.get_tuple()[index].as_str())))
        }
    }

    pub fn check_syntaxe(
        s: String,
        colonnes: &Vec<ColInfo>,
        record: &Record,
    ) -> Result<Condition, PatternError> {
        //ATTENTION TRES ACROBATIQUE !!!
        //("odj-FF_"<>"Uwu_BA-KA")

        let ope_g: String;
        let operat: String;
        let ope_d: String;

        let mut vec_nom_colinfo: Vec<String> = Vec::new();
        for col in colonnes.iter() {
            vec_nom_colinfo.push(col.get_name().to_string());
        }

        if AUCUN_CONST.is_match(&s).unwrap() {
            //println!("CHECK AUCUN CONST");
            (ope_g, operat, ope_d) =
                Condition::split_condition_aucun_const(&s, &AUCUN_CONST).unwrap();

            //TODO: Trouver une Table grace à tableg grace à un iterator ou jsp
            let (_tableg, colonneg) = Condition::split_colonne(ope_g.as_str()).unwrap();
            let (_tabled, colonned) = Condition::split_colonne(ope_d.as_str()).unwrap();

            return Ok(Condition::new(
                Condition::choisir_operande(&colonnes, colonneg.as_str(), record)?,
                Condition::to_operateur(operat.as_str()).unwrap(),
                Condition::choisir_operande(&colonnes, colonned.as_str(), record)?,
            ));
        } else if CHAR_CONST_GAUCHE.is_match(&s).unwrap() {
            //println!("CHECK CHAR CONST GAUCHE");
            (ope_g, operat, ope_d) =
                Condition::split_condition_char_const_gauche(&s, &CHAR_CONST_GAUCHE).unwrap();

            let (_tabled, colonned) = Condition::split_colonne(ope_d.as_str()).unwrap();

            return Ok(Condition::new(
                Box::new(Chars::new(Condition::suppr_guillemets(ope_g.as_str()))),
                Condition::to_operateur(operat.as_str()).unwrap(),
                Condition::choisir_operande(&colonnes, colonned.as_str(), record)?,
            ));
        } else if CHAR_CONST_DROITE.is_match(&s).unwrap() {
            //println!("CHECK CHAR CONST DROITE");
            (ope_g, operat, ope_d) =
                Condition::split_condition_char_const_droite(&s, &CHAR_CONST_DROITE).unwrap();

            let (_tableg, colonneg) = Condition::split_colonne(ope_g.as_str()).unwrap();
            return Ok(Condition::new(
                Condition::choisir_operande(&colonnes, colonneg.as_str(), record)?,
                Condition::to_operateur(operat.as_str()).unwrap(),
                Box::new(Chars::new(Condition::suppr_guillemets(ope_d.as_str()))),
            ));
        } else if NUMBER_CONST_GAUCHE.is_match(&s).unwrap() {
            //println!("CHECK NUMBER CONST GAUCHE");
            (ope_g, operat, ope_d) =
                Condition::split_condition_number_const_gauche(&s, &NUMBER_CONST_GAUCHE).unwrap();

            let (_tabled, colonned) = Condition::split_colonne(ope_d.as_str()).unwrap();
            return Ok(Condition::new(
                Box::new(Number::new(ope_g.as_str())),
                Condition::to_operateur(operat.as_str()).unwrap(),
                Condition::choisir_operande(&colonnes, colonned.as_str(), record)?,
            ));
        } else if NUMBER_CONST_DROITE.is_match(&s).unwrap() {
            //println!("CHECK NUMBER CONST DROITE");
            (ope_g, operat, ope_d) =
                Condition::split_condition_number_const_droite(&s, &NUMBER_CONST_DROITE).unwrap();

            let (_tableg, colonneg) = Condition::split_colonne(ope_g.as_str()).unwrap();
            return Ok(Condition::new(
                Condition::choisir_operande(&colonnes, colonneg.as_str(), record)?,
                Condition::to_operateur(operat.as_str()).unwrap(),
                Box::new(Number::new(ope_d.as_str())),
            ));
        } else if DEUX_CHAR_CONST.is_match(&s).unwrap() {
            //println!("CHECK DEUX CHAR CONST");
            (ope_g, operat, ope_d) =
                Condition::split_condition_deux_char_const(&s, &DEUX_CHAR_CONST).unwrap();

            return Ok(Condition::new(
                Box::new(Chars::new(Condition::suppr_guillemets(ope_g.as_str()))),
                Condition::to_operateur(operat.as_str()).unwrap(),
                Box::new(Chars::new(Condition::suppr_guillemets(ope_d.as_str()))),
            ));
        } else if DEUX_NUMBER_CONST.is_match(&s).unwrap() {
            //println!("CHECK DEUX NUMBER CONST");
            (ope_g, operat, ope_d) =
                Condition::split_condition_deux_number_const(&s, &DEUX_NUMBER_CONST).unwrap();

            return Ok(Condition::new(
                Box::new(Number::new(ope_g.as_str())),
                Condition::to_operateur(operat.as_str()).unwrap(),
                Box::new(Number::new(ope_d.as_str())),
            ));
        } else {
            return Err(PatternError::new("Invalid syntax"));
        }
    }

    pub fn to_operateur(operateur_str: &str) -> Result<Operateur, PatternError> {
        match operateur_str {
            "=" => {
                return Ok(Operateur::EQUAL);
            }
            "<>" => {
                return Ok(Operateur::NOTEQUAL);
            }
            "<" => {
                return Ok(Operateur::LESSTHAN);
            }
            ">" => {
                return Ok(Operateur::GREATERTHAN);
            }
            "<=" => {
                return Ok(Operateur::LESSEQUAL);
            }
            ">=" => {
                return Ok(Operateur::GREATEREQUAL);
            }
            _ => return Err(PatternError::new("Invalid operator")),
        }
    }

    fn split_condition_aucun_const(
        condition: &str,
        regex: &Regex,
    ) -> Result<(String, String, String), String> {
        if let Some(captures) = regex.captures(condition).unwrap() {
            let operande_gauche: String = captures
                .get(1)
                .ok_or("Operande invalide ou manquant.")?
                .as_str()
                .to_string();
            let operateur: String = captures
                .get(2)
                .ok_or("Operateur ivalide ou manquant.")?
                .as_str()
                .to_string();
            let operande_droite: String = captures
                .get(3)
                .ok_or("Operande invalide ou manquant.")?
                .as_str()
                .to_string();
            Ok((operande_gauche, operateur, operande_droite))
        } else {
            //println!("{:?}", regex.captures(condition).unwrap());
            Err("Error: condition has an invalid format.".to_string())
        }
    }

    fn split_condition_char_const_gauche(
        condition: &str,
        regex: &Regex,
    ) -> Result<(String, String, String), String> {
        if let Some(captures) = regex.captures(condition).unwrap() {
            let operande_gauche: String = captures
                .get(2)
                .ok_or("Operande invalide ou manquant.")?
                .as_str()
                .to_string();
            let operateur: String = captures
                .get(3)
                .ok_or("Operateur ivalide ou manquant.")?
                .as_str()
                .to_string();
            let operande_droite: String = captures
                .get(4)
                .ok_or("Operande invalide ou manquant.")?
                .as_str()
                .to_string();
            Ok((operande_gauche, operateur, operande_droite))
        } else {
            //println!("{:?}", regex.captures(condition).unwrap());
            Err("Error: condition has an invalid format.".to_string())
        }
    }

    fn split_condition_char_const_droite(
        condition: &str,
        regex: &Regex,
    ) -> Result<(String, String, String), String> {
        if let Some(captures) = regex.captures(condition).unwrap() {
            let operande_gauche: String = captures
                .get(1)
                .ok_or("Operande invalide ou manquant.")?
                .as_str()
                .to_string();
            let operateur: String = captures
                .get(2)
                .ok_or("Operateur ivalide ou manquant.")?
                .as_str()
                .to_string();
            let operande_droite: String = captures
                .get(4)
                .ok_or("Operande invalide ou manquant.")?
                .as_str()
                .to_string();
            Ok((operande_gauche, operateur, operande_droite))
        } else {
            //println!("{:?}", regex.captures(condition).unwrap());
            Err("Error: condition has an invalid format.".to_string())
        }
    }

    fn split_condition_number_const_droite(
        condition: &str,
        regex: &Regex,
    ) -> Result<(String, String, String), String> {
        if let Some(captures) = regex.captures(condition).unwrap() {
            let operande_gauche: String = captures
                .get(1)
                .ok_or("Operande invalide ou manquant.")?
                .as_str()
                .to_string();
            //println!("operande_gauche: {:?}",operande_gauche);
            let operateur: String = captures
                .get(2)
                .ok_or("Operateur ivalide ou manquant.")?
                .as_str()
                .to_string();
            //println!("operateur: {:?}",operateur);
            let operande_droite: String = captures
                .get(3)
                .ok_or("Operande invalide ou manquant.")?
                .as_str()
                .to_string();
            //println!("operande_droite: {:?}",operande_droite);
            Ok((operande_gauche, operateur, operande_droite))
        } else {
            //println!("{:?}", regex.captures(condition).unwrap());
            Err("Error: condition has an invalid format.".to_string())
        }
    }
    fn split_condition_number_const_gauche(
        condition: &str,
        regex: &Regex,
    ) -> Result<(String, String, String), String> {
        if let Some(captures) = regex.captures(condition).unwrap() {
            let operande_gauche: String = captures
                .get(1)
                .ok_or("Operande invalide ou manquant.")?
                .as_str()
                .to_string();
            //println!("operande_gauche: {:?}",operande_gauche);
            let operateur: String = captures
                .get(3)
                .ok_or("Operateur ivalide ou manquant.")?
                .as_str()
                .to_string();
            //println!("operateur: {:?}",operateur);
            let operande_droite: String = captures
                .get(4)
                .ok_or("Operande invalide ou manquant.")?
                .as_str()
                .to_string();
            //println!("operande_droite: {:?}",operande_droite);
            Ok((operande_gauche, operateur, operande_droite))
        } else {
            //println!("{:?}", regex.captures(condition).unwrap());
            Err("Error: condition has an invalid format.".to_string())
        }
    }

    fn split_condition_deux_number_const(
        condition: &str,
        regex: &Regex,
    ) -> Result<(String, String, String), String> {
        if let Some(captures) = regex.captures(condition).unwrap() {
            let operande_gauche: String = captures
                .get(1)
                .ok_or("Operande invalide ou manquant.")?
                .as_str()
                .to_string();
            //println!("operande_gauche: {:?}",operande_gauche);
            let operateur: String = captures
                .get(3)
                .ok_or("Operateur ivalide ou manquant.")?
                .as_str()
                .to_string();
            //println!("operateur: {:?}",operateur);
            let operande_droite: String = captures
                .get(4)
                .ok_or("Operande invalide ou manquant.")?
                .as_str()
                .to_string();
            //println!("operande_droite: {:?}",operande_droite);
            Ok((operande_gauche, operateur, operande_droite))
        } else {
            //println!("{:?}", regex.captures(condition).unwrap());
            Err("Error: condition has an invalid format.".to_string())
        }
    }

    fn split_condition_deux_char_const(
        condition: &str,
        regex: &Regex,
    ) -> Result<(String, String, String), String> {
        if let Some(captures) = regex.captures(condition).unwrap() {
            let operande_gauche: String = captures
                .get(2)
                .ok_or("Operande invalide ou manquant.")?
                .as_str()
                .to_string();
            let operateur: String = captures
                .get(3)
                .ok_or("Operateur ivalide ou manquant.")?
                .as_str()
                .to_string();
            let operande_droite: String = captures
                .get(5)
                .ok_or("Operande invalide ou manquant.")?
                .as_str()
                .to_string();
            Ok((operande_gauche, operateur, operande_droite))
        } else {
            //println!("{:?}", regex.captures(condition).unwrap());
            Err("Error: condition has an invalid format.".to_string())
        }
    }

    pub fn split_colonne(s: &str) -> Result<(String, String), PatternError> {
        //tablealias.colonne -> tablealias   colonne
        match s.split_once('.') {
            Some((gauche, droite)) => {
                return Ok((gauche.to_string(), droite.to_string()));
            }
            None => {
                return Err(PatternError::new("Erreur split_condition()"));
            }
        }
    }

    //PEUT ETRE INUTILE ! Mais laisser pour le moment
    fn suppr_guillemets(s: &str) -> &str {
        //Suppression des guillemets autour d'un mot
        //A n'utiliser que pour les constantes

        if let Some(mot_sans_guill) = s.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
            mot_sans_guill
        } else if let Some(mot_sans_guill) = s.strip_prefix('\'').and_then(|s| s.strip_suffix('\''))
        {
            mot_sans_guill
        } else if let Some(mot_sans_guill) = s.strip_prefix('ʺ').and_then(|s| s.strip_suffix('ʺ'))
        {
            mot_sans_guill
        } else {
            s
        }
    }

    fn get_operateur(&self) -> &str {
        match self.operateur {
            Operateur::EQUAL => "=",
            Operateur::NOTEQUAL => "<>",
            Operateur::LESSTHAN => "<",
            Operateur::GREATERTHAN => ">",
            Operateur::GREATEREQUAL => ">=",
            Operateur::LESSEQUAL => "<=",
            _ => "NONE", //TODO: Gérer l'erreur si vraiment nécessaire
        }
    }

    pub fn to_string(&self) -> String {
        return format!(
            "Condition {{ oper_gauche={}, operateur={}, oper_droite={} }}",
            self.oper_gauche.get_valeur(),
            self.get_operateur(),
            self.oper_droite.get_valeur()
        );
    }
}

#[cfg(test)]

mod tests {
    use crate::buffer_manager::BufferManager;
    use crate::condition::*;
    use crate::config::DBConfig;
    use crate::disk_manager::DiskManager;
    use crate::relation::Relation;
    use std::cell::RefCell;
    use std::cmp::PartialEq;
    use std::rc::Rc;

    #[test]
    pub fn test1() {
        let s: String = String::from("config.json");
        let config = DBConfig::load_db_config(s);
        let dm = DiskManager::new(&config);
        let algo_lru = String::from("LRU");

        let buffer_manager = Rc::new(RefCell::new(BufferManager::new(&config, dm, algo_lru)));

        let colinfo: Vec<ColInfo> = vec![
            ColInfo::new("NOM".to_string(), "VARCHAR(20)".to_string()),
            ColInfo::new("PRENOM".to_string(), "VARCHAR(20)".to_string()),
            ColInfo::new("AGE".to_string(), "INT".to_string()),
        ];
        let relation = Relation::new("PERSONNE".to_string(), colinfo.clone(), buffer_manager);

        let record = Record::new(vec![
            "GNAHO".to_string(),
            "CHRISTOPHE".to_string(),
            "50".to_string(),
        ]);

        let condition: Result<Condition, PatternError> =
            Condition::check_syntaxe("26=12".to_string(), &relation.get_columns(), &record);
        if condition.is_ok() {
            println!("{:?}", condition.unwrap().to_string());
        } else {
            println!("{:?}", condition.err().unwrap());
        }
        //println!("{:?}",Condition::split_condition_char_const_gauche("\"123_o\"<=table.NOM",&CHAR_CONST_GAUCHE));
        //println!("{:?}",AUCUN_CONST.is_match("\"123_o\"<=table.NOM").unwrap());
    }

    #[test]
    fn test_split_condition_aucun_const() {
        let condition = "table1.col1 = table2.col2";
        let regex = &AUCUN_CONST;
        let result = Condition::split_condition_aucun_const(condition, regex);
        assert!(result.is_ok());
        let (left, op, right) = result.unwrap();
        assert_eq!(left, "table1.col1");
        assert_eq!(op, "=");
        assert_eq!(right, "table2.col2");
    }

    #[test]
    fn test_split_condition_char_const_gauche() {
        let condition = "'value' = table.col";
        let regex = &CHAR_CONST_GAUCHE;
        let result = Condition::split_condition_char_const_gauche(condition, regex);
        assert!(result.is_ok());
        let (left, op, right) = result.unwrap();
        assert_eq!(left, "value");
        assert_eq!(op, "=");
        assert_eq!(right, "table.col");
    }

    #[test]
    fn test_split_condition_deux_char_const() {
        let condition = "'value1' = 'value2'";
        let regex = &DEUX_CHAR_CONST;
        let result = Condition::split_condition_deux_char_const(condition, regex);
        assert!(result.is_ok());
        let (left, op, right) = result.unwrap();
        assert_eq!(left, "value1");
        assert_eq!(op, "=");
        assert_eq!(right, "value2");
    }

    #[test]
    fn test_split_colonne() {
        let col = "table.col";
        let result = Condition::split_colonne(col);
        assert!(result.is_ok());
        let (table, col) = result.unwrap();
        assert_eq!(table, "table");
        assert_eq!(col, "col");
    }

    #[test]
    fn test_suppr_guillemets() {
        let with_quotes = "'value'";
        let result = Condition::suppr_guillemets(with_quotes);
        assert_eq!(result, "value");
    }

    impl PartialEq for Operateur {
        fn eq(&self, _other: &Self) -> bool {
            match self {
                _other => true,
            }
        }
    }

    #[test]
    fn test_to_operateur() {
        let op_str = "=";
        let result = Condition::to_operateur(op_str);
        assert!(result.is_ok());
        let op = result.unwrap();
        assert!(op == Operateur::EQUAL);
    }

    #[test]
    fn test_evaluate() {
        let left = Box::new(Number::new("10"));
        let right = Box::new(Number::new("20"));
        let condition = Condition::new(left, Operateur::LESSTHAN, right);
        assert!(condition.evaluate());
    }

    #[test]
    fn test_check_syntaxe_valid_conditions() {
        let s: String = String::from("config.json");
        let config = DBConfig::load_db_config(s);
        let dm = DiskManager::new(&config);
        let algo_lru = String::from("LRU");

        let buffer_manager = Rc::new(RefCell::new(BufferManager::new(&config, dm, algo_lru)));

        let colinfo: Vec<ColInfo> = vec![
            ColInfo::new("NOM".to_string(), "VARCHAR(20)".to_string()),
            ColInfo::new("PRENOM".to_string(), "VARCHAR(20)".to_string()),
            ColInfo::new("AGE".to_string(), "INT".to_string()),
        ];
        let relation = Relation::new("PERSONNE".to_string(), colinfo.clone(), buffer_manager);

        let record = Record::new(vec![
            "GNAHO".to_string(),
            "CHRISTOPHE".to_string(),
            "50".to_string(),
        ]);

        let conditions = vec![
            ("'Chris'=table.NOM", true),
            ("table.AGE>30", true),
            ("26<=table.AGE", true),
            ("-12.3<=table.AGE", true),
            ("'inconnu'<>'connu'", true),
        ];

        for (cond_str, should_succeed) in conditions {
            let condition_result =
                Condition::check_syntaxe(cond_str.to_string(), &relation.get_columns(), &record);
            if should_succeed {
                assert!(
                    condition_result.is_ok(),
                    "Condition `{}` should succeed.",
                    cond_str
                );
            } else {
                assert!(
                    condition_result.is_err(),
                    "Condition `{}` should fail.",
                    cond_str
                );
            }
        }
    }

    #[test]
    fn test_evaluate_various_conditions() {
        let left_number = Box::new(Number::new("10"));
        let right_number = Box::new(Number::new("20"));
        let condition1 = Condition::new(left_number, Operateur::LESSTHAN, right_number);
        assert!(condition1.evaluate(), "Attendu : 10 < 20 doit être true.");

        let left_string = Box::new(Chars::new("hello"));
        let right_string = Box::new(Chars::new("world"));
        let condition2 = Condition::new(left_string, Operateur::NOTEQUAL, right_string);
        assert!(
            condition2.evaluate(),
            "Attendu : 'hello' <> 'world' doit être true."
        );

        let left_number = Box::new(Number::new("100"));
        let right_number = Box::new(Number::new("100"));
        let condition3 = Condition::new(left_number, Operateur::EQUAL, right_number);
        assert!(condition3.evaluate(), "Attendu : 100 = 100 doit être true.");
    }

    #[test]
    fn test_regex_patterns() {
        let regexs = vec![
            (&AUCUN_CONST, "table1.col1 = table2.col2", true),
            (&CHAR_CONST_GAUCHE, "'value' = table.col", true),
            (&CHAR_CONST_DROITE, "table.col = 'value'", true),
            (&NUMBER_CONST_GAUCHE, "-12.3 <= table.col", true),
            (&NUMBER_CONST_DROITE, "table.col >= -99.99", true),
            (&DEUX_CHAR_CONST, "'hello' <> 'world'", true),
            (&DEUX_NUMBER_CONST, "-50 < -10", true),
        ];

        for (regex, input, attente) in regexs {
            match regex.is_match(input) {
                Ok(matches) => {
                    assert_eq!(
                        matches,
                        attente,
                        "Regex '{}' echec pour '{}'",
                        regex.as_str(),
                        input
                    );
                }
                Err(err) => {
                    panic!(
                        "Regex '{}' echec pour '{}': {:?}",
                        regex.as_str(),
                        input,
                        err
                    );
                }
            }
        }
    }

    #[test]
    fn test_to_string_condition() {
        let left = Box::new(Number::new("25"));
        let right = Box::new(Number::new("50"));
        let condition = Condition::new(left, Operateur::LESSTHAN, right);

        let string_repr = condition.to_string();
        assert_eq!(
            string_repr,
            "Condition { oper_gauche=25, operateur=<, oper_droite=50 }"
        );
    }

    #[test]
    fn test_evaluate2() {
        let s: String = String::from("config.json");
        let config = DBConfig::load_db_config(s);
        let dm = DiskManager::new(&config);
        let algo_lru = String::from("LRU");

        let buffer_manager = Rc::new(RefCell::new(BufferManager::new(&config, dm, algo_lru)));

        let colinfo: Vec<ColInfo> = vec![
            ColInfo::new("NOM".to_string(), "VARCHAR(20)".to_string()),
            ColInfo::new("PRENOM".to_string(), "VARCHAR(20)".to_string()),
            ColInfo::new("AGE".to_string(), "INT".to_string()),
        ];
        let relation = Relation::new("PERSONNE".to_string(), colinfo.clone(), buffer_manager);

        let record = Record::new(vec![
            "GNAHO".to_string(),
            "CHRISTOPHE".to_string(),
            "50".to_string(),
        ]);

        let v = vec![
            "PERSONNE.AGE = PERSONNE.AGE",
            "'value' = table.col",
            "PERSONNE.PRENOM = 'value",
            "-12.3 <= table.col",
            "PERSONNE.AGE >= -99.99",
            "'hello' <> 'world",
            "-50 < -10",
        ];

        for r in v.iter() {
            let condition: Result<Condition, PatternError> =
                Condition::check_syntaxe(r.to_string(), &relation.get_columns(), &record);
            if condition.is_ok() {
                println!("{:?}", condition.unwrap().to_string());
            } else {
                println!("{:?}", condition.err().unwrap());
            }
        }
    }
}
