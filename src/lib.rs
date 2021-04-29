extern crate id_arena;
extern crate num_enum;

mod error;
mod object_map;
mod object_path;
mod properties;
mod tdms_reader;
mod toc;
mod types;

use crate::error::{Result, TdmsReadError};
use crate::object_path::{path_from_channel, path_from_group, ObjectPathId};
use crate::tdms_reader::{read_metadata, TdmsReader};
use crate::types::NativeType;
use std::io::{BufReader, Read, Seek};

pub struct TdmsFile<T: Read + Seek> {
    _reader: BufReader<T>,
    metadata: TdmsReader,
}

pub struct Group<'a, T: Read + Seek> {
    file: &'a mut TdmsFile<T>,
    _object_id: Option<ObjectPathId>,
    name: &'a str,
}

pub struct Channel<'a, T: Read + Seek> {
    file: &'a mut TdmsFile<T>,
    object_id: ObjectPathId,
}

impl<T: Read + Seek> TdmsFile<T> {
    /// Create a new TdmsFile object, parsing TDMS metadata from the reader
    pub fn new(reader: T) -> Result<TdmsFile<T>> {
        let mut reader = BufReader::new(reader);
        let metadata = read_metadata(&mut reader)?;
        Ok(TdmsFile {
            _reader: reader,
            metadata,
        })
    }

    /// Get a group within the TDMS file
    pub fn group<'a>(&'a mut self, group_name: &'a str) -> Option<Group<'a, T>> {
        let group_path = path_from_group(group_name);
        match self.metadata.get_object_id(&group_path) {
            Some(object_id) => Some(Group::new(self, group_name, Some(object_id))),
            // It's currently possible to have a group without an associated object path if there is no
            // metadata associated with this group.
            // TODO: We want to return None here if the group really doesn't exist, so make sure the group
            // path entry exists when any channel path is created?
            None => Some(Group::new(self, group_name, None)),
        }
    }
}

impl<'a, T: Read + Seek> Group<'a, T> {
    fn new(
        file: &'a mut TdmsFile<T>,
        name: &'a str,
        object_id: Option<ObjectPathId>,
    ) -> Group<'a, T> {
        Group {
            file,
            _object_id: object_id,
            name,
        }
    }

    /// Get a channel within this group
    pub fn channel(&'a mut self, channel_name: &str) -> Option<Channel<'a, T>> {
        let channel_path = path_from_channel(self.name, channel_name);
        match self.file.metadata.get_object_id(&channel_path) {
            Some(object_id) => Some(Channel::new(self.file, object_id)),
            None => None,
        }
    }
}

impl<'a, T: Read + Seek> Channel<'a, T> {
    fn new(file: &'a mut TdmsFile<T>, object_id: ObjectPathId) -> Channel<'a, T> {
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
    pub fn read_data<B: NativeType>(&'a mut self, buffer: &mut Vec<B>) -> Result<()> {
        match self.file.metadata.get_channel_data_index(self.object_id) {
            Some(channel_data_index) => {
                let tdms_type = channel_data_index.data_type;
                let expected_native_type = tdms_type.native_type();
                if expected_native_type.is_none() {
                    Err(TdmsReadError::TdmsError(format!(
                        "Reading data of type {:?} is not supported",
                        tdms_type
                    )))
                } else if expected_native_type != Some(B::native_type()) {
                    Err(TdmsReadError::TdmsError(format!(
                        "Expected a buffer with item type {:?}",
                        expected_native_type
                    )))
                } else {
                    // Buffer type matches expected native type, safe to read data
                    buffer.reserve(channel_data_index.number_of_values as usize);
                    unimplemented!();
                }
            }
            None => Ok(()),
        }
    }
}

impl<T: Read + Seek> std::fmt::Debug for TdmsFile<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TdmsFile").finish()
    }
}
