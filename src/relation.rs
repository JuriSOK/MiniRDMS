//! Relation storage: encodes records, manages data pages, and scans table contents.

use crate::buffer::Buffer;
use crate::buffer_manager::BufferManager;
use crate::col_info::ColInfo;
use crate::page::PageId;
use crate::record::Record;
use crate::record_id::RecordId;
use bytebuffer::ByteBuffer;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Relation<'a> {
    name: String,
    columns: Vec<ColInfo>,
    nb_columns: usize,

    buffer_manager: Rc<RefCell<BufferManager<'a>>>,
    header_page_id: PageId,
}

impl<'a> Relation<'a> {
    /// Creates a new relation and initializes its header page.
    pub fn new(name: String, columns: Vec<ColInfo>, bfm: Rc<RefCell<BufferManager<'a>>>) -> Self {
        let tmp = columns.len();

        let header_page_id = bfm.borrow_mut().get_disk_manager_mut().alloc_page();

        // The header page starts with the number of data pages stored at offset 0.
        {
            let mut bfmr = bfm.borrow_mut();
            let _ = bfmr.get_page(&header_page_id).write_int(0, 0);
            bfmr.free_page(&header_page_id, true);
            bfmr.flush_buffers();
        }

        Relation {
            name: String::from(name),
            columns,
            nb_columns: tmp,
            buffer_manager: bfm,
            header_page_id,
        }
    }

    /// Rebuilds a relation from catalog metadata without allocating new pages.
    pub fn from_saved(
        name: String,
        columns: Vec<ColInfo>,
        header_page_id: PageId,
        bfm: Rc<RefCell<BufferManager<'a>>>,
    ) -> Self {
        Relation {
            name,
            columns: columns.clone(),
            nb_columns: columns.clone().len(),
            buffer_manager: bfm,
            header_page_id,
        }
    }

    /// Returns the table name.
    pub fn get_name(&self) -> &String {
        &self.name
    }

    /// Returns a copy of the relation schema.
    pub fn get_columns(&self) -> Vec<ColInfo> {
        self.columns.clone()
    }

    /// Returns the header page used to find all data pages.
    pub fn get_header_page_id(&self) -> &PageId {
        return &self.header_page_id;
    }

