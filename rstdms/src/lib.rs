extern crate byteorder;
extern crate chrono;
extern crate id_arena;
extern crate num_enum;

mod error;
mod interleaved;
mod object_map;
mod object_path;
mod properties;
mod segment;
mod tdms_reader;
pub mod timestamp;
mod toc;
mod types;

pub use crate::error::{Result, TdmsReadError};
use crate::object_path::{path_from_channel, path_from_group, ObjectPath, ObjectPathId};
pub use crate::properties::{TdmsProperty, TdmsValue};
use crate::tdms_reader::{read_metadata, TdmsReader};
pub use crate::timestamp::Timestamp;
pub use crate::types::{NativeType, TdsType};
use std::cell::RefCell;
use std::io::{BufReader, Read, Seek};

pub struct TdmsFile<R: Read + Seek> {
    file_reader: RefCell<BufReader<R>>,
    tdms_reader: TdmsReader,
}

pub struct Group<'a, R: Read + Seek> {
    file: &'a TdmsFile<R>,
    object_id: ObjectPathId,
}

pub struct Channel<'a, R: Read + Seek> {
    file: &'a TdmsFile<R>,
    object_id: ObjectPathId,
}

pub struct GroupIterator<'a, R: Read + Seek> {
    file: &'a TdmsFile<R>,
    object_iterator: std::vec::IntoIter<ObjectPathId>,
}

pub struct ChannelIterator<'a, R: Read + Seek> {
    file: &'a TdmsFile<R>,
    object_iterator: std::vec::IntoIter<ObjectPathId>,
}

impl<R: Read + Seek> TdmsFile<R> {
    /// Create a new TdmsFile object, parsing TDMS metadata from the reader
    pub fn new(file_reader: R) -> Result<TdmsFile<R>> {
        let mut file_reader = BufReader::new(file_reader);
        let tdms_reader = read_metadata(&mut file_reader)?;
        Ok(TdmsFile {
            file_reader: RefCell::new(file_reader),
            tdms_reader,
        })
    }

    pub fn properties(&self) -> &Vec<TdmsProperty> {
        if let Some(object_id) = self.tdms_reader.get_object_id("/") {
            match self.tdms_reader.properties.get(&object_id) {
                Some(properties) => &properties,
                None => &EMPTY_PROPERITES,
            }
        } else {
            &EMPTY_PROPERITES
        }
    }

    /// Get a group within the TDMS file
    pub fn group<'a>(&'a self, group_name: &'a str) -> Option<Group<'a, R>> {
        let group_path = path_from_group(group_name);
        self.tdms_reader
            .get_object_id(&group_path)
            .map(move |object_id| Group::new(self, object_id))
    }

    /// Get an iterator over groups within this TDMS file
    pub fn groups<'a>(&'a self) -> GroupIterator<'a, R> {
        GroupIterator::new(self)
    }
}

impl<'a, R: Read + Seek> Group<'a, R> {
    fn new(file: &'a TdmsFile<R>, object_id: ObjectPathId) -> Group<'a, R> {
        Group { file, object_id }
    }

    /// Get the name of this group
    pub fn name(&self) -> &str {
        let group_path = self
            .file
            .tdms_reader
            .get_object_path(self.object_id)
            .unwrap();
        match group_path {
            ObjectPath::Group(ref group_name) => group_name,
            _ => panic!(
                "Expected a group path for object id {:?}, got {:?}",
                self.object_id, group_path
            ),
        }
    }

    pub fn properties(&self) -> &Vec<TdmsProperty> {
        match self.file.tdms_reader.properties.get(&self.object_id) {
            Some(properties) => &properties,
            None => &EMPTY_PROPERITES,
        }
    }

    /// Get a channel within this group
    pub fn channel<'b>(&'b self, channel_name: &str) -> Option<Channel<'b, R>> {
        let channel_path = path_from_channel(self.name(), channel_name);
        self.file
            .tdms_reader
            .get_object_id(&channel_path)
            .map(move |object_id| Channel::new(self.file, object_id))
    }

    /// Get an iterator over channels within this group
    pub fn channels<'b>(&'b self) -> ChannelIterator<'b, R> {
        ChannelIterator::new(self.file, self.name())
    }
}

