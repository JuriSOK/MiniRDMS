use fancy_regex::Regex;
use std::{collections::HashSet, str};

use crate::col_info::ColInfo;
use crate::condition::Condition;
use crate::operator::ERRORS;
use crate::record::Record;

#[derive(Debug, Clone)]
pub struct Select {
    tables: Vec<String>,
    columns: Vec<String>,
    conditions: Vec<String>,
}

impl Select {
    pub fn new(command: &str) -> Result<Select, String> {
        let sep_command = Select::split_command(command);
        if sep_command.is_err() {
            return Err(sep_command.err().unwrap().to_string());
        }
        let (columns, tables, conditions) = sep_command.unwrap();

        let res = Select {
            tables,
            columns,
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
    pub fn get_columns(&self) -> &Vec<String> {
        &self.columns
    }

    pub fn to_string(&self) -> String {
        format!(
            "SELECT {}\nFROM {}\nWHERE {}",
            self.columns.join(", "),
            self.tables.join(", "),
            self.conditions.join(" AND ")
        )
    }

    fn split_command(command: &str) -> Result<(Vec<String>, Vec<String>, Vec<String>), String> {
        // Parse each clause independently so malformed SELECT/FROM/WHERE input can be rejected early.
        let select_regex = Regex::new(r"(?i)\bselect\b\s+(.+?)\s+\bfrom\b").unwrap();
        let from_regex = Regex::new(r"(?i)\bfrom\b\s+(.+?)(?:\s+\bwhere\b|$)").unwrap();
        let where_regex = Regex::new(r"(?i)\bwhere\b\s+(.+)").unwrap();

        let select_bloc: &str = select_regex
            .captures(command)
            .ok()
            .flatten()
            .and_then(|capture| capture.get(1).map(|match_| match_.as_str()))
            .unwrap_or("");
        let from_bloc: &str = from_regex
            .captures(command)
            .ok()
            .flatten()
            .and_then(|capture| capture.get(1).map(|match_| match_.as_str()))
            .unwrap_or("");
        let where_bloc: &str = where_regex
            .captures(command)
            .ok()
            .flatten()
            .and_then(|capture| capture.get(1).map(|match_| match_.as_str()))
            .unwrap_or("");

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

        let table_with_alias_regex = Regex::new(r"(?i)^[a-zA-Z0-9_.-]+\s+[a-zA-Z0-9_]+$").unwrap();

        for table in &from_elements {
            if !table_with_alias_regex.is_match(table).unwrap() {
                return Err(format!("Error: table '{}' must define an alias.", table));
            }
        }

        if command.to_uppercase().contains("WHERE") && where_bloc.is_empty() {
            return Err("Error: WHERE clause is malformed or empty.".to_string());
        }

        Ok((select_elements, from_elements, where_elements))
    }

    pub fn check_alias(&self) -> Result<(), String> {
        let mut from_aliases: HashSet<String> = HashSet::new();
        for table in &self.tables {
            if let Some((_, alias)) = table.split_once(' ') {
                from_aliases.insert(alias.trim().to_string());
            } else {
                from_aliases.insert(table.trim().to_string());
            }
        }

        for column in &self.columns {
            if let Some((alias, _)) = column.split_once('.') {
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
        columns: &Vec<ColInfo>,
        record: &Record,
    ) -> Result<Vec<Condition>, String> {
        let mut vec_cond: Vec<Condition> = Vec::new();
        for condition in &self.conditions {
            let cond = Condition::check_syntax(condition.clone(), columns, record);
            if cond.is_err() {
                let error = cond.err().unwrap().to_string();
                unsafe { ERRORS.push(error.clone()) };
                return Err(error.clone());
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
    fn test_missing_alias() {
        let command = "SELECT t1.col1, table2.col2 FROM table1 t1, table2";
        let res = Select::new(command);
        assert!(
            res.is_err(),
            "Expected an error because table2 has no alias."
        );
    }

    #[test]
    fn test_invalid_select_alias() {
        let command = "SELECT t3.col1, t2.col2 FROM table1 t1, table2 t2";
        let res = Select::new(command);
        assert!(
            res.is_err(),
            "Expected an error because t3 is not defined in FROM."
        );
    }

    #[test]
    fn test_command_malformed_without_from() {
        let command = "SELECT col1";
        let res = Select::new(command);
        assert!(res.is_err(), "Expected an error because FROM is missing.");
    }

    #[test]
    fn test_command_without_where() {
        let command = "SELECT t1.col1 FROM table1 t1, table2 t2";
        let res = Select::new(command);
        assert!(
            res.is_err(),
            "Expected an error for this command without WHERE."
        );
    }

    #[test]
    fn test_empty_command() {
        let command = "";
        let res = Select::new(command);
        assert!(res.is_err(), "Expected an error for an empty command.");
    }

    #[test]
    fn test_malformed_from_with_where() {
        let command = "SELECT t1.col1 FROM table1 t1 WHERE";
        let res = Select::new(command);
        assert!(
            res.is_err(),
            "Expected an error because WHERE has no condition."
        );
    }
}
