extern crate num_enum;

mod error;
mod object_path;
mod properties;
mod tdms_reader;
mod toc;
mod types;

use crate::error::Result;
use crate::tdms_reader::{read_metadata, TdmsMetadata};
use std::io::{BufReader, Read, Seek};

pub struct TdmsFile<T: Read + Seek> {
    pub reader: BufReader<T>,
    pub metadata: TdmsMetadata,
}

impl<T: Read + Seek> TdmsFile<T> {
    pub fn new(reader: T) -> Result<TdmsFile<T>> {
        let mut reader = BufReader::new(reader);
        let metadata = read_metadata(&mut reader)?;
        Ok(TdmsFile { reader, metadata })
    }
}
