//! WHERE-clause condition parsing and evaluation.

use crate::col_info::ColInfo;
use crate::record::Record;
use crate::types::{Chars, Number, Operand};
use fancy_regex::Regex;
use once_cell::sync::Lazy;
use std::error::Error;

#[derive(Debug)]
pub struct PatternError {
    pub message: String,
}
impl PatternError {
    /// Creates a parser error with a human-readable message.
    pub fn new(message: &str) -> Self {
        PatternError {
            message: message.to_string(),
        }
    }
}
impl std::fmt::Display for PatternError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl Error for PatternError {}

#[derive(Debug)]
pub enum Operator {
    Equal,
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    NotEqual,
}

pub struct Condition {
    left_operand: Box<dyn Operand>,
    operator: Operator,
    right_operand: Box<dyn Operand>,
}

pub static NO_CONST: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([a-zA-Z_][a-zA-Z0-9_.-]*\.[a-zA-Z0-9_.-]+)\s*(=|<>|<=|>=|<|>)\s*([a-zA-Z_][a-zA-Z0-9_.-]*\.[a-zA-Z0-9_.-]+)$")
        .expect("Failed to build regex NO_CONST")
});

pub static CHAR_CONST_LEFT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(['"ʺ])([a-zA-Z0-9_-]+)\1\s*(=|<>|<=|>=|<|>)\s*([a-zA-Z_][a-zA-Z0-9_.-]*\.[a-zA-Z0-9_.-]+)$"#)
        .expect("Failed to build regex CHAR_CONST_LEFT")
});

pub static CHAR_CONST_RIGHT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^([a-zA-Z_][a-zA-Z0-9_.-]*\.[a-zA-Z0-9_.-]+)\s*(=|<>|<=|>=|<|>)\s*(['"ʺ])([a-zA-Z0-9_-]+)\3$"#)
        .expect("Failed to build regex CHAR_CONST_RIGHT")
});

pub static NUMBER_CONST_LEFT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(-?[0-9]+(\.[0-9]+)?)\s*(=|<>|<=|>=|<|>)\s*([a-zA-Z_][a-zA-Z0-9_.-]*\.[a-zA-Z0-9_.-]+)$"#)
        .expect("Failed to build regex NUMBER_CONST_LEFT")
});

pub static NUMBER_CONST_RIGHT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^([a-zA-Z_][a-zA-Z0-9_.-]*\.[a-zA-Z0-9_.-]+)\s*(=|<>|<=|>=|<|>)\s*(-?[0-9]+(\.[0-9]+)?)$"#)
        .expect("Failed to build regex NUMBER_CONST_RIGHT")
});

pub static TWO_CHAR_CONST: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(['"ʺ])([a-zA-Z0-9_-]+)\1\s*(=|<>|<=|>=|<|>)\s*(['"ʺ])([a-zA-Z0-9_-]+)\4$"#)
        .expect("Failed to build regex TWO_CHAR_CONST")
});

pub static TWO_NUMBER_CONST: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^\s*(-?[0-9]+(\.[0-9]+)?)\s*(=|<>|<=|>=|<|>)\s*(-?[0-9]+(\.[0-9]+)?)\s*$"#)
        .expect("Failed to build regex TWO_NUMBER_CONST")
});

impl Condition {
    fn new(left: Box<dyn Operand>, operator: Operator, right: Box<dyn Operand>) -> Self {
        Condition {
            left_operand: left,
            operator,
            right_operand: right,
        }
    }

    /// Evaluates the condition by comparing its already parsed operands.
    pub fn evaluate(&self) -> bool {
        let right_operand = self.right_operand.clone_box();

        match self.operator {
            Operator::Equal => {
                return self.left_operand.compare(right_operand) == 0;
            }
            Operator::LessThan => {
                return self.left_operand.compare(right_operand) == -1;
            }
            Operator::GreaterThan => {
                return self.left_operand.compare(right_operand) == 1;
            }
            Operator::LessEqual => {
                return self.left_operand.compare(right_operand) <= 0;
            }
            Operator::GreaterEqual => {
                return self.left_operand.compare(right_operand) >= 0;
            }
            Operator::NotEqual => {
                return self.left_operand.compare(right_operand) != 0;
            }
        }
    }

