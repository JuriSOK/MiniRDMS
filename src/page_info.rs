use crate::PageId;
pub struct PageInfo {
    page_id: PageId,
    pin_count: u32,
    dirty_bit: bool,
    time: i32,
}

impl PageInfo {
    pub fn new(page_id: PageId, pin_count: u32, dirty_bit: bool, time: i32) -> Self {
        Self {
            page_id,
            pin_count,
            dirty_bit,
            time,
        }
    }

    pub fn get_pin_count(&self) -> u32 {
        self.pin_count
    }
    pub fn get_dirty(&self) -> bool {
        self.dirty_bit
    }
    pub fn get_time(&self) -> i32 {
        self.time
    }
    pub fn get_page_id(&self) -> &PageId {
        &self.page_id
    }

    pub fn set_pin_count(&mut self, pin_count: u32) {
        self.pin_count = pin_count;
    }

    pub fn set_dirty_bit(&mut self, dirty_bit: bool) {
        self.dirty_bit = dirty_bit;
    }

    pub fn set_time(&mut self, time: i32) {
        self.time = time;
    }
}
