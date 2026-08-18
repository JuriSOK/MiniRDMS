use crate::page::PageId;

pub struct RecordId {
    page_id: PageId,
    slot_idx: usize,
}
impl RecordId {
    pub fn new(page_id: PageId, slot_idx: usize) -> Self {
        RecordId { page_id, slot_idx }
    }

    pub fn get_page_id(&self) -> &PageId {
        return &self.page_id;
    }

    pub fn get_slot_idx(&self) -> &usize {
        return &self.slot_idx;
    }
}
