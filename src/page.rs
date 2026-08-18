use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub struct PageId {
    file_idx: u32,
    page_idx: u32,
}

impl PageId {
    pub fn new(fidx: u32, pidx: u32) -> Self {
        //Constructeur de la classe
        Self {
            //dans ce scope on met les attributs de la classe
            file_idx: fidx,
            page_idx: pidx,
        }
    }

    pub fn get_file_idx(&self) -> u32 {
        self.file_idx
    }
    pub fn get_page_idx(&self) -> u32 {
        self.page_idx
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_constructeur() {
        let f_test: u32 = 1;
        let p_test: u32 = 3;

        let classe = PageId::new(f_test, p_test);
        assert_eq!(classe.file_idx, 1);
        assert_eq!(classe.page_idx, 3);
    }
}
