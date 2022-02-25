use crate::error::{Result, TdmsReadError};
use crate::object_map::ObjectMap;
use crate::object_path::{ObjectPath, ObjectPathCache, ObjectPathId};
use crate::properties::TdmsProperty;
use crate::segment::{RawDataIndex, RawDataIndexCache, SegmentObject, TdmsSegment};
use crate::toc::{TocFlag, TocMask};
use crate::types::{read_string, ByteOrderExt, NativeType, TdsType};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use id_arena::Arena;
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};

const RAW_DATA_INDEX_NO_DATA: u32 = 0xFFFFFFFF;
const RAW_DATA_INDEX_MATCHES_PREVIOUS: u32 = 0x00000000;
const FORMAT_CHANGING_SCALER: u32 = 0x00001269;
const DIGITAL_LINE_SCALER: u32 = 0x0000126A;

pub fn read_metadata<R: Read + Seek>(reader: &mut R) -> Result<TdmsReader> {
    let mut tdms_reader = TdmsReader::new();
    match tdms_reader.read_segments(reader) {
        Ok(()) => Ok(tdms_reader),
        Err(e) => Err(e),
    }
}

pub struct ChannelDataIndex {
    pub number_of_values: u64,
    pub data_type: TdsType,
}

impl ChannelDataIndex {
    fn from_segment_index(index: &RawDataIndex) -> ChannelDataIndex {
        ChannelDataIndex {
            data_type: index.data_type,
            number_of_values: index.number_of_values,
        }
    }

    fn update_with_segment_index(&mut self, index: &RawDataIndex) -> Result<()> {
        // We have data in this segment for an object that already had data in a
        // previous segment, check the raw data index is compatible.
        if index.data_type != self.data_type {
            return Err(TdmsReadError::TdmsError(format!(
                "Data type {:?} does not match existing data type {:?}",
                index.data_type, self.data_type
            )));
        }
        self.number_of_values += index.number_of_values;
        Ok(())
    }
}

type ChannelDataIndexMap = ObjectMap<ChannelDataIndex>;

pub struct TdmsReader {
    pub properties: HashMap<ObjectPathId, Vec<TdmsProperty>>,
    object_paths: ObjectPathCache,
    data_indexes: Arena<RawDataIndex>,
    raw_data_index_cache: RawDataIndexCache,
    segments: Vec<TdmsSegment>,
    channel_data_index_map: ChannelDataIndexMap,
}

impl TdmsReader {
    fn new() -> TdmsReader {
        TdmsReader {
            properties: HashMap::new(),
            object_paths: ObjectPathCache::new(),
            data_indexes: Arena::<RawDataIndex>::new(),
            raw_data_index_cache: RawDataIndexCache::new(),
            segments: Vec::new(),
            channel_data_index_map: ChannelDataIndexMap::new(),
        }
    }

    pub fn get_object_id(&self, path: &str) -> Option<ObjectPathId> {
        self.object_paths.get_id(path)
    }

    pub fn get_object_path(&self, object_path_id: ObjectPathId) -> Option<&ObjectPath> {
        self.object_paths.get_path(object_path_id)
    }

    pub fn objects(&self) -> impl Iterator<Item = (ObjectPathId, &ObjectPath)> {
        self.object_paths.objects()
    }

    pub fn get_channel_data_index(&self, object_id: ObjectPathId) -> Option<&ChannelDataIndex> {
        self.channel_data_index_map.get(object_id)
    }

    pub fn read_channel_data<R: Read + Seek, T: NativeType>(
        &self,
        reader: &mut R,
        channel_id: ObjectPathId,
        buffer: &mut [T],
    ) -> Result<()> {
        let mut offset = 0;
        for segment in self.segments.iter() {
            if segment
                .objects
                .iter()
                .any(|o| o.object_id == channel_id && o.raw_data_index.is_some())
            {
                offset += segment.read_channel_data(
                    reader,
                    channel_id,
                    &mut buffer[offset..],
                    &self.data_indexes,
                )?;
            }
        }
        Ok(())
    }

    fn read_segments<R: Read + Seek>(&mut self, reader: &mut R) -> Result<()> {
        let mut object_merger = ObjectMerger::new();
        loop {
            let position = reader.seek(SeekFrom::Current(0))?;
            match self.read_segment(reader, position, &mut object_merger) {
                Err(e) => return Err(e),
                Ok(None) => {
                    // Reached end of file
                    break;
                }
                Ok(Some(segment)) => {
                    // Seek to the start of the next segment
                    reader.seek(SeekFrom::Start(segment.next_segment_position))?;
                    self.segments.push(segment);
                }
            }
        }
        Ok(())
    }

    fn read_segment<R: Read + Seek>(
        &mut self,
        reader: &mut R,
        position: u64,
        object_merger: &mut ObjectMerger,
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

        let toc_mask = TocMask::from_flags(reader.read_u32::<LittleEndian>()?);

        if toc_mask.has_flag(TocFlag::BigEndian) {
            self.read_segment_metadata::<R, BigEndian>(reader, toc_mask, position, object_merger)
        } else {
            self.read_segment_metadata::<R, LittleEndian>(reader, toc_mask, position, object_merger)
        }
    }

