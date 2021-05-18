use crate::error::{Result, TdmsReadError};
use crate::interleaved::InterleavedReader;
use crate::object_map::ObjectMap;
use crate::object_path::ObjectPathId;
use crate::toc::{TocFlag, TocMask};
use crate::types::{ByteOrderWrapper, NativeType, TdsType};
use byteorder::{BigEndian, LittleEndian};
use id_arena::{Arena, Id};
use std::io::{Read, Seek, SeekFrom};

#[derive(Debug)]
pub struct TdmsSegment {
    pub next_segment_position: u64,
    pub objects: Vec<SegmentObject>,
    toc_mask: TocMask,
    data_position: u64,
}

impl TdmsSegment {
    pub fn new(
        toc_mask: TocMask,
        data_position: u64,
        next_segment_position: u64,
        objects: Vec<SegmentObject>,
    ) -> TdmsSegment {
        TdmsSegment {
            toc_mask,
            data_position,
            next_segment_position,
            objects,
        }
    }

    pub fn read_channel_data<R: Read + Seek, T: NativeType>(
        &self,
        reader: &mut R,
        channel_id: ObjectPathId,
        buffer: &mut Vec<T>,
        raw_data_indexes: &Arena<RawDataIndex>,
    ) -> Result<()> {
        let interleaved = self.toc_mask.has_flag(TocFlag::InterleavedData);
        let big_endian = self.toc_mask.has_flag(TocFlag::BigEndian);
        match (interleaved, big_endian) {
            (false, false) => self.read_contiguous_channel_data::<_, _, LittleEndian>(
                reader,
                channel_id,
                buffer,
                raw_data_indexes,
            ),
            (false, true) => self.read_contiguous_channel_data::<_, _, BigEndian>(
                reader,
                channel_id,
                buffer,
                raw_data_indexes,
            ),
            (true, false) => self.read_interleaved_channel_data::<_, _, LittleEndian>(
                reader,
                channel_id,
                buffer,
                raw_data_indexes,
            ),
            (true, true) => self.read_interleaved_channel_data::<_, _, BigEndian>(
                reader,
                channel_id,
                buffer,
                raw_data_indexes,
            ),
        }
    }

    fn read_contiguous_channel_data<R: Read + Seek, T: NativeType, O: ByteOrderWrapper>(
        &self,
        reader: &mut R,
        channel_id: ObjectPathId,
        buffer: &mut Vec<T>,
        raw_data_indexes: &Arena<RawDataIndex>,
    ) -> Result<()> {
        let mut channel_offset = 0;
        for obj in self.objects.iter() {
            if let Some(raw_data_index_id) = obj.raw_data_index {
                let raw_data_index = raw_data_indexes.get(raw_data_index_id).unwrap();
                if obj.object_id == channel_id {
                    reader.seek(SeekFrom::Start(self.data_position + channel_offset))?;
                    T::read_values::<_, O>(
                        buffer,
                        reader,
                        raw_data_index.number_of_values as usize,
                    )?;
                    break;
                } else {
                    channel_offset += raw_data_index.data_size;
                }
            }
        }
        Ok(())
    }

    fn read_interleaved_channel_data<R: Read + Seek, T: NativeType, O: ByteOrderWrapper>(
        &self,
        reader: &mut R,
        channel_id: ObjectPathId,
        buffer: &mut Vec<T>,
        raw_data_indexes: &Arena<RawDataIndex>,
    ) -> Result<()> {
        let mut length = None;
        let mut channel_params = None;
        let mut chunk_width = 0;

        for obj in self.objects.iter() {
            if let Some(raw_data_index_id) = obj.raw_data_index {
                let raw_data_index = raw_data_indexes.get(raw_data_index_id).unwrap();
                let type_size = raw_data_index.data_type.size().ok_or_else(|| {
                    TdmsReadError::TdmsError(format!(
                        "Cannot read unsized data type {:?} in interleaved data chunk",
                        raw_data_index.data_type
                    ))
                })?;
                match length {
                    None => length = Some(raw_data_index.number_of_values),
                    Some(length) => {
                        if raw_data_index.number_of_values != length {
                            return Err(TdmsReadError::TdmsError(format!(
                                "Different data lengths in interleaved data segment. Expected length {} but got {}",
                                length, raw_data_index.number_of_values)));
                        }
                    }
                }
                if obj.object_id == channel_id {
                    channel_params = Some((type_size, chunk_width));
                }
                chunk_width += type_size;
            }
        }

        if let (Some((type_size, channel_offset)), Some(length)) = (channel_params, length) {
            let mut chunk = vec![0; (length as usize) * (chunk_width as usize)];
            reader.seek(SeekFrom::Start(self.data_position))?;
            reader.read_exact(&mut chunk)?;
            let mut interleaved_reader = InterleavedReader::new(
                &chunk,
                chunk_width as usize,
                type_size as usize,
                channel_offset as usize,
            );
            T::read_values::<_, O>(buffer, &mut interleaved_reader, length as usize)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SegmentObject {
    pub object_id: ObjectPathId,
    pub raw_data_index: Option<RawDataIndexId>,
}

impl SegmentObject {
    pub fn no_data(object_id: ObjectPathId) -> SegmentObject {
        SegmentObject {
            object_id,
            raw_data_index: None,
        }
    }

    pub fn with_data(object_id: ObjectPathId, raw_data_index: RawDataIndexId) -> SegmentObject {
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

pub type RawDataIndexId = Id<RawDataIndex>;

pub type RawDataIndexCache = ObjectMap<RawDataIndexId>;
