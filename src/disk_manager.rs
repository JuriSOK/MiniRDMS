//! Disk layer that allocates, frees, reads, writes, and persists physical pages.

use crate::config::DBConfig;
use crate::page::PageId;
use bincode;
use bytebuffer::ByteBuffer;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::{Read, Seek, SeekFrom, Write};

pub struct DiskManager<'a> {
    config: &'a DBConfig,
    free_pages: Vec<PageId>,
}

impl<'a> DiskManager<'a> {
    /// Creates the disk manager and restores the saved free-page list.
    pub fn new(config: &'a DBConfig) -> Self {
        let mut dm = Self {
            config,
            free_pages: Vec::new(),
        };

        if let Err(e) = dm.load_state() {
            eprintln!("Failed to load disk manager state: {}", e);
            panic!("Could not initialize the disk manager: {}", e);
        }

        dm
    }

    /// Allocates a reusable free page or appends a new page to a data file.
    pub fn alloc_page(&mut self) -> PageId {
        self.free_pages.clear();
        if let Err(e) = self.load_state() {
            eprintln!("Failed to load disk manager state: {}", e);
            panic!("Could not allocate a page: {}", e);
        }

        while let Some(page_id) = self.free_pages.pop() {
            if !self.page_exists(&page_id) {
                continue;
            }
            if let Err(e) = self.save_state() {
                eprintln!("Failed to save disk manager state: {}", e);
                panic!("Could not allocate a page: {}", e);
            }

            return page_id;
        }

        let mut file_idx = 0;

        loop {
            let file_path = format!("{}/BinData/F{}.rsdb", self.config.get_dbpath(), file_idx);

            if let Some(parent_dir) = std::path::Path::new(&file_path).parent() {
                fs::create_dir_all(parent_dir).expect("Could not create parent directories.");
            }

            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&file_path)
                .unwrap();

            let current_size = file.metadata().unwrap().len() as u32;
            let page_size = self.config.get_page_size();
            let max_file_size = self.config.get_dm_maxfilesize();

            if current_size < max_file_size {
                let new_page_id = PageId::new(file_idx, current_size / page_size);

                let forbidden_value = 0xFF;
                let mut write_buffer = Vec::<u8>::new();

                let byte_array = vec![forbidden_value; self.config.get_page_size() as usize];
                write_buffer.extend_from_slice(byte_array.as_ref());

                if let Err(e) = file.write_all(&write_buffer) {
                    eprintln!("Failed to write page data: {}", e);
                    panic!("Could not write the page: {}", e);
                }

                return new_page_id;
            }

            file_idx += 1;
        }
    }

    fn page_exists(&self, page_id: &PageId) -> bool {
        let file_path = format!(
            "{}/BinData/F{}.rsdb",
            self.config.get_dbpath(),
            page_id.get_file_idx()
        );
        let page_end = (page_id.get_page_idx() + 1) * self.config.get_page_size();

        std::fs::metadata(file_path)
            .map(|metadata| metadata.len() as u32 >= page_end)
            .unwrap_or(false)
    }

    /// Reads one physical page into the provided buffer.
    pub fn read_page(&self, page_id: &PageId, buff: &mut ByteBuffer) -> Result<(), std::io::Error> {
        let num_file = page_id.get_file_idx();
        let num_page = page_id.get_page_idx();

        let mut file: File = OpenOptions::new().read(true).open(format!(
            "{}/BinData/F{}.rsdb",
            self.config.get_dbpath(),
            num_file
        ))?;

        file.seek(SeekFrom::Start(
            (num_page * self.config.get_page_size()) as u64,
        ))?;

        let mut temp_buffer: Vec<u8> = vec![0; self.config.get_page_size() as usize];

        file.read_exact(&mut temp_buffer)?;
        buff.write_bytes(&temp_buffer);

        Ok(())
    }

    /// Writes the provided buffer bytes into one physical page.
    pub fn write_page(
        &self,
        page_id: &PageId,
        buff: &mut ByteBuffer,
    ) -> Result<(), Box<dyn Error>> {
        let num_file = page_id.get_file_idx();
        let num_page = page_id.get_page_idx();

        let mut file: File = OpenOptions::new().write(true).append(false).open(format!(
            "{}/BinData/F{}.rsdb",
            self.config.get_dbpath(),
            num_file
        ))?;

        file.seek(SeekFrom::Start(
            (num_page * self.config.get_page_size()) as u64,
        ))?;

        let data_to_write = buff.as_bytes();
        file.write_all(&data_to_write)?;

        Ok(())
    }

    /// Persists the free-page list to `dm.save`.
    pub fn save_state(&self) -> std::io::Result<()> {
        let dm_save_path = format!("{}/dm.save", self.config.get_dbpath());

        let _ = std::fs::remove_file(&dm_save_path);

        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(dm_save_path)?;

        let mut writer = BufWriter::new(file);

        // Each PageId is serialized separately so load_state can stream them back one by one.
        for page in &self.free_pages {
            let serialized_content = bincode::serialize(&page)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

            writer.write_all(&serialized_content)?;
        }

        Ok(())
    }

    /// Restores the free-page list from `dm.save`.
    pub fn load_state(&mut self) -> std::io::Result<()> {
        let dm_save_path = format!("{}/dm.save", self.config.get_dbpath());

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(dm_save_path)
            .expect("Could not open or create dm.save");

        let mut content = Vec::new();
        file.read_to_end(&mut content)
            .expect("Could not read dm.save");

        let mut pos = 0;

        while pos < content.len() {
            match bincode::deserialize::<PageId>(&content[pos..]) {
                Ok(instance) => {
                    self.free_pages.push(instance);

                    pos += bincode::serialized_size(&self.free_pages.last().unwrap()).unwrap()
                        as usize;
                }
                Err(_) => break,
            }
        }
        Ok(())
    }

    /// Marks a page as reusable and persists the updated free-page list.
    pub fn dealloc_page(&mut self, page_id: PageId) {
        if !self.free_pages.contains(&page_id) {
            self.free_pages.push(page_id);
            if let Err(e) = self.save_state() {
                eprintln!("Failed to save disk manager state: {}", e);
                panic!("Could not deallocate the page: {}", e);
            }
        }
    }

    /// Gives callers access to page-size and file-size settings.
    pub fn get_db_config(&self) -> &DBConfig {
        return &self.config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_constructor() {
        let s: String = String::from("config.json");
        let config = DBConfig::load_db_config(s);
        let dm = DiskManager::new(&config);
        assert_eq!(dm.config.get_dbpath(), "res/dbpath");
    }

    #[test]
    fn test_write_page_and_read_page_and_alloc_page() {
        let config = DBConfig::load_db_config("config.json".to_string());
        let mut dm = DiskManager::new(&config);
        let page_id = dm.alloc_page();

        let mut write_buffer = ByteBuffer::new();
        let byte_array = [7; 32];
        write_buffer.write_bytes(byte_array.as_ref());

        dm.write_page(&page_id, &mut write_buffer)
            .expect("write_page failed");

        let mut read_buff = ByteBuffer::new();
        dm.read_page(&page_id, &mut read_buff)
            .expect("read_page failed");

        let expected_data = [7; 32];
        let read_data = read_buff.as_bytes();

        assert_eq!(&read_data[..expected_data.len()], &expected_data[..]);
    }

    #[test]
    fn test_alloc_page() {
        let config = DBConfig::load_db_config("config.json".to_string());
        let mut dm = DiskManager::new(&config);
        let _page_id = dm.alloc_page();
    }

    #[test]
    fn test_dealloc_page() {
        let config = DBConfig::load_db_config("config.json".to_string());
        let mut dm = DiskManager::new(&config);
        let page_id = PageId::new(0, 0);
        dm.dealloc_page(page_id);
        let expected_page_id = PageId::new(0, 0);
        assert!(dm.free_pages.contains(&expected_page_id));
    }

    #[test]
    fn test_save_state() {
        let config = DBConfig::load_db_config("config.json".to_string());
        let mut dm = DiskManager::new(&config);

        let page_id = PageId::new(999, 0);
        dm.dealloc_page(page_id);
        let _ = dm.save_state();

        let dm2 = DiskManager::new(&config);
        let expected_page_id = PageId::new(999, 0);
        assert!(dm2.free_pages.contains(&expected_page_id));
    }

    #[test]
    fn test_load_state() {
        let config = DBConfig::load_db_config("config.json".to_string());
        let _dm = DiskManager::new(&config);
    }
}
