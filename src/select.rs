use fancy_regex::Regex;
use std::{collections::HashSet, str};

use crate::col_info::ColInfo;
use crate::condition::Condition;
use crate::operator::ERREURS;
use crate::record::Record;

#[derive(Debug, Clone)]
pub struct Select {
    tables: Vec<String>,
    colonnes: Vec<String>,
    conditions: Vec<String>,
}

impl Select {
    pub fn new(commande: &str) -> Result<Select, String> {
        let sep_commande = Select::split_commande(commande);
        if sep_commande.is_err() {
            return Err(sep_commande.err().unwrap().to_string());
        }
        let (colonnes, tables, conditions) = sep_commande.unwrap();

        let res = Select {
            tables,
            colonnes,
            conditions,
        };
        if res.check_alias().is_err() {
            return Err(res.check_alias().err().unwrap());
        }
        Ok(res)
    }

    pub fn get_tables(&self) -> &Vec<String> {
        &self.tables
    }
    pub fn get_colonnes(&self) -> &Vec<String> {
        &self.colonnes
    }

    pub fn to_string(&self) -> String {
        format!(
            "SELECT {}\nFROM {}\nWHERE {}",
            self.colonnes.join(", "),
            self.tables.join(", "),
            self.conditions.join(" AND ")
        )
    }

    fn split_commande(commande: &str) -> Result<(Vec<String>, Vec<String>, Vec<String>), String> {
        //Regex pour capturer les blocs SELECT, FROM, WHERE
        let select_regex = Regex::new(r"(?i)\bselect\b\s+(.+?)\s+\bfrom\b").unwrap();
        let from_regex = Regex::new(r"(?i)\bfrom\b\s+(.+?)(?:\s+\bwhere\b|$)").unwrap();
        let where_regex = Regex::new(r"(?i)\bwhere\b\s+(.+)").unwrap();

        // Extraire les blocs correspondants
        let select_bloc: &str = select_regex
            .captures(commande)
            .ok()
            .flatten()
            .and_then(|capture| capture.get(1).map(|match_| match_.as_str()))
            .unwrap_or("");
        let from_bloc: &str = from_regex
            .captures(commande)
            .ok()
            .flatten()
            .and_then(|capture| capture.get(1).map(|match_| match_.as_str()))
            .unwrap_or("");
        let where_bloc: &str = where_regex
            .captures(commande)
            .ok()
            .flatten()
            .and_then(|capture| capture.get(1).map(|match_| match_.as_str()))
            .unwrap_or("");

        // Split des blocs
        let select_elements: Vec<String> = select_bloc
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        let from_elements: Vec<String> =
            from_bloc.split(',').map(|s| s.trim().to_string()).collect();
        let where_elements: Vec<String> = where_bloc
            .split_terminator("AND")
            .map(|s| s.trim().to_string())
            .collect();

        if select_elements[0].eq("") || from_elements[0].eq("") || from_elements.len() > 1 {
            return Err("Invalid or missing operand.".to_string());
        }
        if from_elements
            .iter()
            .any(|table| table.to_uppercase().contains("WHERE"))
            && where_bloc.is_empty()
        {
            return Err("Error: malformed FROM clause.".to_string());
        }
        // Vérification : chaque table dans FROM doit avoir un alias
        // table alias
        let table_with_alias_regex = Regex::new(r"(?i)^[a-zA-Z0-9_.-]+\s+[a-zA-Z0-9_]+$").unwrap();

        for table in &from_elements {
            if !table_with_alias_regex.is_match(table).unwrap() {
                return Err(format!("Error: table '{}' must define an alias.", table));
            }
        }

        // Vérifie si WHERE est mal formé
        if commande.to_uppercase().contains("WHERE") && where_bloc.is_empty() {
            return Err("Error: WHERE clause is malformed or empty.".to_string());
        }

        Ok((select_elements, from_elements, where_elements))
    }

    pub fn check_alias(&self) -> Result<(), String> {
        // Extraire les alias des tables dans FROM
        let mut from_aliases: HashSet<String> = HashSet::new();
        for table in &self.tables {
            //Vérifie s'il y a un alias ("table alias")
            if let Some((_, alias)) = table.split_once(' ') {
                from_aliases.insert(alias.trim().to_string());
            } else {
                // Si aucune alias explicite, utiliser le nom de la table
                from_aliases.insert(table.trim().to_string());
            }
        }

        // Vérifie si chaque colonne utilise un alias valide
        for colonne in &self.colonnes {
            if let Some((alias, _)) = colonne.split_once('.') {
                if !from_aliases.contains(alias.trim()) {
                    return Err(format!(
                        "Error: SELECT alias \"{}\" is not defined in FROM.",
                        alias.trim()
                    ));
                }
            }
        }

        Ok(())
    }

    pub fn get_list_conditions(
        &self,
        colonnes: &Vec<ColInfo>,
        record: &Record,
    ) -> Result<Vec<Condition>, String> {
        let mut vec_cond: Vec<Condition> = Vec::new();
        for condition in &self.conditions {
            // Vérifie si chaque colonne utilise un alias valide
            let cond = Condition::check_syntaxe(condition.clone(), colonnes, record);
            if cond.is_err() {
                let erreur = cond.err().unwrap().to_string();
                unsafe { ERREURS.push(erreur.clone()) };
                return Err(erreur.clone());
            }
            vec_cond.push(cond.unwrap());
        }
        Ok(vec_cond)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alias_manquant() {
        let commande = "SELECT t1.col1, table2.col2 FROM table1 t1, table2";
        let res = Select::new(commande);
        assert!(
            res.is_err(),
            "Une erreur devrait se produire car table2 n'a pas d'alias."
        );
    }

    #[test]
    fn test_alias_invalide_dans_select() {
        let commande = "SELECT t3.col1, t2.col2 FROM table1 t1, table2 t2";
        let res = Select::new(commande);
        assert!(
            res.is_err(),
            "Une erreur devrait se produire car t3 n'est pas défini dans FROM."
        );
    }

    #[test]
    fn test_commande_mal_formee_sans_from() {
        let commande = "SELECT col1";
        let res = Select::new(commande);
        assert!(
            res.is_err(),
            "Une erreur devrait se produire car FROM est manquant."
        );
    }

    #[test]
    fn test_commande_valide_sans_where() {
        let commande = "SELECT t1.col1 FROM table1 t1, table2 t2";
        let res = Select::new(commande);
        assert!(
            res.is_err(),
            "La commande sans WHERE est valide si les alias sont définis."
        );
    }

    #[test]
    fn test_commande_vide() {
        let commande = "";
        let res = Select::new(commande);
        assert!(
            res.is_err(),
            "Une erreur devrait se produire pour une commande vide."
        );
    }

    #[test]
    fn test_from_mal_forme_avec_where() {
        let commande = "SELECT t1.col1 FROM table1 t1 WHERE";
        let res = Select::new(commande);
        assert!(
            res.is_err(),
            "Une erreur devrait se produire car WHERE est mal placé sans condition."
        );
    }
}