    /// Serializes a record into a page buffer and returns the number of bytes written.
    pub fn write_record_to_buffer(&self, record: Record, buffer: &mut Buffer, pos: usize) -> usize {
        let tuple = record.get_tuple().clone();

        let mut counter: usize = 0;

        let mut index: usize = pos;

        let mut varchar_found: bool = false;

        for i in 0..self.columns.len() {
            if self.columns[i].get_column_type().starts_with("VARCHAR") {
                varchar_found = true;
                break;
            }
        }

        let mut field_sizes: Vec<usize> = Vec::new();

        if varchar_found {
            // Variable-length records start with an offset table, followed by field data.
            for i in 0..tuple.len() {
                match self.columns[i].get_column_type().as_str() {
                    "INT" => {
                        field_sizes.push(4);
                        counter += 4;
                        continue;
                    }
                    "REAL" => {
                        field_sizes.push(4);
                        counter += 4;
                        continue;
                    }
                    s if s.starts_with("CHAR") => {
                        let index: Option<usize> = s.find(')');

                        let substring: &str = &self.columns[i].get_column_type()[5..index.unwrap()];

                        let nbytes = " "
                            .repeat(substring.parse::<usize>().unwrap())
                            .as_bytes()
                            .len();
                        field_sizes.push(nbytes);
                        counter += 4;
                        continue;
                    }
                    s2 if s2.starts_with("VARCHAR") => {
                        let index: Option<usize> = s2.find(')');
                        let substring: &str = &self.columns[i].get_column_type()[8..index.unwrap()];

                        let len_s = substring.parse::<usize>().unwrap();
                        let nbytes = if len_s >= tuple[i].len() {
                            tuple[i].as_bytes().len()
                        } else {
                            " ".repeat(len_s).as_bytes().len()
                        };
                        field_sizes.push(nbytes);
                        counter += 4;
                        continue;
                    }
                    _ => {}
                }
            }
            counter += 4;

            let mut counter2 = counter;
            for i in 0..field_sizes.len() {
                match self.columns[i].get_column_type().as_str() {
                    "INT" => {
                        buffer.write_int(index, (counter2 + pos) as i32).unwrap();
                        counter2 += 4;
                        index += 4;
                        continue;
                    }
                    "REAL" => {
                        buffer.write_int(index, (counter2 + pos) as i32).unwrap();
                        counter2 += 4;
                        index += 4;
                        continue;
                    }
                    s if s.starts_with("CHAR") => {
                        let size = field_sizes[i];

                        buffer.write_int(index, (counter2 + pos) as i32).unwrap();

                        counter2 += size;
                        index += 4;
                        continue;
                    }
                    s2 if s2.starts_with("VARCHAR") => {
                        let size = field_sizes[i];

                        buffer.write_int(index, (counter2 + pos) as i32).unwrap();

                        counter2 += size;
                        index += 4;
                        continue;
                    }
                    _ => {}
                }
            }

            buffer.write_int(index, (counter2 + pos) as i32).unwrap();
            index = pos + counter;

            for i in 0..field_sizes.len() {
                match self.columns[i].get_column_type().as_str() {
                    "INT" => {
                        let value: i32 = tuple[i].parse().unwrap();
                        let _ = buffer.write_int(index, value);

                        counter += 4;
                        index += 4;
                        continue;
                    }
                    "REAL" => {
                        let value: f32 = tuple[i].parse().unwrap();
                        let _ = buffer.write_float(index, value);
                        counter += 4;
                        index += 4;
                        continue;
                    }
                    s if s.starts_with("CHAR") => {
                        let bytes = tuple[i].as_bytes();
                        let _ = buffer.write_string(index, tuple[i].as_str(), bytes.len());
                        counter += bytes.len();
                        index += bytes.len();
                        continue;
                    }
                    s2 if s2.starts_with("VARCHAR") => {
                        let nbytes = tuple[i].as_bytes().len();
                        let bytes = tuple[i].as_bytes();

                        let _ = buffer.write_string(index, tuple[i].as_str(), bytes.len());
                        counter += nbytes;
                        index += nbytes;
                        continue;
                    }
                    _ => {}
                }
            }
        } else {
            for i in 0..self.nb_columns {
                match self.columns[i].get_column_type().as_str() {
                    "INT" => {
                        let value: i32 = tuple[i].parse().unwrap();
                        let _ = buffer.write_int(index, value);

                        counter += 4;
                        index += 4;
                        continue;
                    }
                    "REAL" => {
                        let value: f32 = tuple[i].parse().unwrap();
                        let _ = buffer.write_float(index, value);

                        counter += 4;
                        index += 4;
                        continue;
                    }
                    s if s.starts_with("CHAR") => {
                        let bytes = tuple[i].as_bytes();
                        let _ = buffer.write_string(index, tuple[i].as_str(), bytes.len());
                        counter += bytes.len();
                        index += bytes.len();

                        continue;
                    }

                    _ => {}
                }
            }
        }
        return counter;
    }