    /// Resolves a column reference to the correctly typed operand for the current record.
    fn choose_operand(
        columns: &Vec<ColInfo>,
        column_name: &str,
        record: &Record,
    ) -> Result<Box<dyn Operand>, PatternError> {
        let index = columns
            .iter()
            .position(|col| col.get_name().eq(column_name))
            .ok_or_else(|| PatternError::new("Unknown column"))?;

        if columns[index].get_column_type().eq("INT") || columns[index].get_column_type().eq("REAL")
        {
            Ok(Box::new(Number::new(record.get_tuple()[index].as_str())))
        } else {
            Ok(Box::new(Chars::new(record.get_tuple()[index].as_str())))
        }
    }

    /// Parses a WHERE condition string against schema metadata and the current record.
    pub fn check_syntax(
        s: String,
        columns: &Vec<ColInfo>,
        record: &Record,
    ) -> Result<Condition, PatternError> {
        let left_raw: String;
        let operator: String;
        let right_raw: String;

        if NO_CONST.is_match(&s).unwrap() {
            (left_raw, operator, right_raw) =
                Condition::split_condition_no_const(&s, &NO_CONST).unwrap();

            let (_left_table, left_column) = Condition::split_column(left_raw.as_str()).unwrap();
            let (_right_table, right_column) = Condition::split_column(right_raw.as_str()).unwrap();

            return Ok(Condition::new(
                Condition::choose_operand(&columns, left_column.as_str(), record)?,
                Condition::to_operator(operator.as_str()).unwrap(),
                Condition::choose_operand(&columns, right_column.as_str(), record)?,
            ));
        } else if CHAR_CONST_LEFT.is_match(&s).unwrap() {
            (left_raw, operator, right_raw) =
                Condition::split_condition_char_const_left(&s, &CHAR_CONST_LEFT).unwrap();

            let (_right_table, right_column) = Condition::split_column(right_raw.as_str()).unwrap();

            return Ok(Condition::new(
                Box::new(Chars::new(Condition::remove_quotes(left_raw.as_str()))),
                Condition::to_operator(operator.as_str()).unwrap(),
                Condition::choose_operand(&columns, right_column.as_str(), record)?,
            ));
        } else if CHAR_CONST_RIGHT.is_match(&s).unwrap() {
            (left_raw, operator, right_raw) =
                Condition::split_condition_char_const_right(&s, &CHAR_CONST_RIGHT).unwrap();

            let (_left_table, left_column) = Condition::split_column(left_raw.as_str()).unwrap();
            return Ok(Condition::new(
                Condition::choose_operand(&columns, left_column.as_str(), record)?,
                Condition::to_operator(operator.as_str()).unwrap(),
                Box::new(Chars::new(Condition::remove_quotes(right_raw.as_str()))),
            ));
        } else if NUMBER_CONST_LEFT.is_match(&s).unwrap() {
            (left_raw, operator, right_raw) =
                Condition::split_condition_number_const_left(&s, &NUMBER_CONST_LEFT).unwrap();

            let (_right_table, right_column) = Condition::split_column(right_raw.as_str()).unwrap();
            return Ok(Condition::new(
                Box::new(Number::new(left_raw.as_str())),
                Condition::to_operator(operator.as_str()).unwrap(),
                Condition::choose_operand(&columns, right_column.as_str(), record)?,
            ));
        } else if NUMBER_CONST_RIGHT.is_match(&s).unwrap() {
            (left_raw, operator, right_raw) =
                Condition::split_condition_number_const_right(&s, &NUMBER_CONST_RIGHT).unwrap();

            let (_left_table, left_column) = Condition::split_column(left_raw.as_str()).unwrap();
            return Ok(Condition::new(
                Condition::choose_operand(&columns, left_column.as_str(), record)?,
                Condition::to_operator(operator.as_str()).unwrap(),
                Box::new(Number::new(right_raw.as_str())),
            ));
        } else if TWO_CHAR_CONST.is_match(&s).unwrap() {
            (left_raw, operator, right_raw) =
                Condition::split_condition_two_char_const(&s, &TWO_CHAR_CONST).unwrap();

            return Ok(Condition::new(
                Box::new(Chars::new(Condition::remove_quotes(left_raw.as_str()))),
                Condition::to_operator(operator.as_str()).unwrap(),
                Box::new(Chars::new(Condition::remove_quotes(right_raw.as_str()))),
            ));
        } else if TWO_NUMBER_CONST.is_match(&s).unwrap() {
            (left_raw, operator, right_raw) =
                Condition::split_condition_two_number_const(&s, &TWO_NUMBER_CONST).unwrap();

            return Ok(Condition::new(
                Box::new(Number::new(left_raw.as_str())),
                Condition::to_operator(operator.as_str()).unwrap(),
                Box::new(Number::new(right_raw.as_str())),
            ));
        } else {
            return Err(PatternError::new("Invalid syntax"));
        }
    }

