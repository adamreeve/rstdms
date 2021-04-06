use crate::error::{Result, TdmsReadError};
use crate::properties::TdmsProperty;
use crate::types::{LittleEndianReader, TypeReader};
use std::collections::HashMap;
use std::io::{Read, Seek};

pub struct TdmsMetadata {
    pub properties: HashMap<u32, TdmsProperty>,
}

pub fn read_metadata<T: Read + Seek>(reader: &mut T) -> Result<TdmsMetadata> {
    let mut properties = HashMap::new();
    loop {
        match read_segment(reader, &mut properties) {
            Err(e) => return Err(e),
            Ok(None) => {
                // Reached end of file
                break;
            }
            Ok(Some(())) => {
                // Continue
            }
        }
    }
    Ok(TdmsMetadata { properties })
}

fn read_segment<T: Read + Seek>(
    reader: &mut T,
    properties: &mut HashMap<u32, TdmsProperty>,
) -> Result<Option<()>> {
    let mut header_bytes = [0u8; 4];
    let mut bytes_read = 0;
    while bytes_read < 4 {
        match reader.read(&mut header_bytes[bytes_read..])? {
            0 => return Ok(None),
            n => bytes_read += n,
        }
    }

    // Check segment header
    let expected_header = [0x54, 0x44, 0x53, 0x6d];
    if header_bytes != expected_header {
        return Err(TdmsReadError::TdmsError(String::from(
            "Invalid segment header",
        )));
    }

    let mut type_reader = LittleEndianReader::new(reader);
    let toc_mask = type_reader.read_uint32()?;

    // TODO: Check endianness from ToC mask
    let mut type_reader = LittleEndianReader::new(reader);

    let version = type_reader.read_int32()?;
    let next_segment_offset = type_reader.read_uint64()?;
    let raw_data_offset = type_reader.read_uint64()?;

    println!("Read segment with toc_mask = {}, version = {}, next_segment_offset = {}, raw_data_offset = {}",
            toc_mask, version, next_segment_offset, raw_data_offset);

    Ok(Some(()))
}