    /// Deserializes one record from a page buffer and returns the number of bytes read.
    pub fn read_from_buffer(&self, record: &mut Record, buff: &Buffer, pos: usize) -> usize {
        let mut tuple: Vec<String> = Vec::new();
        let mut varchar = false;
        let mut bytes_read = 0;
        let mut pos_local = pos;

        for i in 0..self.nb_columns {
            if self.columns[i]
                .get_column_type()
                .as_str()
                .starts_with("VARCHAR")
            {
                varchar = true;
                break;
            }
        }

        if varchar {
            // Read the offset table first, then resolve each field value from its byte range.
            for i in 0..self.nb_columns {
                let value_start: usize = buff.read_int(pos_local).unwrap().try_into().unwrap();

                let value_end: usize = buff.read_int(pos_local + 4).unwrap().try_into().unwrap();

                bytes_read += 4;

                if self.columns[i].get_column_type().eq("INT") {
                    let value = buff.read_int(value_start).unwrap();
                    tuple.push(value.to_string());
                    bytes_read += 4;
                } else if self.columns[i].get_column_type().eq("REAL") {
                    let value = buff.read_float(value_start).unwrap();
                    tuple.push(value.to_string());
                    bytes_read += 4;
                } else {
                    let string_value = buff
                        .read_string(value_start, (value_end - value_start) as usize)
                        .unwrap();
                    tuple.push(string_value);
                    bytes_read += (value_end - value_start) as usize;
                }

                pos_local += 4;
            }
            bytes_read += 4;
        } else {
            let mut counter_pos = pos;
            for i in 0..self.nb_columns {
                match self.columns[i].get_column_type().as_str() {
                    "INT" => {
                        let value = buff.read_int(counter_pos).unwrap();
                        tuple.push(value.to_string());
                        counter_pos += 4;
                        bytes_read += 4;
                        continue;
                    }
                    "REAL" => {
                        let value = buff.read_float(counter_pos).unwrap();
                        tuple.push(value.to_string());
                        counter_pos += 4;
                        bytes_read += 4;
                        continue;
                    }
                    s if s.starts_with("CHAR") => {
                        let open_paren_index = s.find("(");
                        let close_paren_index = s.find(")");
                        let size_char = s
                            [(open_paren_index.unwrap() + 1)..close_paren_index.unwrap()]
                            .parse::<i32>()
                            .unwrap();

                        let string_value =
                            buff.read_string(counter_pos, size_char as usize).unwrap();

                        tuple.push(string_value);
                        counter_pos += size_char as usize;
                        bytes_read += size_char as usize;
                        continue;
                    }

                    _ => {}
                }
            }
        }
        record.set_tuple(tuple);
        return bytes_read as usize;
    }

    /// Allocates a new data page and records it in the relation header page.
    pub fn add_data_page(&mut self) -> () {
        let mut buffer_manager = self.buffer_manager.borrow_mut();
        let remaining_bytes = buffer_manager
            .get_disk_manager()
            .get_db_config()
            .get_page_size() as u32;

        let new_page = buffer_manager.get_disk_manager_mut().alloc_page();

        let mut header_page = buffer_manager.get_page(&self.header_page_id);

        let mut nb_pages = header_page.read_int(0).unwrap();
        nb_pages += 1;
        let _ = header_page.write_int(0, nb_pages);

        // Each header entry stores file id, page id, and remaining free bytes.
        let next_offset = 4 + (nb_pages - 1) * 12;

        let _ = header_page.write_int(next_offset as usize, new_page.get_file_idx() as i32);
        let _ = header_page.write_int((next_offset + 4) as usize, new_page.get_page_idx() as i32);

        let _ = header_page.write_int((next_offset + 8) as usize, (remaining_bytes - 8) as i32);

        buffer_manager.free_page(&self.header_page_id, true);

        let mut data_page = buffer_manager.get_page(&new_page);

        let _ = data_page.write_int((remaining_bytes - 4) as usize, 0);
        let _ = data_page.write_int((remaining_bytes - 8) as usize, 0);
        buffer_manager.free_page(&new_page, true);

        buffer_manager.flush_buffers();
    }