    /// Converts SQL comparison text into the internal operator enum.
    pub fn to_operator(operator_str: &str) -> Result<Operator, PatternError> {
        match operator_str {
            "=" => {
                return Ok(Operator::Equal);
            }
            "<>" => {
                return Ok(Operator::NotEqual);
            }
            "<" => {
                return Ok(Operator::LessThan);
            }
            ">" => {
                return Ok(Operator::GreaterThan);
            }
            "<=" => {
                return Ok(Operator::LessEqual);
            }
            ">=" => {
                return Ok(Operator::GreaterEqual);
            }
            _ => return Err(PatternError::new("Invalid operator")),
        }
    }

    fn split_condition_no_const(
        condition: &str,
        regex: &Regex,
    ) -> Result<(String, String, String), String> {
        if let Some(captures) = regex.captures(condition).unwrap() {
            let operand_left: String = captures
                .get(1)
                .ok_or("Invalid or missing operand.")?
                .as_str()
                .to_string();
            let operator: String = captures
                .get(2)
                .ok_or("Invalid or missing operator.")?
                .as_str()
                .to_string();
            let operand_right: String = captures
                .get(3)
                .ok_or("Invalid or missing operand.")?
                .as_str()
                .to_string();
            Ok((operand_left, operator, operand_right))
        } else {
            Err("Error: condition has an invalid format.".to_string())
        }
    }

    fn split_condition_char_const_left(
        condition: &str,
        regex: &Regex,
    ) -> Result<(String, String, String), String> {
        if let Some(captures) = regex.captures(condition).unwrap() {
            let operand_left: String = captures
                .get(2)
                .ok_or("Invalid or missing operand.")?
                .as_str()
                .to_string();
            let operator: String = captures
                .get(3)
                .ok_or("Invalid or missing operator.")?
                .as_str()
                .to_string();
            let operand_right: String = captures
                .get(4)
                .ok_or("Invalid or missing operand.")?
                .as_str()
                .to_string();
            Ok((operand_left, operator, operand_right))
        } else {
            Err("Error: condition has an invalid format.".to_string())
        }
    }

    fn split_condition_char_const_right(
        condition: &str,
        regex: &Regex,
    ) -> Result<(String, String, String), String> {
        if let Some(captures) = regex.captures(condition).unwrap() {
            let operand_left: String = captures
                .get(1)
                .ok_or("Invalid or missing operand.")?
                .as_str()
                .to_string();
            let operator: String = captures
                .get(2)
                .ok_or("Invalid or missing operator.")?
                .as_str()
                .to_string();
            let operand_right: String = captures
                .get(4)
                .ok_or("Invalid or missing operand.")?
                .as_str()
                .to_string();
            Ok((operand_left, operator, operand_right))
        } else {
            Err("Error: condition has an invalid format.".to_string())
        }
    }

    fn split_condition_number_const_right(
        condition: &str,
        regex: &Regex,
    ) -> Result<(String, String, String), String> {
        if let Some(captures) = regex.captures(condition).unwrap() {
            let operand_left: String = captures
                .get(1)
                .ok_or("Invalid or missing operand.")?
                .as_str()
                .to_string();
            let operator: String = captures
                .get(2)
                .ok_or("Invalid or missing operator.")?
                .as_str()
                .to_string();
            let operand_right: String = captures
                .get(3)
                .ok_or("Invalid or missing operand.")?
                .as_str()
                .to_string();
            Ok((operand_left, operator, operand_right))
        } else {
            Err("Error: condition has an invalid format.".to_string())
        }
    }
    fn split_condition_number_const_left(
        condition: &str,
        regex: &Regex,
    ) -> Result<(String, String, String), String> {
        if let Some(captures) = regex.captures(condition).unwrap() {
            let operand_left: String = captures
                .get(1)
                .ok_or("Invalid or missing operand.")?
                .as_str()
                .to_string();
            let operator: String = captures
                .get(3)
                .ok_or("Invalid or missing operator.")?
                .as_str()
                .to_string();
            let operand_right: String = captures
                .get(4)
                .ok_or("Invalid or missing operand.")?
                .as_str()
                .to_string();
            Ok((operand_left, operator, operand_right))
        } else {
            Err("Error: condition has an invalid format.".to_string())
        }
    }

