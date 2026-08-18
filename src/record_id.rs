//! Identifies a record by its data page and slot-table offset.

use crate::page::PageId;

pub struct RecordId {
    pub page_id: PageId,
    pub slot_idx: usize,
}
impl RecordId {
    /// Creates a new stable pointer to a record inside a data page.
    pub fn new(page_id: PageId, slot_idx: usize) -> Self {
        RecordId { page_id, slot_idx }
    }

    #[cfg(test)]
    /// Returns the data page that contains the record.
    pub fn get_page_id(&self) -> &PageId {
        return &self.page_id;
    }

    #[cfg(test)]
    /// Returns the slot-table offset of the record inside its data page.
    pub fn get_slot_idx(&self) -> &usize {
        return &self.slot_idx;
    }
}
