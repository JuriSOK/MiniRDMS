use crate::col_info::ColInfo;
use crate::condition::Condition;
use crate::record::Record;
use crate::select::Select;
use once_cell::sync::Lazy;
use std::rc::Rc;

pub trait IRecordIterator {
    fn get_next_record(&mut self) -> Option<Record>;
    fn close(&mut self);
    fn reset(&mut self);
}

pub struct RelationScanner {
    records: Vec<Record>,
    current_index: usize,
}

impl RelationScanner {
    pub fn new(records: Vec<Record>) -> Self {
        RelationScanner {
            records,
            current_index: 0,
        }
    }
}

impl IRecordIterator for RelationScanner {
    fn get_next_record(&mut self) -> Option<Record> {
        if self.current_index < self.records.len() {
            let record = self.records[self.current_index].clone();
            self.current_index += 1;
            Some(record)
        } else {
            None
        }
    }

    fn close(&mut self) {}

    fn reset(&mut self) {
        self.current_index = 0;
    }
}

pub struct SelectOperator {
    select: Select,
    child_iterator: Box<dyn IRecordIterator>,
    col_info: Rc<Vec<ColInfo>>,
}

impl IRecordIterator for SelectOperator {
    fn get_next_record(&mut self) -> Option<Record> {
        // Keep pulling from the child iterator until a record matches the WHERE clause.
        loop {
            if let Some(record) = self.child_iterator.get_next_record() {
                if self.evaluate_conditions(&record) {
                    return Some(record);
                }
            } else {
                return None;
            }
        }
    }

    fn close(&mut self) {
        self.child_iterator.close();
    }

    fn reset(&mut self) {
        self.child_iterator.reset();
    }
}

impl SelectOperator {
    pub fn new(
        select: Select,
        child_iterator: Box<dyn IRecordIterator>,
        col_info: Rc<Vec<ColInfo>>,
    ) -> Self {
        SelectOperator {
            select,
            child_iterator,
            col_info,
        }
    }

    fn evaluate_conditions(&self, record: &Record) -> bool {
        let conditions: &Result<Vec<Condition>, String> =
            &self.select.get_list_conditions(&self.col_info, record);

        if conditions.is_err() {
            return false;
        }
        let conditions = conditions.as_ref().unwrap();
        for condition in conditions {
            if !condition.evaluate() {
                return false;
            }
        }
        true
    }
}

pub struct ProjectionOperator {
    columns_to_project: Vec<String>,
    child_iterator: Box<dyn IRecordIterator>,
    col_info: Rc<Vec<ColInfo>>,
}

impl ProjectionOperator {
    pub fn new(
        columns_to_project: Vec<String>,
        child_iterator: Box<dyn IRecordIterator>,
        col_info: Rc<Vec<ColInfo>>,
    ) -> Self {
        ProjectionOperator {
            columns_to_project,
            child_iterator,
            col_info,
        }
    }

    fn project_columns(&self, record: &Record) -> Record {
        let mut projected_tuple = Vec::new();

        for col_name in &self.columns_to_project {
            if let Some(index) = self
                .col_info
                .iter()
                .position(|col| col.get_name() == col_name)
            {
                projected_tuple.push(record.get_value(index).clone());
            }
        }

        Record::new(projected_tuple)
    }
}

impl IRecordIterator for ProjectionOperator {
    fn get_next_record(&mut self) -> Option<Record> {
        if let Some(record) = self.child_iterator.get_next_record() {
            Some(self.project_columns(&record))
        } else {
            None
        }
    }

    fn close(&mut self) {
        self.child_iterator.close();
    }

    fn reset(&mut self) {
        self.child_iterator.reset();
    }
}

pub struct RecordPrinter<'a> {
    iterator: Box<dyn IRecordIterator + 'a>,
    total: usize,
}

pub static mut ERRORS: Lazy<Vec<String>> = Lazy::new(|| Vec::new());

impl<'a> RecordPrinter<'a> {
    pub fn new(iterator: Box<dyn IRecordIterator + 'a>) -> Self {
        RecordPrinter { iterator, total: 0 }
    }

    pub fn print_records(&mut self) {
        while let Some(record) = self.iterator.get_next_record() {
            self.print_record(&record);
            self.total += 1;
        }

        if (unsafe { ERRORS.len() } > 0) {
            println!("\nError(s): {}", unsafe { ERRORS.get(0).unwrap() });
            unsafe {
                ERRORS.clear();
            }
        } else {
            println!("\nTotal records =  {}", self.total);
        }
    }

    fn print_record(&self, record: &Record) {
        let tuple = record.get_tuple();

        println!("{}", tuple.join(" ; "));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relation_scanner() {
        let record1 = Record::new(vec!["1".to_string(), "John".to_string()]);
        let record2 = Record::new(vec!["2".to_string(), "Jane".to_string()]);
        let records = vec![record1.clone(), record2.clone()];
        let mut scanner = RelationScanner::new(records);

        let result = scanner.get_next_record();
        assert_eq!(result, Some(record1), "The first record should be correct.");

        let result = scanner.get_next_record();
        assert_eq!(
            result,
            Some(record2),
            "The second record should be correct."
        );

        let result = scanner.get_next_record();
        assert_eq!(result, None, "There should be no more records.");
    }

    #[test]

    fn test_record_printer() {
        let record1 = Record::new(vec!["1".to_string(), "John".to_string()]);
        let record2 = Record::new(vec!["2".to_string(), "Jane".to_string()]);
        let records = vec![record1.clone(), record2.clone()];
        let col_info = Rc::new(vec![
            ColInfo::new("id".to_string(), "INT".to_string()),
            ColInfo::new("name".to_string(), "VARCHAR".to_string()),
        ]);

        let scanner = Box::new(RelationScanner::new(records));
        let projection_operator = ProjectionOperator::new(
            vec!["id".to_string(), "name".to_string()],
            scanner,
            col_info,
        );
        let mut printer = RecordPrinter::new(Box::new(projection_operator));

        printer.print_records();
    }
}