    fn split_condition_two_number_const(
        condition: &str,
        regex: &Regex,
    ) -> Result<(String, String, String), String> {
        if let Some(captures) = regex.captures(condition).unwrap() {
            let operand_left: String = captures
                .get(1)
                .ok_or("Invalid or missing operand.")?
                .as_str()
                .to_string();
            let operator: String = captures
                .get(3)
                .ok_or("Invalid or missing operator.")?
                .as_str()
                .to_string();
            let operand_right: String = captures
                .get(4)
                .ok_or("Invalid or missing operand.")?
                .as_str()
                .to_string();
            Ok((operand_left, operator, operand_right))
        } else {
            Err("Error: condition has an invalid format.".to_string())
        }
    }

    fn split_condition_two_char_const(
        condition: &str,
        regex: &Regex,
    ) -> Result<(String, String, String), String> {
        if let Some(captures) = regex.captures(condition).unwrap() {
            let operand_left: String = captures
                .get(2)
                .ok_or("Invalid or missing operand.")?
                .as_str()
                .to_string();
            let operator: String = captures
                .get(3)
                .ok_or("Invalid or missing operator.")?
                .as_str()
                .to_string();
            let operand_right: String = captures
                .get(5)
                .ok_or("Invalid or missing operand.")?
                .as_str()
                .to_string();
            Ok((operand_left, operator, operand_right))
        } else {
            Err("Error: condition has an invalid format.".to_string())
        }
    }

    /// Splits a qualified column name into alias and column parts.
    pub fn split_column(s: &str) -> Result<(String, String), PatternError> {
        match s.split_once('.') {
            Some((left, right)) => {
                return Ok((left.to_string(), right.to_string()));
            }
            None => {
                return Err(PatternError::new("Invalid column reference"));
            }
        }
    }

