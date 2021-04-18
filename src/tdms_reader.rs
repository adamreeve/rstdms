use crate::error::{Result, TdmsReadError};
use crate::object_path::{ObjectPathCache, ObjectPathId};
use crate::properties::TdmsProperty;
use crate::toc::{TocFlag, TocMask};
use crate::types::{LittleEndianReader, TdsType, TypeReader};
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use typed_arena::Arena;

const RAW_DATA_INDEX_NO_DATA: u32 = 0xFFFFFFFF;
const RAW_DATA_INDEX_MATCHES_PREVIOUS: u32 = 0x00000000;
const FORMAT_CHANGING_SCALER: u32 = 0x00001269;
const DIGITAL_LINE_SCALER: u32 = 0x0000126A;

#[derive(Debug)]
pub struct TdmsSegment<'a> {
    data_position: u64,
    next_segment_position: u64,
    objects: Vec<SegmentObject<'a>>,
}

impl<'a> TdmsSegment<'a> {
    fn new(
        data_position: u64,
        next_segment_position: u64,
        objects: Vec<SegmentObject<'a>>,
    ) -> TdmsSegment<'a> {
        TdmsSegment {
            data_position,
            next_segment_position,
            objects,
        }
    }
}

#[derive(Debug)]
pub struct SegmentObject<'a> {
    pub object_id: ObjectPathId,
    pub raw_data_index: Option<&'a RawDataIndex>,
}

impl<'a> SegmentObject<'a> {
    pub fn no_data(object_id: ObjectPathId) -> SegmentObject<'a> {
        SegmentObject {
            object_id,
            raw_data_index: None,
        }
    }

    pub fn with_data(
        object_id: ObjectPathId,
        raw_data_index: &'a RawDataIndex,
    ) -> SegmentObject<'a> {
        SegmentObject {
            object_id,
            raw_data_index: Some(raw_data_index),
        }
    }
}

#[derive(Debug)]
pub struct RawDataIndex {
    pub number_of_values: u64,
    pub data_type: TdsType,
    pub data_size: u64,
}

pub struct RawDataIndexCache<'a> {
    prev_raw_data_indexes: Vec<Option<&'a RawDataIndex>>,
}

impl<'a> RawDataIndexCache<'a> {
    fn new() -> RawDataIndexCache<'a> {
        RawDataIndexCache {
            prev_raw_data_indexes: Vec::new(),
        }
    }

    fn set_raw_data_index(&mut self, object: ObjectPathId, raw_data_index: &'a RawDataIndex) {
        let index = object.as_usize();
        if index >= self.prev_raw_data_indexes.len() {
            let padding_length = index - self.prev_raw_data_indexes.len();
            self.prev_raw_data_indexes.reserve(1 + padding_length);
            for _ in 0..padding_length {
                self.prev_raw_data_indexes.push(None);
            }
            self.prev_raw_data_indexes.push(Some(raw_data_index));
        } else {
            self.prev_raw_data_indexes[index] = Some(raw_data_index);
        }
    }

    fn get_raw_data_index(&self, object: ObjectPathId) -> Option<&'a RawDataIndex> {
        match self.prev_raw_data_indexes.get(object.as_usize()) {
            Some(option) => *option,
            _ => None,
        }
    }
}

pub struct TdmsReader<'a> {
    pub properties: HashMap<ObjectPathId, Vec<TdmsProperty>>,
    object_paths: ObjectPathCache,
    data_indexes: &'a Arena<RawDataIndex>,
    raw_data_index_cache: RawDataIndexCache<'a>,
}

impl<'a> TdmsReader<'a> {
    fn new(arena: &'a Arena<RawDataIndex>) -> TdmsReader<'a> {
        TdmsReader {
            properties: HashMap::new(),
            object_paths: ObjectPathCache::new(),
            data_indexes: arena,
            raw_data_index_cache: RawDataIndexCache::new(),
        }
    }

    fn read_segment<T: Read + Seek>(
        &mut self,
        reader: &mut T,
        position: u64,
    ) -> Result<Option<TdmsSegment>> {
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
            return Err(TdmsReadError::TdmsError(format!(
                "Invalid segment header at position {}: {:?}",
                position, header_bytes,
            )));
        }

        let mut type_reader = LittleEndianReader::new(reader);
        let toc_mask = TocMask::from_flags(type_reader.read_uint32()?);

        // TODO: Check endianness from ToC mask
        let mut type_reader = LittleEndianReader::new(reader);

        let version = type_reader.read_int32()?;
        let next_segment_offset = type_reader.read_uint64()?;
        let raw_data_offset = type_reader.read_uint64()?;

        let lead_in_length = 28;
        let next_segment_position = position + lead_in_length + next_segment_offset;
        let raw_data_position = position + lead_in_length + raw_data_offset;

        println!("Read segment with toc_mask = {}, version = {}, next_segment_offset = {}, raw_data_offset = {}",
                toc_mask, version, next_segment_offset, raw_data_offset);

        let segment_objects = if toc_mask.has_flag(TocFlag::MetaData) {
            self.read_object_metadata(&mut type_reader, &toc_mask)?
        } else {
            unimplemented!();
        };

        Ok(Some(TdmsSegment::new(
            raw_data_position,
            next_segment_position,
            segment_objects,
        )))
    }

