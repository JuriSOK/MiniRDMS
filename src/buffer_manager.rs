//! Buffer pool manager responsible for loading, pinning, replacing, and flushing pages.

use crate::buffer::Buffer;
use crate::{config::DBConfig, disk_manager::DiskManager, page::PageId, page_info::PageInfo};
use bytebuffer::ByteBuffer;
use std::cell::RefCell;
use std::cell::RefMut;
use std::rc::Rc;

pub struct BufferManager<'a> {
    db_config: &'a DBConfig,
    disk_manager: RefCell<DiskManager<'a>>,
    page_infos: Vec<PageInfo>,
    buffers: Vec<Rc<RefCell<ByteBuffer>>>,
    clock: u64,
    replacement_policy: String,
    loaded_page_count: u32,
}

impl<'a> BufferManager<'a> {
    /// Builds an in-memory buffer pool and attaches it to the disk manager.
    pub fn new(
        db_config: &'a DBConfig,
        disk_manager: DiskManager<'a>,
        replacement_policy: String,
    ) -> Self {
        let clock: u64 = 0;
        let mut buffers: Vec<Rc<RefCell<ByteBuffer>>> =
            Vec::<Rc<RefCell<ByteBuffer>>>::with_capacity(db_config.get_bm_buffer_count() as usize);

        for _i in 0..db_config.get_bm_buffer_count() as usize {
            let buffer = Rc::new(RefCell::new(ByteBuffer::new()));
            buffer
                .borrow_mut()
                .resize(db_config.get_page_size() as usize);
            buffers.push(buffer);
        }

        let page_infos: Vec<PageInfo> =
            Vec::<PageInfo>::with_capacity(db_config.get_bm_buffer_count() as usize);

        Self {
            db_config,
            disk_manager: RefCell::new(disk_manager),
            page_infos,
            buffers,
            clock,
            replacement_policy,
            loaded_page_count: 0,
        }
    }

