use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub struct PageId {
    file_idx: u32,
    page_idx: u32,
}

impl PageId {
    pub fn new(fidx: u32, pidx: u32) -> Self {
        Self {
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
    fn test_constructor() {
        let f_test: u32 = 1;
        let p_test: u32 = 3;

        let config = PageId::new(f_test, p_test);
        assert_eq!(config.file_idx, 1);
        assert_eq!(config.page_idx, 3);
    }
}