    fn read_object_metadata<T: TypeReader>(
        &mut self,
        reader: &mut T,
        toc_mask: &TocMask,
    ) -> Result<Vec<SegmentObject<'a>>> {
        if !toc_mask.has_flag(TocFlag::NewObjList) {
            unimplemented!();
        }

        let num_objects = reader.read_uint32()?;
        let mut segment_objects = Vec::with_capacity(num_objects as usize);
        for _ in 0..num_objects {
            let object_path = reader.read_string()?;
            let object_id = self.object_paths.get_or_create_id(object_path);
            let raw_data_index_header = reader.read_uint32()?;
            let segment_object = match raw_data_index_header {
                RAW_DATA_INDEX_NO_DATA => SegmentObject::no_data(object_id),
                RAW_DATA_INDEX_MATCHES_PREVIOUS => {
                    match self.raw_data_index_cache.get_raw_data_index(object_id) {
                        Some(ref raw_data_index) => {
                            SegmentObject::with_data(object_id, raw_data_index)
                        }
                        None => {
                            return Err(TdmsReadError::TdmsError(format!(
                                "Object has no previous raw data index"
                            )))
                        }
                    }
                }
                FORMAT_CHANGING_SCALER => unimplemented!(),
                DIGITAL_LINE_SCALER => unimplemented!(),
                _ => {
                    // Raw data index header gives length of index information
                    let raw_data_index = self.data_indexes.alloc(read_raw_data_index(reader)?);
                    self.raw_data_index_cache
                        .set_raw_data_index(object_id, raw_data_index);
                    SegmentObject::with_data(object_id, raw_data_index)
                }
            };
            segment_objects.push(segment_object);
            let num_properties = reader.read_uint32()?;
            for _ in 0..num_properties {
                let property = TdmsProperty::read(reader)?;
                self.properties
                    .entry(object_id)
                    .or_insert_with(|| Vec::new())
                    .push(property);
            }
        }

        Ok(segment_objects)
    }
}

pub fn read_metadata<'a, T: Read + Seek>(
    reader: &mut T,
    arena: &'a Arena<RawDataIndex>,
) -> Result<TdmsReader<'a>> {
    let mut tdms_reader = TdmsReader::new(arena);
    loop {
        let position = reader.seek(SeekFrom::Current(0))?;
        match tdms_reader.read_segment(reader, position) {
            Err(e) => return Err(e),
            Ok(None) => {
                // Reached end of file
                break;
            }
            Ok(Some(segment)) => {
                // Seek to the start of the next segment
                reader.seek(SeekFrom::Start(segment.next_segment_position))?;
            }
        }
    }
    Ok(tdms_reader)
}

fn read_raw_data_index<T: TypeReader>(reader: &mut T) -> Result<RawDataIndex> {
    let data_type = reader.read_uint32()?;
    let data_type = TdsType::from_u32(data_type)?;
    let dimension = reader.read_uint32()?;
    let number_of_values = reader.read_uint64()?;

    if dimension != 1 {
        return Err(TdmsReadError::TdmsError(format!(
            "Dimension must be 1, got {}",
            dimension
        )));
    }

    let data_size = match data_type.size() {
        Some(type_size) => (type_size as u64) * number_of_values,
        None => {
            if data_type == TdsType::String {
                reader.read_uint64()?
            } else {
                return Err(TdmsReadError::TdmsError(format!(
                    "Unsupported data type: {:?}",
                    data_type
                )));
            }
        }
    };
    Ok(RawDataIndex {
        number_of_values,
        data_type,
        data_size,
    })
}