    /// Finds the first data page with enough free space for a record.
    pub fn get_free_data_page_id(&self, size_record: usize) -> Option<PageId> {
        let mut buffer_manager = self.buffer_manager.borrow_mut();

        let total = buffer_manager
            .get_page(&self.header_page_id)
            .read_int(0)
            .unwrap();
        buffer_manager.free_page(&self.header_page_id, false);

        for i in 0..total {
            let offset = 4 + i * 12;

            let test = buffer_manager
                .get_page(&self.header_page_id)
                .read_int((offset + 8) as usize)
                .unwrap();
            buffer_manager.free_page(&self.header_page_id, false);

            if size_record + 8 <= test as usize {
                let page = Some(PageId::new(
                    buffer_manager
                        .get_page(&self.header_page_id)
                        .read_int(offset as usize)
                        .unwrap() as u32,
                    buffer_manager
                        .get_page(&self.header_page_id)
                        .read_int((offset + 4) as usize)
                        .unwrap() as u32,
                ));

                buffer_manager.free_page(&self.header_page_id, false);
                buffer_manager.free_page(&self.header_page_id, false);

                return page;
            }
        }

        return None;
    }

    /// Writes a record into a specific data page and returns its record identifier.
    pub fn write_record_to_data_page(&mut self, record: Record, page_id: PageId) -> RecordId {
        let mut buffer_manager: std::cell::RefMut<'_, BufferManager<'a>> =
            self.buffer_manager.borrow_mut();

        let page_size = buffer_manager
            .get_disk_manager()
            .get_db_config()
            .get_page_size();

        let mut page = buffer_manager.get_page(&page_id);

        let free_position = page.read_int((page_size - 4) as usize).unwrap() as usize;

        let size_record: usize = self.write_record_to_buffer(record, &mut page, free_position);

        let m_nb_slot: usize = page.read_int((page_size - 8) as usize).unwrap() as usize;

        let _ = page.write_int((page_size - 8) as usize, (m_nb_slot + 1) as i32);
        let _ = page.write_int(
            (page_size - 4) as usize,
            (free_position + size_record) as i32,
        );

        let size_pos: usize = m_nb_slot * 8;

        let _ = page.write_int(
            (page_size as usize) - 8 - size_pos - 8,
            free_position as i32,
        );
        let _ = page.write_int((page_size as usize) - 8 - size_pos - 4, size_record as i32);

        let size_totale: usize = size_record + 8;
        buffer_manager.free_page(&page_id, true);

        let mut header_page = buffer_manager.get_page(&self.header_page_id);
        for i in 0..header_page.read_int(0).unwrap() {
            let offset = 4 + i * 12;

            if header_page.read_int(offset as usize).unwrap() == (page_id.get_file_idx() as i32)
                && header_page.read_int((offset + 4) as usize).unwrap()
                    == (page_id.get_page_idx() as i32)
            {
                let tmp = header_page.read_int((offset + 8) as usize).unwrap();
                let _ = header_page.write_int((offset + 8) as usize, tmp - size_totale as i32);
                break;
            }
        }

        buffer_manager.free_page(&self.header_page_id, true);
        buffer_manager.flush_buffers();

        RecordId::new(page_id.clone(), (page_size as usize) - 8 - size_pos - 8)
    }

    /// Reads every record stored in one data page.
    pub fn get_records_in_data_page(&self, page_id: &PageId) -> Vec<Record> {
        let mut buffer_manager: std::cell::RefMut<'_, BufferManager<'a>> =
            self.buffer_manager.borrow_mut();

        let mut records = Vec::new();
        let page_size = buffer_manager
            .get_disk_manager()
            .get_db_config()
            .get_page_size() as usize;

        let buffer_data = buffer_manager.get_page(&page_id);
        let nb_record = buffer_data.read_int(page_size - 8).unwrap() as usize;

        let mut pos = 0;

        for _i in 0..nb_record {
            let vec: Vec<String> = Vec::new();

            let mut record = Record::new(vec);

            pos = pos + self.read_from_buffer(&mut record, &buffer_data, pos);

            records.push(record);
        }

        buffer_manager.free_page(&page_id, false);
        return records;
    }

