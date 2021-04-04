#[macro_use]
extern crate num_derive;
extern crate num_traits;
extern crate thiserror;

mod error;
mod object_path;
mod properties;
mod types;

use std::io::{BufReader, Read, Seek};

pub struct TdmsFile<T: Read + Seek> {
    reader: BufReader<T>,
}

impl<T: Read + Seek> TdmsFile<T> {
    pub fn new(reader: T) -> TdmsFile<T> {
        let reader = BufReader::new(reader);
        TdmsFile { reader }
    }
}