    fn read_segment_metadata<R: Read + Seek, O: ByteOrderExt>(
        &mut self,
        reader: &mut R,
        toc_mask: TocMask,
        position: u64,
        object_merger: &mut ObjectMerger,
    ) -> Result<Option<TdmsSegment>> {
        let _version = reader.read_i32::<O>()?;
        let next_segment_offset = reader.read_u64::<O>()?;
        let raw_data_offset = reader.read_u64::<O>()?;

        let lead_in_length = 28;
        let next_segment_position = position + lead_in_length + next_segment_offset;
        let raw_data_position = position + lead_in_length + raw_data_offset;

        let segment_objects = if toc_mask.has_flag(TocFlag::MetaData) {
            let this_segment_objects = self.read_object_metadata::<R, O>(reader)?;
            if toc_mask.has_flag(TocFlag::NewObjList) {
                this_segment_objects
            } else {
                // Not a new object list so merge with previous segment objects
                let prev_objs = self.segments.last().map(|segment| &segment.objects);
                object_merger.merge_objects(prev_objs, this_segment_objects)
            }
        } else {
            // No meta data in this segment, re-use metadata from the previous segment
            match self.segments.last() {
                // TODO: Share references to object vectors?
                Some(segment) => segment.objects.to_vec(),
                None => Vec::new(),
            }
        };

        self.update_data_indexes(&segment_objects)?;

        Ok(Some(TdmsSegment::new(
            toc_mask,
            raw_data_position,
            next_segment_position,
            segment_objects,
        )))
    }

    fn read_object_metadata<R: Read, O: ByteOrderExt>(
        &mut self,
        reader: &mut R,
    ) -> Result<Vec<SegmentObject>> {
        let num_objects = reader.read_u32::<O>()?;
        let mut segment_objects = Vec::with_capacity(num_objects as usize);
        for _ in 0..num_objects {
            let object_path = read_string::<R, O>(reader)?;
            let object_id = self.object_paths.get_or_create_id(object_path)?;
            let raw_data_index_header = reader.read_u32::<O>()?;
            let segment_object = match raw_data_index_header {
                RAW_DATA_INDEX_NO_DATA => SegmentObject::no_data(object_id),
                RAW_DATA_INDEX_MATCHES_PREVIOUS => match self.raw_data_index_cache.get(object_id) {
                    Some(raw_data_index_id) => {
                        SegmentObject::with_data(object_id, *raw_data_index_id)
                    }
                    None => {
                        return Err(TdmsReadError::TdmsError(String::from(
                            "Object has no previous raw data index",
                        )))
                    }
                },
                FORMAT_CHANGING_SCALER => unimplemented!(),
                DIGITAL_LINE_SCALER => unimplemented!(),
                _ => {
                    // Raw data index header gives length of index information
                    let raw_data_index = self
                        .data_indexes
                        .alloc(read_raw_data_index::<R, O>(reader)?);
                    self.raw_data_index_cache.set(object_id, raw_data_index);
                    SegmentObject::with_data(object_id, raw_data_index)
                }
            };
            segment_objects.push(segment_object);
            let num_properties = reader.read_u32::<O>()?;
            for _ in 0..num_properties {
                let property = TdmsProperty::read::<_, O>(reader)?;
                self.properties
                    .entry(object_id)
                    .or_insert_with(Vec::new)
                    .push(property);
            }
        }

        Ok(segment_objects)
    }

    /// Update the channel data indexes with data indexes for the current objects in a segment
    fn update_data_indexes(&mut self, segment_objects: &[SegmentObject]) -> Result<()> {
        for segment_obj in segment_objects {
            if let Some(segment_data_index_id) = segment_obj.raw_data_index {
                // If we have a valid raw data index id it must correspond to a raw data index
                // in data_indexes so unwrap here is safe.
                let segment_raw_data_index = self.data_indexes.get(segment_data_index_id).unwrap();
                let existing_data_index =
                    self.channel_data_index_map.get_mut(segment_obj.object_id);
                match existing_data_index {
                    Some(existing_data_index) => {
                        existing_data_index.update_with_segment_index(segment_raw_data_index)?;
                    }
                    None => {
                        let new_data_index =
                            ChannelDataIndex::from_segment_index(segment_raw_data_index);
                        self.channel_data_index_map
                            .set(segment_obj.object_id, new_data_index);
                    }
                }
            }
        }
        Ok(())
    }
}

struct ObjectMerger {
    object_indexes: ObjectMap<usize>,
}

impl ObjectMerger {
    pub fn new() -> ObjectMerger {
        ObjectMerger {
            object_indexes: ObjectMap::new(),
        }
    }

    /// Combine previous segment's object list with objects in the current segment
    pub fn merge_objects(
        &mut self,
        previous_segment_objects: Option<&Vec<SegmentObject>>,
        new_objects: Vec<SegmentObject>,
    ) -> Vec<SegmentObject> {
        if let Some(prev_objs) = previous_segment_objects {
            let mut merged_objects = Vec::with_capacity(prev_objs.len());
            // Store indexes of existing objects and add to merged vector
            self.object_indexes.clear();
            for (i, obj) in prev_objs.iter().enumerate() {
                self.object_indexes.set(obj.object_id, i);
                merged_objects.push(obj.clone());
            }
            // Replace or push new objects
            for obj in new_objects {
                match self.object_indexes.get(obj.object_id) {
                    Some(i) => {
                        merged_objects[*i] = obj;
                    }
                    None => {
                        merged_objects.push(obj);
                    }
                }
            }
            merged_objects
        } else {
            new_objects
        }
    }
}

fn read_raw_data_index<R: Read, O: ByteOrderExt>(reader: &mut R) -> Result<RawDataIndex> {
    let data_type = reader.read_u32::<O>()?;
    let data_type = TdsType::from_u32(data_type)?;
    let dimension = reader.read_u32::<O>()?;
    let number_of_values = reader.read_u64::<O>()?;

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
                reader.read_u64::<O>()?
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