    /// Reads all data page identifiers from the relation header page.
    pub fn get_data_pages(&self) -> Vec<PageId> {
        let mut page_infos = Vec::new();
        let mut buffer_manager = self.buffer_manager.borrow_mut();

        let buffer_header = buffer_manager.get_page(&self.header_page_id);
        let nb_pages = buffer_header.read_int(0).unwrap();

        for i in 0..nb_pages {
            let file_idx = buffer_header.read_int((4 + i * 12) as usize).unwrap();
            let page_idx = buffer_header.read_int((4 + i * 12 + 4) as usize).unwrap();

            page_infos.push(PageId::new(file_idx as u32, page_idx as u32));
        }

        buffer_manager.free_page(&self.header_page_id, false);
        return page_infos;
    }

    /// Inserts a record into an existing page or allocates a new page when needed.
    pub fn insert_record(&mut self, record: Record) -> RecordId {
        let page_size = self
            .buffer_manager
            .borrow_mut()
            .get_disk_manager()
            .get_db_config()
            .get_page_size();

        let mut byte_record = ByteBuffer::new();
        byte_record.resize(page_size as usize);

        let refcell_record = RefCell::new(byte_record);
        let mut buffer_record = Buffer::new(&Rc::new(refcell_record));

        let size_record = self.write_record_to_buffer(record.clone(), &mut buffer_record, 0);

        let data_page = self.get_free_data_page_id(size_record);

        if data_page.is_none() {
            self.add_data_page();

            let data_page = (self.get_free_data_page_id(size_record)).unwrap();
            return self.write_record_to_data_page(record, data_page);
        } else {
            return self.write_record_to_data_page(record, data_page.unwrap());
        }
    }

    /// Scans every data page and returns all records in this relation.
    pub fn get_all_records(&self) -> Vec<Record> {
        let mut records = Vec::new();
        let data_pages = self.get_data_pages();

        for page in data_pages.iter() {
            let mut page_records = self.get_records_in_data_page(page);
            records.append(&mut page_records);
        }

        return records;
    }
}

#[cfg(test)]
mod tests {

    use crate::disk_manager::DiskManager;
    use crate::DBConfig;

    use super::*;
    use std::rc::Rc;

    #[test]
    fn test_write_varchar() {
        let s: String = String::from("config.json");
        let config = DBConfig::load_db_config(s);
        let dm = DiskManager::new(&config);
        let lru_policy = String::from("LRU");

        let buffer_manager = Rc::new(RefCell::new(BufferManager::new(&config, dm, lru_policy)));

        let record = Record::new(vec![
            "SOK".to_string(),
            "ARNAUD".to_string(),
            "20".to_string(),
        ]);
        let column_info: Vec<ColInfo> = vec![
            ColInfo::new("NOM".to_string(), "CHAR(3)".to_string()),
            ColInfo::new("AGE".to_string(), "VARCHAR(6)".to_string()),
            ColInfo::new("PRENOM".to_string(), "INT".to_string()),
        ];
        let relation = Relation::new("PERSONNE".to_string(), column_info.clone(), buffer_manager);
        let pos = 0;

        let mut buffer = ByteBuffer::new();
        buffer.resize(32);
        let refcbuffer = RefCell::new(buffer);
        let mut buffer2 = Buffer::new(&Rc::new(refcbuffer));

        relation.write_record_to_buffer(record, &mut buffer2, pos);
        println!("{:?}", buffer2.get_mut_buffer().as_bytes());
    }

    #[test]

