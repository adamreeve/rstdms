extern crate id_arena;
extern crate num_enum;

mod error;
mod object_path;
mod properties;
mod tdms_reader;
mod toc;
mod types;

use crate::error::Result;
use crate::tdms_reader::{read_metadata, TdmsReader};
use std::io::{BufReader, Read, Seek};

pub struct TdmsFile<T: Read + Seek> {
    _reader: BufReader<T>,
    _metadata: TdmsReader,
}

impl<T: Read + Seek> std::fmt::Debug for TdmsFile<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TdmsFile").finish()
    }
}

impl<T: Read + Seek> TdmsFile<T> {
    pub fn new(reader: T) -> Result<TdmsFile<T>> {
        let mut reader = BufReader::new(reader);
        let metadata = read_metadata(&mut reader)?;
        Ok(TdmsFile {
            _reader: reader,
            _metadata: metadata,
        })
    }
}