    /// Returns mutable access to the disk manager for page allocation and deallocation.
    pub fn get_disk_manager_mut(&self) -> RefMut<DiskManager<'a>> {
        self.disk_manager.borrow_mut()
    }
    /// Returns mutable access to the disk manager for existing call sites.
    pub fn get_disk_manager(&self) -> RefMut<DiskManager<'a>> {
        self.disk_manager.borrow_mut()
    }

    /// Exposes buffer slots for tests that verify pool initialization.
    #[cfg(test)]
    pub fn get_buffers(&self) -> &Vec<Rc<RefCell<ByteBuffer>>> {
        return &self.buffers;
    }

    /// Exposes the replacement policy for tests.
    #[cfg(test)]
    pub fn get_replacement_policy(&self) -> String {
        return self.replacement_policy.clone();
    }

    /// Exposes the loaded page count for tests.
    #[cfg(test)]
    pub fn get_loaded_page_count(&self) -> u32 {
        return self.loaded_page_count;
    }

    /// Chooses the least recently used unpinned page for replacement.
    pub fn lru(&mut self) -> usize {
        let mut index: u32 = 0;
        let mut oldest_page: &PageInfo = &self.page_infos[0];
        let mut first_candidate_found: bool = false;

        for i in 0..self.page_infos.len() {
            if self.page_infos[i].get_pin_count() == 0 {
                if first_candidate_found {
                    if oldest_page.get_time() > self.page_infos[i].get_time() {
                        oldest_page = &self.page_infos[i];
                        index = i as u32;
                    }
                } else {
                    oldest_page = &self.page_infos[i];
                    first_candidate_found = true;
                }
            }
        }

        if oldest_page.get_pin_count() == 0 {
            return index as usize;
        } else {
            return self.db_config.get_bm_buffer_count() as usize;
        }
    }

    /// Chooses the most recently used unpinned page for replacement.
    pub fn mru(&mut self) -> usize {
        let mut index: u32 = 0;
        let mut newest_page: &PageInfo = &self.page_infos[0];
        let mut first_candidate_found: bool = false;

        for i in 0..self.page_infos.len() {
            if self.page_infos[i].get_pin_count() == 0 {
                if first_candidate_found {
                    if newest_page.get_time() < self.page_infos[i].get_time() {
                        newest_page = &self.page_infos[i];
                        index = i as u32;
                    }
                } else {
                    newest_page = &self.page_infos[i];
                    first_candidate_found = true;
                }
            }
        }

        if newest_page.get_pin_count() == 0 {
            return index as usize;
        } else {
            return self.db_config.get_bm_buffer_count() as usize;
        }
    }

    /// Pins and returns a page, loading it from disk or replacing another page if needed.
    pub fn get_page(&mut self, page_id: &PageId) -> Buffer {
        if self.loaded_page_count < self.db_config.get_bm_buffer_count() {
            for i in 0..self.page_infos.len() {
                if page_id.get_file_idx() == self.page_infos[i].get_page_id().get_file_idx()
                    && page_id.get_page_idx() == self.page_infos[i].get_page_id().get_page_idx()
                {
                    let new_pin_count = self.page_infos[i].get_pin_count() + 1;
                    self.page_infos[i].set_pin_count(new_pin_count);
                    self.page_infos[i].set_time(self.clock as i32);
                    self.clock += 1;
                    return Buffer::new(&self.buffers[i]);
                }
            }

            let page_info: PageInfo = PageInfo::new(page_id.clone(), 1, false, self.clock as i32);

            let index: u32 = self.loaded_page_count;
            self.page_infos.push(page_info);
            let _ = self
                .disk_manager
                .borrow()
                .read_page(&page_id, &mut self.buffers[index as usize].borrow_mut());
            self.loaded_page_count += 1;
            self.clock += 1;

            return Buffer::new(&self.buffers[index as usize]);
        } else {
            for i in 0..self.page_infos.len() {
                if page_id.get_file_idx() == self.page_infos[i].get_page_id().get_file_idx()
                    && page_id.get_page_idx() == self.page_infos[i].get_page_id().get_page_idx()
                {
                    let new_pin_count = self.page_infos[i].get_pin_count() + 1;
                    self.page_infos[i].set_pin_count(new_pin_count);
                    self.page_infos[i].set_time(self.clock as i32);
                    self.clock += 1;
                    return Buffer::new(&self.buffers[i]);
                }
            }

            let replacement_index: usize;

            if self.replacement_policy.eq("LRU") {
                replacement_index = self.lru();
            } else {
                replacement_index = self.mru();
            }
            if self.page_infos[replacement_index].get_pin_count() == 0 {
                if self.page_infos[replacement_index].get_dirty() == true {
                    let _ = self.disk_manager.borrow().write_page(
                        &self.page_infos[replacement_index].get_page_id(),
                        &mut self.buffers[replacement_index].borrow_mut(),
                    );
                }

                self.buffers[replacement_index].borrow_mut().clear();
                let _ = self
                    .disk_manager
                    .borrow()
                    .write_page(&page_id, &mut self.buffers[replacement_index].borrow_mut());

                let _ = self
                    .disk_manager
                    .borrow()
                    .read_page(&page_id, &mut self.buffers[replacement_index].borrow_mut());
                let page_info: PageInfo =
                    PageInfo::new(page_id.clone(), 1, false, self.clock as i32);
                self.page_infos[replacement_index] = page_info;
            }
            self.clock += 1;
            return Buffer::new(&self.buffers[replacement_index]);
        }
    }

    /// Unpins a page and optionally marks it as dirty so it will be flushed later.
    pub fn free_page(&mut self, page_id: &PageId, bit_dirty: bool) -> () {
        let mut page_info: &mut PageInfo = &mut PageInfo::new(page_id.clone(), 0, false, 0);
        let mut found: bool = false;
        for i in self.page_infos.iter_mut() {
            if page_id.get_file_idx() == i.get_page_id().get_file_idx()
                && page_id.get_page_idx() == i.get_page_id().get_page_idx()
            {
                page_info = i;
                found = true;
                break;
            }
        }
        if !found {
            return;
        }
        let index = page_info.get_pin_count() - 1;
        page_info.set_pin_count(index);
        page_info.set_dirty_bit(bit_dirty);
        if page_info.get_pin_count() == 0 {
            page_info.set_time(self.clock as i32);
        }
    }

    /// Writes every dirty page back to disk and resets the in-memory buffer slots.
    pub fn flush_buffers(&mut self) {
        for i in 0..self.loaded_page_count {
            if self.page_infos[i as usize].get_dirty() == true {
                let _ = self.disk_manager.borrow().write_page(
                    self.page_infos[i as usize].get_page_id(),
                    &mut self.buffers[i as usize].borrow_mut(),
                );
            }
            self.page_infos[i as usize].set_pin_count(0);
            self.page_infos[i as usize].set_dirty_bit(false);
            self.page_infos[i as usize].set_time(0);
        }
        self.loaded_page_count = 0;
        self.page_infos.clear();

        let mut buffers: Vec<Rc<RefCell<ByteBuffer>>> =
            Vec::<Rc<RefCell<ByteBuffer>>>::with_capacity(
                self.db_config.get_bm_buffer_count() as usize
            );

        for _i in 0..self.db_config.get_bm_buffer_count() as usize {
            let buffer = Rc::new(RefCell::new(ByteBuffer::new()));
            buffer
                .borrow_mut()
                .resize(self.db_config.get_page_size() as usize);
            buffers.push(buffer);
        }

        self.buffers = buffers;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;
    use std::io::Write;

    #[test]
    fn test_constructor_buffer() {
        let s: String = String::from("config.json");
        let config = DBConfig::load_db_config(s);
        let dm = DiskManager::new(&config);

        let lru_policy = String::from("LRU");

        let buffer_manager = BufferManager::new(&config, dm, lru_policy);
        assert_eq!(
            buffer_manager.get_buffers().len(),
            config.get_bm_buffer_count() as usize
        );
        assert_eq!(buffer_manager.get_loaded_page_count(), 0);
        assert_eq!(buffer_manager.get_replacement_policy(), "LRU");
    }

    #[test]
    fn test_flush_buffer() {
        let s: String = String::from("config.json");
        let config = DBConfig::load_db_config(s);
        let mut dm = DiskManager::new(&config);
        let lru_policy = String::from("LRU");

        let pagea = dm.alloc_page();
        let pageb = dm.alloc_page();
        let pagec = dm.alloc_page();
        let paged = dm.alloc_page();
        let pagee = dm.alloc_page();

        let mut buffer_manager = BufferManager::new(&config, dm, lru_policy);

        let mut buffer1 = Vec::new();
        let _ = buffer1.write_all("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".as_bytes());
        let _ = buffer1.write_all("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".as_bytes());

        let num1 = pagea.get_file_idx();
        let file_name1 = format!("res/dbpath/BinData/F{num1}.rsdb");
        println!("{}", file_name1);
        let mut file1 = OpenOptions::new()
            .write(true)
            .open(file_name1)
            .expect("test data file should open");
        let _ = file1.write_all(&buffer1);

        let mut buffer2 = Vec::new();
        let _ = buffer2.write_all("cccccccccccccccccccccccccccccccc".as_bytes());
        let _ = buffer2.write_all("dddddddddddddddddddddddddddddddd".as_bytes());
        let num2 = pagec.get_file_idx();
        let file_name2 = format!("res/dbpath/BinData/F{num2}.rsdb");
        println!("{}", file_name2);
        let mut file2 = OpenOptions::new()
            .write(true)
            .open(file_name2)
            .expect("test data file should open");
        let _ = file2.write_all(&buffer2);

        let mut buffer3 = Vec::new();
        let _ = buffer3.write_all("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".as_bytes());
        let num3 = pagee.get_file_idx();
        let file_name3 = format!("res/dbpath/BinData/F{num3}.rsdb");
        println!("{}", file_name3);
        let mut file3 = OpenOptions::new()
            .write(true)
            .open(file_name3)
            .expect("test data file should open");
        let _ = file3.write_all(&buffer3);

        let _page_a_buffer = buffer_manager.get_page(&pagea);
        let _page_b_buffer = buffer_manager.get_page(&pageb);
        let _page_c_buffer = buffer_manager.get_page(&pagec);
        let _page_d_buffer = buffer_manager.get_page(&paged);
        buffer_manager.free_page(&pagea, false);
        let _page_e_buffer = buffer_manager.get_page(&pagee);

        buffer_manager.flush_buffers();
        assert_eq!(buffer_manager.get_loaded_page_count(), 0);
    }

    #[test]

    fn test_get_page_and_free_page() {
        let s: String = String::from("config.json");
        let config = DBConfig::load_db_config(s);
        let mut dm = DiskManager::new(&config);
        let lru_policy = String::from("LRU");

        let pagea = dm.alloc_page();
        let pageb = dm.alloc_page();
        let pagec = dm.alloc_page();
        let paged = dm.alloc_page();
        let pagee = dm.alloc_page();

        let mut buffer_manager = BufferManager::new(&config, dm, lru_policy);

        let mut buffer1 = ByteBuffer::new();
        let _ = buffer1.write_all("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".as_bytes());
        let _ = buffer1.write_all("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".as_bytes());
        let data1 = buffer1.as_bytes();
        let num1 = pagea.get_file_idx();
        let file_name1 = format!("res/dbpath/BinData/F{num1}.rsdb");
        println!("{}", file_name1);
        let mut file1 = OpenOptions::new()
            .append(true)
            .write(true)
            .open(file_name1)
            .expect("test data file should open");
        let _ = file1.write_all(&data1);

        let mut buffer2 = ByteBuffer::new();
        let _ = buffer2.write_all("cccccccccccccccccccccccccccccccc".as_bytes());
        let _ = buffer2.write_all("dddddddddddddddddddddddddddddddd".as_bytes());

        let data2 = buffer2.as_bytes();
        let num2 = pagec.get_file_idx();
        let file_name2 = format!("res/dbpath/BinData/F{num2}.rsdb");
        println!("{}", file_name2);
        let mut file2 = OpenOptions::new()
            .append(true)
            .write(true)
            .open(file_name2)
            .expect("test data file should open");
        let _ = file2.write_all(&data2);

        let mut buffer3 = ByteBuffer::new();
        let _ = buffer3.write_all("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".as_bytes());

        let data3 = buffer3.as_bytes();
        let num3 = pagee.get_file_idx();
        let file_name3 = format!("res/dbpath/BinData/F{num3}.rsdb");
        println!("{}", file_name3);
        let mut file3 = OpenOptions::new()
            .write(true)
            .open(file_name3)
            .expect("test data file should open");
        let _ = file3.write_all(&data3);

        let _page_a_buffer = buffer_manager.get_page(&pagea);
        let _page_b_buffer = buffer_manager.get_page(&pageb);
        let _page_c_buffer = buffer_manager.get_page(&pagec);
        let _page_d_buffer = buffer_manager.get_page(&paged);
        buffer_manager.free_page(&pagea, false);
        let _page_e_buffer = buffer_manager.get_page(&pagee);

        let buffer3 = buffer_manager.buffers[3].borrow();
        let buffer1 = buffer_manager.buffers[1].borrow();
        let buffer2 = buffer_manager.buffers[2].borrow();

        let mut file_test = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("res/file_test_buffermanager")
            .expect("Failed to open the file");

        file_test
            .write_all(&buffer3.as_bytes())
            .expect("Failed to write data");
        file_test
            .write_all(&buffer1.as_bytes())
            .expect("Failed to write data");
        file_test
            .write_all(&buffer2.as_bytes())
            .expect("Failed to write data");
    }
}