impl<'a, R: Read + Seek> Channel<'a, R> {
    fn new(file: &'a TdmsFile<R>, object_id: ObjectPathId) -> Channel<'a, R> {
        Channel { file, object_id }
    }

    /// Get the name of this channel
    pub fn name(&self) -> &str {
        let channel_path = self
            .file
            .tdms_reader
            .get_object_path(self.object_id)
            .unwrap();
        match channel_path {
            ObjectPath::Channel(_, ref channel_name) => channel_name,
            _ => panic!(
                "Expected a channel path for object id {:?}, got {:?}",
                self.object_id, channel_path
            ),
        }
    }

    pub fn properties(&self) -> &Vec<TdmsProperty> {
        match self.file.tdms_reader.properties.get(&self.object_id) {
            Some(properties) => &properties,
            None => &EMPTY_PROPERITES,
        }
    }

    pub fn data_type(&'a self) -> TdsType {
        match self.file.tdms_reader.get_channel_data_index(self.object_id) {
            Some(channel_data_index) => channel_data_index.data_type,
            None => TdsType::Void,
        }
    }

    /// Get the total number of values in this channel
    pub fn len(&'a self) -> u64 {
        match self.file.tdms_reader.get_channel_data_index(self.object_id) {
            Some(channel_data) => channel_data.number_of_values,
            None => 0,
        }
    }

    /// Read all data for this channel into the given buffer.
    pub fn read_all_data<T: NativeType>(&'a self, buffer: &mut [T]) -> Result<()> {
        match self.file.tdms_reader.get_channel_data_index(self.object_id) {
            Some(channel_data_index) => {
                if channel_data_index.number_of_values > buffer.len() as u64 {
                    return Err(TdmsReadError::TdmsError(format!(
                        "Buffer length needs to be at least {}, received a buffer with length {}",
                        channel_data_index.number_of_values,
                        buffer.len()
                    )));
                }
                let tdms_type = channel_data_index.data_type;
                let expected_native_type = tdms_type.native_type();
                match expected_native_type {
                    Some(expected_native_type) if expected_native_type == T::native_type() => {
                        // Buffer type matches expected native type, safe to read data
                        self.file.tdms_reader.read_channel_data(
                            &mut *self.file.file_reader.borrow_mut(),
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

impl<'a, R: Read + Seek> GroupIterator<'a, R> {
    fn new(file: &'a TdmsFile<R>) -> GroupIterator<'a, R> {
        let group_objects: Vec<ObjectPathId> = file
            .tdms_reader
            .objects()
            .filter(|(_, path)| match path {
                ObjectPath::Group(_) => true,
                _ => false,
            })
            .map(|(id, _)| id)
            .collect();
        GroupIterator {
            file,
            object_iterator: group_objects.into_iter(),
        }
    }
}

impl<'a, R: Read + Seek> Iterator for GroupIterator<'a, R> {
    type Item = Group<'a, R>;

    fn next(&mut self) -> Option<Group<'a, R>> {
        self.object_iterator
            .next()
            .map(|object_id| Group::new(self.file, object_id))
    }
}

impl<'a, R: Read + Seek> ChannelIterator<'a, R> {
    fn new(file: &'a TdmsFile<R>, group_name: &str) -> ChannelIterator<'a, R> {
        let channel_objects: Vec<ObjectPathId> = file
            .tdms_reader
            .objects()
            .filter(|(_, path)| match path {
                ObjectPath::Channel(g, _) if g == group_name => true,
                _ => false,
            })
            .map(|(id, _)| id)
            .collect();
        ChannelIterator {
            file,
            object_iterator: channel_objects.into_iter(),
        }
    }
}

impl<'a, R: Read + Seek> Iterator for ChannelIterator<'a, R> {
    type Item = Channel<'a, R>;

    fn next(&mut self) -> Option<Channel<'a, R>> {
        self.object_iterator
            .next()
            .map(|object_id| Channel::new(self.file, object_id))
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

static EMPTY_PROPERITES: Vec<TdmsProperty> = Vec::new();
