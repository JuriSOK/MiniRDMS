use std::any::Any;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct Number {
    pub valeur: f64,
}

#[derive(Debug, Clone)]
pub struct Chars {
    pub valeur: String,
}

pub trait Operande: Debug + 'static {
    fn compare(&self, other: Box<dyn Operande>) -> i8;
    fn get_type(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
    fn get_valeur(&self) -> String;

    fn clone_box(&self) -> Box<dyn Operande>;
}

impl Clone for Box<dyn Operande> {
    fn clone(&self) -> Box<dyn Operande> {
        self.clone_box()
    }
}

impl Number {
    pub fn new(s: &str) -> Self {
        Self {
            valeur: s.parse::<f64>().unwrap_or(0.0),
        }
    }
}

impl Operande for Number {
    fn compare(&self, operande: Box<dyn Operande>) -> i8 {
        if self.get_type() == "NUMBER" && operande.get_type() == "NUMBER" {
            if let Some(other_number) = operande.as_any().downcast_ref::<Number>() {
                return if self.valeur < other_number.valeur {
                    -1
                } else if self.valeur > other_number.valeur {
                    1
                } else {
                    0
                };
            }
        }
        -1 // donc un nombre valide > à un truc invalide donc à changer si pas d'accord
    }

    fn get_type(&self) -> &str {
        "NUMBER"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_valeur(&self) -> String {
        self.valeur.to_string()
    }

    fn clone_box(&self) -> Box<dyn Operande> {
        Box::new(self.clone())
    }
}

impl Chars {
    pub fn new(s: &str) -> Self {
        Self {
            valeur: s.to_string(),
        }
    }
}

impl Operande for Chars {
    fn compare(&self, operande: Box<dyn Operande>) -> i8 {
        if self.get_type() == "CHARS" && operande.get_type() == "CHARS" {
            if let Some(other_chars) = operande.as_any().downcast_ref::<Chars>() {
                return if self.valeur < other_chars.valeur {
                    -1
                } else if self.valeur > other_chars.valeur {
                    1
                } else {
                    0
                };
            }
        }
        -1 // donc un nombre valide > à un truc invalide donc à changer si pas d'accord
    }

    fn get_type(&self) -> &str {
        "CHARS"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_valeur(&self) -> String {
        self.valeur.to_string()
    }

    fn clone_box(&self) -> Box<dyn Operande> {
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

        assert_eq!(num1.compare(Box::new(num2.clone())), -1); // num1 < num2
        assert_eq!(num2.compare(Box::new(num1.clone())), 1); // num2 > num1
        assert_eq!(num1.compare(Box::new(num3.clone())), 0); // num1 == num3
    }

    #[test]
    fn test_mixed_compare() {
        let num = Number::new("10.5");
        let str = Chars::new("apple");

        assert_eq!(num.compare(Box::new(str.clone())), -1); // return -1 car type diff
        assert_eq!(str.compare(Box::new(num.clone())), -1); // return -1 car type diff
    }

    #[test]
    fn test_invalid_number() {
        let invalid_num = Number::new("pas_nombre");
        let valid_num = Number::new("10.0");

        assert_eq!(invalid_num.compare(Box::new(valid_num.clone())), -1); // -1 car invalide
        assert_eq!(valid_num.compare(Box::new(invalid_num.clone())), 1); // nb valide > nb invalide
    }
}