    fn test_read_from_buffer() {
        let s: String = String::from("config.json");
        let config = DBConfig::load_db_config(s);
        let dm = DiskManager::new(&config);
        let lru_policy = String::from("LRU");

        let buffer_manager = Rc::new(RefCell::new(BufferManager::new(&config, dm, lru_policy)));

        let record = Record::new(vec![
            "SOK".to_string(),
            "20".to_string(),
            "ARNAUD".to_string(),
        ]);
        let column_info: Vec<ColInfo> = vec![
            ColInfo::new("NOM".to_string(), "CHAR(3)".to_string()),
            ColInfo::new("AGE".to_string(), "INT".to_string()),
            ColInfo::new("PRENOM".to_string(), "VARCHAR(6)".to_string()),
        ];
        let relation = Relation::new("PERSONNE".to_string(), column_info.clone(), buffer_manager);
        let pos = 0;

        let mut buffer = ByteBuffer::new();
        buffer.resize(32);
        let refcbuffer = RefCell::new(buffer);
        let mut buffer2 = Buffer::new(&Rc::new(refcbuffer));

        relation.write_record_to_buffer(record, &mut buffer2, pos);
        println!("{:?}", buffer2.get_mut_buffer());

        let string_tuple = vec!["".to_string(), "".to_string(), "".to_string()];

        let record_test: Record = Record::new(string_tuple);

        println!("record_test contents after reading from the buffer:");
        for field in record_test.get_tuple() {
            println!("{}", field);
        }
    }

    #[test]

    fn test_add_data_page() {
        let s: String = String::from("config.json");
        let config = DBConfig::load_db_config(s);
        let dm = DiskManager::new(&config);
        let lru_policy = String::from("LRU");

        let buffer_manager = Rc::new(RefCell::new(BufferManager::new(&config, dm, lru_policy)));

        let column_info: Vec<ColInfo> = vec![
            ColInfo::new("NOM".to_string(), "CHAR(3)".to_string()),
            ColInfo::new("AGE".to_string(), "VARCHAR(6)".to_string()),
            ColInfo::new("PRENOM".to_string(), "INT".to_string()),
        ];
        let mut relation =
            Relation::new("PERSONNE".to_string(), column_info.clone(), buffer_manager);
        relation.add_data_page();
        relation.add_data_page();
        relation.add_data_page();
        relation.add_data_page();
    }

    #[test]
    fn test_get_free_data_page() {
        let s: String = String::from("config.json");
        let config = DBConfig::load_db_config(s);
        let dm = DiskManager::new(&config);
        let lru_policy = String::from("LRU");

        let buffer_manager = Rc::new(RefCell::new(BufferManager::new(&config, dm, lru_policy)));

        let column_info: Vec<ColInfo> = vec![
            ColInfo::new("NOM".to_string(), "CHAR(3)".to_string()),
            ColInfo::new("AGE".to_string(), "VARCHAR(6)".to_string()),
            ColInfo::new("PRENOM".to_string(), "INT".to_string()),
        ];
        let mut relation =
            Relation::new("PERSONNE".to_string(), column_info.clone(), buffer_manager);
        relation.add_data_page();
        relation.add_data_page();

        let freepage = relation.get_free_data_page_id(10).unwrap();
        println!(
            "Page ID : {},{}",
            freepage.get_file_idx(),
            freepage.get_page_idx()
        );
    }

    #[test]

    fn test_write_record_to_data_page() {
        let s: String = String::from("config.json");
        let config = DBConfig::load_db_config(s);
        let dm = DiskManager::new(&config);
        let lru_policy = String::from("LRU");

        let buffer_manager = Rc::new(RefCell::new(BufferManager::new(&config, dm, lru_policy)));

        let column_info: Vec<ColInfo> = vec![
            ColInfo::new("NOM".to_string(), "VARCHAR(20)".to_string()),
            ColInfo::new("PRENOM".to_string(), "VARCHAR(20)".to_string()),
            ColInfo::new("AGE".to_string(), "INT".to_string()),
        ];
        let mut relation =
            Relation::new("PERSONNE".to_string(), column_info.clone(), buffer_manager);

        let record1 = Record::new(vec![
            "SOK".to_string(),
            "ARNAUD".to_string(),
            "20".to_string(),
        ]);
        let record2 = Record::new(vec![
            "MEUNIER".to_string(),
            "YOHANN".to_string(),
            "20".to_string(),
        ]);

        relation.add_data_page();
        let page_id = relation.get_data_pages()[0].clone();
        let rid1 = relation.write_record_to_data_page(record1, page_id.clone());
        let rid2 = relation.write_record_to_data_page(record2, page_id);

        println!(
            "RID tuple 1 : File idx {}, Page idx {}, Slot idx : {}",
            rid1.get_page_id().get_file_idx(),
            rid1.get_page_id().get_page_idx(),
            rid1.get_slot_idx()
        );

        println!(
            "RID tuple 2 : File idx {}, Page idx {}, Slot idx : {}",
            rid2.get_page_id().get_file_idx(),
            rid2.get_page_id().get_page_idx(),
            rid2.get_slot_idx()
        );
    }

