pub mod object_path;

use std::io::{BufReader, Read, Seek};

pub struct TdmsFile<T: Read + Seek> {
    reader: BufReader<T>,
}

impl<T: Read + Seek> TdmsFile<T> {
    pub fn new(reader: T) -> TdmsFile<T> {
        TdmsFile {
            reader: BufReader::new(reader),
        }
    }
}
