//! Runtime metadata for pages currently loaded in the buffer pool.

use crate::PageId;
pub struct PageInfo {
    page_id: PageId,
    pin_count: u32,
    dirty_bit: bool,
    time: i32,
}

impl PageInfo {
    /// Tracks one loaded page with its pin count, dirty flag, and replacement timestamp.
    pub fn new(page_id: PageId, pin_count: u32, dirty_bit: bool, time: i32) -> Self {
        Self {
            page_id,
            pin_count,
            dirty_bit,
            time,
        }
    }

    /// Returns how many callers currently pin this page in memory.
    pub fn get_pin_count(&self) -> u32 {
        self.pin_count
    }
    /// Returns whether this page must be written back before eviction.
    pub fn get_dirty(&self) -> bool {
        self.dirty_bit
    }
    /// Returns the timestamp used by LRU/MRU replacement policies.
    pub fn get_time(&self) -> i32 {
        self.time
    }
    /// Returns the physical page represented by this buffer slot.
    pub fn get_page_id(&self) -> &PageId {
        &self.page_id
    }

    /// Updates the pin count after `get_page` or `free_page`.
    pub fn set_pin_count(&mut self, pin_count: u32) {
        self.pin_count = pin_count;
    }

    /// Marks whether the page contents changed.
    pub fn set_dirty_bit(&mut self, dirty_bit: bool) {
        self.dirty_bit = dirty_bit;
    }

    /// Updates the replacement timestamp.
    pub fn set_time(&mut self, time: i32) {
        self.time = time;
    }
}