    #[test]

    fn test_get_records_in_data_page() {
        let s: String = String::from("config.json");
        let config = DBConfig::load_db_config(s);
        let dm = DiskManager::new(&config);
        let lru_policy = String::from("LRU");

        let buffer_manager = Rc::new(RefCell::new(BufferManager::new(&config, dm, lru_policy)));

        let column_info: Vec<ColInfo> = vec![
            ColInfo::new("NOM".to_string(), "VARCHAR(20)".to_string()),
            ColInfo::new("PRENOM".to_string(), "VARCHAR(20)".to_string()),
            ColInfo::new("AGE".to_string(), "INT".to_string()),
        ];
        let mut relation =
            Relation::new("PERSONNE".to_string(), column_info.clone(), buffer_manager);

        let record1 = Record::new(vec![
            "SOK".to_string(),
            "ARNAUD".to_string(),
            "20".to_string(),
        ]);
        let record2 = Record::new(vec![
            "MEUNIER".to_string(),
            "YOHANN".to_string(),
            "20".to_string(),
        ]);
        let record3 = Record::new(vec![
            "MOUE".to_string(),
            "MAT".to_string(),
            "20".to_string(),
        ]);

        relation.add_data_page();
        let page_id = relation.get_data_pages()[0].clone();
        relation.write_record_to_data_page(record1, page_id.clone());
        relation.write_record_to_data_page(record2, page_id.clone());
        relation.write_record_to_data_page(record3, page_id.clone());

        let records = relation.get_records_in_data_page(&page_id);

        println!("{:?}", records);
    }

    #[test]

    fn test_get_data_pages() {
        let s: String = String::from("config.json");
        let config = DBConfig::load_db_config(s);
        let dm = DiskManager::new(&config);
        let lru_policy = String::from("LRU");

        let buffer_manager = Rc::new(RefCell::new(BufferManager::new(&config, dm, lru_policy)));

        let column_info: Vec<ColInfo> = vec![
            ColInfo::new("NOM".to_string(), "VARCHAR(20)".to_string()),
            ColInfo::new("PRENOM".to_string(), "VARCHAR(20)".to_string()),
            ColInfo::new("AGE".to_string(), "INT".to_string()),
        ];
        let mut relation =
            Relation::new("PERSONNE".to_string(), column_info.clone(), buffer_manager);

        relation.add_data_page();
        relation.add_data_page();
        relation.add_data_page();
        relation.add_data_page();
        relation.add_data_page();

        let pages = relation.get_data_pages();

        println!("{:?}", pages);
    }

    #[test]

    fn test_insert_record() {
        let s: String = String::from("config.json");
        let config = DBConfig::load_db_config(s);
        let dm = DiskManager::new(&config);
        let lru_policy = String::from("LRU");

        let buffer_manager = Rc::new(RefCell::new(BufferManager::new(&config, dm, lru_policy)));

        let column_info: Vec<ColInfo> = vec![
            ColInfo::new("NOM".to_string(), "VARCHAR(20)".to_string()),
            ColInfo::new("PRENOM".to_string(), "VARCHAR(20)".to_string()),
            ColInfo::new("AGE".to_string(), "INT".to_string()),
        ];
        let mut relation =
            Relation::new("PERSONNE".to_string(), column_info.clone(), buffer_manager);

        let record1 = Record::new(vec![
            "SOK".to_string(),
            "ARNAUD".to_string(),
            "20".to_string(),
        ]);
        let record2 = Record::new(vec![
            "MEUNIER".to_string(),
            "YOHANN".to_string(),
            "20".to_string(),
        ]);

        let rid1 = relation.insert_record(record1);
        let rid2 = relation.insert_record(record2);

        println!(
            "RID tuple 1 : File idx {}, Page idx {}, Slot idx : {}",
            rid1.get_page_id().get_file_idx(),
            rid1.get_page_id().get_page_idx(),
            rid1.get_slot_idx()
        );

        println!(
            "RID tuple 2 : File idx {}, Page idx {}, Slot idx : {}",
            rid2.get_page_id().get_file_idx(),
            rid2.get_page_id().get_page_idx(),
            rid2.get_slot_idx()
        );
    }

