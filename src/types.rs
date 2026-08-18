use std::any::Any;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct Number {
    pub value: f64,
}

#[derive(Debug, Clone)]
pub struct Chars {
    pub value: String,
}

pub trait Operand: Debug + 'static {
    fn compare(&self, other: Box<dyn Operand>) -> i8;
    fn get_type(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
    fn get_value(&self) -> String;

    fn clone_box(&self) -> Box<dyn Operand>;
}

impl Clone for Box<dyn Operand> {
    fn clone(&self) -> Box<dyn Operand> {
        self.clone_box()
    }
}

impl Number {
    pub fn new(s: &str) -> Self {
        Self {
            value: s.parse::<f64>().unwrap_or(0.0),
        }
    }
}

impl Operand for Number {
    fn compare(&self, operand: Box<dyn Operand>) -> i8 {
        if self.get_type() == "NUMBER" && operand.get_type() == "NUMBER" {
            if let Some(other_number) = operand.as_any().downcast_ref::<Number>() {
                return if self.value < other_number.value {
                    -1
                } else if self.value > other_number.value {
                    1
                } else {
                    0
                };
            }
        }
        -1
    }

    fn get_type(&self) -> &str {
        "NUMBER"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_value(&self) -> String {
        self.value.to_string()
    }

    fn clone_box(&self) -> Box<dyn Operand> {
        Box::new(self.clone())
    }
}

impl Chars {
    pub fn new(s: &str) -> Self {
        Self {
            value: s.to_string(),
        }
    }
}

impl Operand for Chars {
    fn compare(&self, operand: Box<dyn Operand>) -> i8 {
        if self.get_type() == "CHARS" && operand.get_type() == "CHARS" {
            if let Some(other_chars) = operand.as_any().downcast_ref::<Chars>() {
                return if self.value < other_chars.value {
                    -1
                } else if self.value > other_chars.value {
                    1
                } else {
                    0
                };
            }
        }
        -1
    }

    fn get_type(&self) -> &str {
        "CHARS"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_value(&self) -> String {
        self.value.to_string()
    }

    fn clone_box(&self) -> Box<dyn Operand> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_compare() {
        let num1 = Number::new("10.5");
        let num2 = Number::new("15.2");
        let num3 = Number::new("10.5");

        assert_eq!(num1.compare(Box::new(num2.clone())), -1);
        assert_eq!(num2.compare(Box::new(num1.clone())), 1);
        assert_eq!(num1.compare(Box::new(num3.clone())), 0);
    }

    #[test]
    fn test_mixed_compare() {
        let num = Number::new("10.5");
        let str = Chars::new("apple");

        assert_eq!(num.compare(Box::new(str.clone())), -1);
        assert_eq!(str.compare(Box::new(num.clone())), -1);
    }

    #[test]
    fn test_invalid_number() {
        let invalid_num = Number::new("pas_nombre");
        let valid_num = Number::new("10.0");

        assert_eq!(invalid_num.compare(Box::new(valid_num.clone())), -1);
        assert_eq!(valid_num.compare(Box::new(invalid_num.clone())), 1);
    }
}
