extern crate num_enum;
extern crate typed_arena;

mod error;
mod object_path;
mod properties;
mod tdms_reader;
mod toc;
mod types;

use crate::error::Result;
use crate::tdms_reader::{read_metadata, RawDataIndex, TdmsReader};
use std::io::{BufReader, Read, Seek};
use typed_arena::Arena;

pub struct TdmsFile<'a, T: Read + Seek> {
    pub reader: BufReader<T>,
    pub metadata: TdmsReader<'a>,
}

impl<'a, T: Read + Seek> std::fmt::Debug for TdmsFile<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TdmsFile").finish()
    }
}

impl<'a, T: Read + Seek> TdmsFile<'a, T> {
    pub fn new(reader: T, arena: &'a Arena<RawDataIndex>) -> Result<TdmsFile<'a, T>> {
        let mut reader = BufReader::new(reader);
        let metadata = read_metadata(&mut reader, arena)?;
        Ok(TdmsFile { reader, metadata })
    }
}