    #[test]
    fn test_get_all_records() {
        let s: String = String::from("config.json");
        let config = DBConfig::load_db_config(s);
        let dm = DiskManager::new(&config);
        let lru_policy = String::from("LRU");

        let buffer_manager = Rc::new(RefCell::new(BufferManager::new(&config, dm, lru_policy)));

        let column_info: Vec<ColInfo> = vec![
            ColInfo::new("NOM".to_string(), "VARCHAR(20)".to_string()),
            ColInfo::new("PRENOM".to_string(), "VARCHAR(20)".to_string()),
            ColInfo::new("AGE".to_string(), "INT".to_string()),
        ];
        let mut relation =
            Relation::new("PERSONNE".to_string(), column_info.clone(), buffer_manager);

        let record1 = Record::new(vec![
            "SOK".to_string(),
            "ARNAUD".to_string(),
            "20".to_string(),
        ]);
        let record2 = Record::new(vec![
            "MEUNIER".to_string(),
            "YOHANN".to_string(),
            "20".to_string(),
        ]);
        let record3 = Record::new(vec![
            "MOUST".to_string(),
            "MATH".to_string(),
            "20".to_string(),
        ]);
        let record4 = Record::new(vec![
            "LETACONNOUX".to_string(),
            "AYMERIC".to_string(),
            "20".to_string(),
        ]);
        let record5 = Record::new(vec![
            "CHIBANNI".to_string(),
            "RAMZY".to_string(),
            "20".to_string(),
        ]);
        let record6 = Record::new(vec![
            "BOTKZ".to_string(),
            "LEFOU".to_string(),
            "89".to_string(),
        ]);
        let record7 = Record::new(vec![
            "GNAHO".to_string(),
            "CHRISTOPHE".to_string(),
            "50".to_string(),
        ]);

        let rid1 = relation.insert_record(record1);
        let rid2 = relation.insert_record(record2);
        let rid3 = relation.insert_record(record3);
        let rid4 = relation.insert_record(record4);
        let _rid5 = relation.insert_record(record5);
        let _rid6 = relation.insert_record(record6);
        let _rid7 = relation.insert_record(record7);

        let list_record = relation.get_all_records();

        println!("Records : {:?}", list_record);

        println!(
            "RID tuple 1 : File idx {}, Page idx {}, Slot idx : {}",
            rid1.get_page_id().get_file_idx(),
            rid1.get_page_id().get_page_idx(),
            rid1.get_slot_idx()
        );

        println!(
            "RID tuple 2 : File idx {}, Page idx {}, Slot idx : {}",
            rid2.get_page_id().get_file_idx(),
            rid2.get_page_id().get_page_idx(),
            rid2.get_slot_idx()
        );

        println!(
            "RID tuple 3 : File idx {}, Page idx {}, Slot idx : {}",
            rid3.get_page_id().get_file_idx(),
            rid3.get_page_id().get_page_idx(),
            rid3.get_slot_idx()
        );

        println!(
            "RID tuple 4 : File idx {}, Page idx {}, Slot idx : {}",
            rid4.get_page_id().get_file_idx(),
            rid4.get_page_id().get_page_idx(),
            rid4.get_slot_idx()
        );
    }
}
