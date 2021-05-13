extern crate byteorder;
extern crate id_arena;
extern crate num_enum;

mod error;
mod interleaved;
mod object_map;
mod object_path;
mod properties;
mod segment;
mod tdms_reader;
mod toc;
mod types;

use crate::error::{Result, TdmsReadError};
use crate::object_path::{path_from_channel, path_from_group, ObjectPathId};
use crate::tdms_reader::{read_metadata, TdmsReader};
pub use crate::types::NativeType;
use std::io::{BufReader, Read, Seek};

pub struct TdmsFile<R: Read + Seek> {
    reader: BufReader<R>,
    metadata: TdmsReader,
}

pub struct Group<'a, R: Read + Seek> {
    file: &'a mut TdmsFile<R>,
    _object_id: ObjectPathId,
    name: &'a str,
}

pub struct Channel<'a, R: Read + Seek> {
    file: &'a mut TdmsFile<R>,
    object_id: ObjectPathId,
}

pub struct GroupIterator<'a, R: Read + Seek> {
    _file: &'a mut TdmsFile<R>,
}

pub struct ChannelIterator<'a, R: Read + Seek> {
    _file: &'a mut TdmsFile<R>,
}

impl<R: Read + Seek> TdmsFile<R> {
    /// Create a new TdmsFile object, parsing TDMS metadata from the reader
    pub fn new(reader: R) -> Result<TdmsFile<R>> {
        let mut reader = BufReader::new(reader);
        let metadata = read_metadata(&mut reader)?;
        Ok(TdmsFile { reader, metadata })
    }

    /// Get a group within the TDMS file
    pub fn group<'a>(&'a mut self, group_name: &'a str) -> Option<Group<'a, R>> {
        let group_path = path_from_group(group_name);
        self.metadata
            .get_object_id(&group_path)
            .map(move |object_id| Group::new(self, group_name, object_id))
    }

    /// Get an iterator over groups within this TDMS file
    pub fn groups<'a>(&'a mut self) -> GroupIterator<'a, R> {
        GroupIterator { _file: self }
    }
}

impl<'a, R: Read + Seek> Group<'a, R> {
    fn new(file: &'a mut TdmsFile<R>, name: &'a str, object_id: ObjectPathId) -> Group<'a, R> {
        Group {
            file,
            _object_id: object_id,
            name,
        }
    }

    /// Get a channel within this group
    pub fn channel<'b>(&'b mut self, channel_name: &str) -> Option<Channel<'b, R>> {
        let channel_path = path_from_channel(self.name, channel_name);
        self.file
            .metadata
            .get_object_id(&channel_path)
            .map(move |object_id| Channel::new(self.file, object_id))
    }

    /// Get an iterator over channels within this group
    pub fn channels<'b>(&'b mut self) -> ChannelIterator<'b, R> {
        ChannelIterator { _file: self.file }
    }
}

impl<'a, R: Read + Seek> Channel<'a, R> {
    fn new(file: &'a mut TdmsFile<R>, object_id: ObjectPathId) -> Channel<'a, R> {
        Channel { file, object_id }
    }

    /// Get the total number of values in this channel
    pub fn len(&'a self) -> u64 {
        match self.file.metadata.get_channel_data_index(self.object_id) {
            Some(channel_data) => channel_data.number_of_values,
            None => 0,
        }
    }

    /// Read all data for this channel into the given buffer.
    pub fn read_data<T: NativeType>(&'a mut self, buffer: &mut Vec<T>) -> Result<()> {
        match self.file.metadata.get_channel_data_index(self.object_id) {
            Some(channel_data_index) => {
                let tdms_type = channel_data_index.data_type;
                let expected_native_type = tdms_type.native_type();
                match expected_native_type {
                    Some(expected_native_type) if expected_native_type == T::native_type() => {
                        // Buffer type matches expected native type, safe to read data
                        buffer.reserve(channel_data_index.number_of_values as usize);
                        self.file.metadata.read_channel_data(
                            &mut self.file.reader,
                            self.object_id,
                            buffer,
                        )?;
                        Ok(())
                    }
                    Some(expected_native_type) => Err(TdmsReadError::TdmsError(format!(
                        "Expected a buffer with item type {:?}",
                        expected_native_type
                    ))),
                    None => Err(TdmsReadError::TdmsError(format!(
                        "Reading data of type {:?} is not supported",
                        tdms_type
                    ))),
                }
            }
            None => Ok(()),
        }
    }
}

impl<'a, R: Read + Seek> Iterator for GroupIterator<'a, R> {
    type Item = Group<'a, R>;

    fn next(&mut self) -> Option<Group<'a, R>> {
        None
    }
}

impl<'a, R: Read + Seek> Iterator for ChannelIterator<'a, R> {
    type Item = Channel<'a, R>;

    fn next(&mut self) -> Option<Channel<'a, R>> {
        None
    }
}

impl<R: Read + Seek> std::fmt::Debug for TdmsFile<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TdmsFile").finish()
    }
}

impl<'a, R: Read + Seek> std::fmt::Debug for Group<'a, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Group").finish()
    }
}

impl<'a, R: Read + Seek> std::fmt::Debug for Channel<'a, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Group").finish()
    }
}