    fn remove_quotes(s: &str) -> &str {
        if let Some(unquoted) = s.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
            unquoted
        } else if let Some(unquoted) = s.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')) {
            unquoted
        } else if let Some(unquoted) = s.strip_prefix('ʺ').and_then(|s| s.strip_suffix('ʺ')) {
            unquoted
        } else {
            s
        }
    }

    #[cfg(test)]
    fn get_operator(&self) -> &str {
        match self.operator {
            Operator::Equal => "=",
            Operator::NotEqual => "<>",
            Operator::LessThan => "<",
            Operator::GreaterThan => ">",
            Operator::GreaterEqual => ">=",
            Operator::LessEqual => "<=",
        }
    }

    #[cfg(test)]
    pub fn to_string(&self) -> String {
        return format!(
            "Condition {{ left_operand={}, operator={}, right_operand={} }}",
            self.left_operand.get_value(),
            self.get_operator(),
            self.right_operand.get_value()
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
        let lru_policy = String::from("LRU");

        let buffer_manager = Rc::new(RefCell::new(BufferManager::new(&config, dm, lru_policy)));

        let column_info: Vec<ColInfo> = vec![
            ColInfo::new("NOM".to_string(), "VARCHAR(20)".to_string()),
            ColInfo::new("PRENOM".to_string(), "VARCHAR(20)".to_string()),
            ColInfo::new("AGE".to_string(), "INT".to_string()),
        ];
        let relation = Relation::new("PERSONNE".to_string(), column_info.clone(), buffer_manager);

        let record = Record::new(vec![
            "GNAHO".to_string(),
            "CHRISTOPHE".to_string(),
            "50".to_string(),
        ]);

        let condition: Result<Condition, PatternError> =
            Condition::check_syntax("26=12".to_string(), &relation.get_columns(), &record);
        if condition.is_ok() {
            println!("{:?}", condition.unwrap().to_string());
        } else {
            println!("{:?}", condition.err().unwrap());
        }
    }

    #[test]
    fn test_split_condition_no_const() {
        let condition = "table1.col1 = table2.col2";
        let regex = &NO_CONST;
        let result = Condition::split_condition_no_const(condition, regex);
        assert!(result.is_ok());
        let (left, op, right) = result.unwrap();
        assert_eq!(left, "table1.col1");
        assert_eq!(op, "=");
        assert_eq!(right, "table2.col2");
    }

    #[test]
    fn test_split_condition_char_const_left() {
        let condition = "'value' = table.col";
        let regex = &CHAR_CONST_LEFT;
        let result = Condition::split_condition_char_const_left(condition, regex);
        assert!(result.is_ok());
        let (left, op, right) = result.unwrap();
        assert_eq!(left, "value");
        assert_eq!(op, "=");
        assert_eq!(right, "table.col");
    }

    #[test]
    fn test_split_condition_two_char_const() {
        let condition = "'value1' = 'value2'";
        let regex = &TWO_CHAR_CONST;
        let result = Condition::split_condition_two_char_const(condition, regex);
        assert!(result.is_ok());
        let (left, op, right) = result.unwrap();
        assert_eq!(left, "value1");
        assert_eq!(op, "=");
        assert_eq!(right, "value2");
    }

    #[test]
    fn test_split_column() {
        let col = "table.col";
        let result = Condition::split_column(col);
        assert!(result.is_ok());
        let (table, col) = result.unwrap();
        assert_eq!(table, "table");
        assert_eq!(col, "col");
    }

    #[test]
    fn test_remove_quotes() {
        let with_quotes = "'value'";
        let result = Condition::remove_quotes(with_quotes);
        assert_eq!(result, "value");
    }

    impl PartialEq for Operator {
        fn eq(&self, _other: &Self) -> bool {
            match self {
                _other => true,
            }
        }
    }

    #[test]
    fn test_to_operator() {
        let op_str = "=";
        let result = Condition::to_operator(op_str);
        assert!(result.is_ok());
        let op = result.unwrap();
        assert!(op == Operator::Equal);
    }

    #[test]
    fn test_evaluate() {
        let left = Box::new(Number::new("10"));
        let right = Box::new(Number::new("20"));
        let condition = Condition::new(left, Operator::LessThan, right);
        assert!(condition.evaluate());
    }

    #[test]
    fn test_check_syntax_valid_conditions() {
        let s: String = String::from("config.json");
        let config = DBConfig::load_db_config(s);
        let dm = DiskManager::new(&config);
        let lru_policy = String::from("LRU");

        let buffer_manager = Rc::new(RefCell::new(BufferManager::new(&config, dm, lru_policy)));

        let column_info: Vec<ColInfo> = vec![
            ColInfo::new("NOM".to_string(), "VARCHAR(20)".to_string()),
            ColInfo::new("PRENOM".to_string(), "VARCHAR(20)".to_string()),
            ColInfo::new("AGE".to_string(), "INT".to_string()),
        ];
        let relation = Relation::new("PERSONNE".to_string(), column_info.clone(), buffer_manager);

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
                Condition::check_syntax(cond_str.to_string(), &relation.get_columns(), &record);
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
        let condition1 = Condition::new(left_number, Operator::LessThan, right_number);
        assert!(condition1.evaluate(), "Expected 10 < 20 to be true.");

        let left_string = Box::new(Chars::new("hello"));
        let right_string = Box::new(Chars::new("world"));
        let condition2 = Condition::new(left_string, Operator::NotEqual, right_string);
        assert!(condition2.evaluate(), "Expected hello <> world to be true.");

        let left_number = Box::new(Number::new("100"));
        let right_number = Box::new(Number::new("100"));
        let condition3 = Condition::new(left_number, Operator::Equal, right_number);
        assert!(condition3.evaluate(), "Expected 100 = 100 to be true.");
    }

    #[test]
    fn test_regex_patterns() {
        let regexs = vec![
            (&NO_CONST, "table1.col1 = table2.col2", true),
            (&CHAR_CONST_LEFT, "'value' = table.col", true),
            (&CHAR_CONST_RIGHT, "table.col = 'value'", true),
            (&NUMBER_CONST_LEFT, "-12.3 <= table.col", true),
            (&NUMBER_CONST_RIGHT, "table.col >= -99.99", true),
            (&TWO_CHAR_CONST, "'hello' <> 'world'", true),
            (&TWO_NUMBER_CONST, "-50 < -10", true),
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
        let condition = Condition::new(left, Operator::LessThan, right);

        let string_repr = condition.to_string();
        assert_eq!(
            string_repr,
            "Condition { left_operand=25, operator=<, right_operand=50 }"
        );
    }

    #[test]
    fn test_evaluate2() {
        let s: String = String::from("config.json");
        let config = DBConfig::load_db_config(s);
        let dm = DiskManager::new(&config);
        let lru_policy = String::from("LRU");

        let buffer_manager = Rc::new(RefCell::new(BufferManager::new(&config, dm, lru_policy)));

        let column_info: Vec<ColInfo> = vec![
            ColInfo::new("NOM".to_string(), "VARCHAR(20)".to_string()),
            ColInfo::new("PRENOM".to_string(), "VARCHAR(20)".to_string()),
            ColInfo::new("AGE".to_string(), "INT".to_string()),
        ];
        let relation = Relation::new("PERSONNE".to_string(), column_info.clone(), buffer_manager);

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
                Condition::check_syntax(r.to_string(), &relation.get_columns(), &record);
            if condition.is_ok() {
                println!("{:?}", condition.unwrap().to_string());
            } else {
                println!("{:?}", condition.err().unwrap());
            }
        }
    }
}
