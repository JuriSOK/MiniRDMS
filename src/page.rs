//! Physical page identifiers used by the disk and buffer managers.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub struct PageId {
    file_idx: u32,
    page_idx: u32,
}

impl PageId {
    /// Creates the identifier for one page inside one `.rsdb` data file.
    pub fn new(fidx: u32, pidx: u32) -> Self {
        Self {
            file_idx: fidx,
            page_idx: pidx,
        }
    }

    /// Returns the data-file index.
    pub fn get_file_idx(&self) -> u32 {
        self.file_idx
    }
    /// Returns the page index within the data file.
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
