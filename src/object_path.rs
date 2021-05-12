use crate::error::{Result, TdmsReadError};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub fn path_from_group(group_name: &str) -> String {
    format!("/'{}'", group_name.replace("'", "''"))
}

pub fn path_from_channel(group_name: &str, channel_name: &str) -> String {
    format!(
        "/'{}'/'{}'",
        group_name.replace("'", "''"),
        channel_name.replace("'", "''")
    )
}

#[derive(Debug, PartialEq, Eq)]
pub enum ObjectPath {
    Root,
    Group(String),
    Channel(String, String),
}

enum PathParserState {
    /// We're expecting the beginning of a new componet
    ComponentStart,

    /// We're within a component, remembering the start position
    InComponent(usize),
}

impl ObjectPath {
    /// Parse a TDMS object path
    pub fn parse(input_string: &str) -> Result<ObjectPath> {
        let mut components: Vec<&str> = Vec::new();
        let mut char_iterator = input_string.char_indices();

        let mut parser_state = PathParserState::ComponentStart;

        // Iterate over characters, always peeking forward one extra character
        // so we can check for escaped quotes ("''" becomes "'").
        let mut current_char = char_iterator.next();
        let mut next_char = char_iterator.next();
        loop {
            match parser_state {
                PathParserState::ComponentStart => {
                    match (current_char, next_char) {
                        (None, _) => {
                            // End of the path
                            break;
                        }
                        (Some((_, '/')), None) => {
                            // Root object
                            break;
                        }
                        (Some((start_index, '/')), Some((_, '\''))) => {
                            next_char = char_iterator.next();
                            parser_state = PathParserState::InComponent(start_index + 2);
                        }
                        _ => {
                            return Err(TdmsReadError::TdmsError(format!(
                                "Invalid object path {}",
                                input_string
                            )))
                        }
                    }
                }
                PathParserState::InComponent(start_index) => {
                    match (current_char, next_char) {
                        (Some((_, '\'')), Some((_, '\''))) => {
                            // Escaped quote, skip over it
                            next_char = char_iterator.next();
                        }
                        (Some((end_index, '\'')), _) => {
                            // At end of path
                            components.push(&input_string[start_index..end_index]);
                            parser_state = PathParserState::ComponentStart;
                        }
                        (Some((_, _)), _) => {
                            // Normal character in component, continue
                        }
                        (None, _) => {
                            // Unexpected end of path
                            return Err(TdmsReadError::TdmsError(format!(
                                "Invalid object path {}",
                                input_string
                            )));
                        }
                    }
                }
            };
            current_char = next_char;
            next_char = char_iterator.next();
        }

        return match components.as_slice() {
            [] => Ok(ObjectPath::Root),
            [group_name] => Ok(ObjectPath::Group(group_name.replace("''", "'"))),
            [group_name, channel_name] => Ok(ObjectPath::Channel(
                group_name.replace("''", "'"),
                channel_name.replace("''", "'"),
            )),
            _ => Err(TdmsReadError::TdmsError(format!(
                "Invalid object path '{}' with more than 2 components",
                input_string
            ))),
        };
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ObjectPathId(u32);

impl ObjectPathId {
    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug)]
pub struct ObjectPathCache {
    path_to_id: HashMap<String, ObjectPathId>,
    next_id: u32,
}

impl ObjectPathCache {
    pub fn new() -> ObjectPathCache {
        ObjectPathCache {
            path_to_id: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn get_id(&self, object_path: &str) -> Option<ObjectPathId> {
        match self.path_to_id.get(object_path) {
            Some(&obj_id) => Some(obj_id),
            None => None,
        }
    }

    pub fn get_or_create_id(&mut self, object_path: String) -> ObjectPathId {
        match self.path_to_id.entry(object_path) {
            Entry::Occupied(occupied_entry) => *occupied_entry.get(),
            Entry::Vacant(vacant_entry) => {
                let new_id = ObjectPathId(self.next_id);
                self.next_id += 1;
                vacant_entry.insert(new_id);
                new_id
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_and_retrieve() {
        let mut object_path_cache = ObjectPathCache::new();
        let object_id = object_path_cache.get_or_create_id("/".to_string());

        let found_id = object_path_cache.get_id("/");

        assert_eq!(found_id, Some(object_id));
    }

    #[test]
    fn different_ids() {
        let mut object_path_cache = ObjectPathCache::new();

        let object_id_0 = object_path_cache.get_or_create_id("/".to_string());
        let object_id_1 = object_path_cache.get_or_create_id("/'group'".to_string());

        assert_ne!(object_id_0, object_id_1);
    }

    #[test]
    fn not_found() {
        let object_path_cache = ObjectPathCache::new();

        let missing = object_path_cache.get_id("/");

        assert_eq!(missing, None);
    }

    #[test]
    fn parse_root_path() {
        let path_string = "/";

        let path = ObjectPath::parse(path_string);

        assert_eq!(path.unwrap(), ObjectPath::Root);
    }

    #[test]
    fn parse_group_path() {
        let path_string = "/'GroupName'";

        let path = ObjectPath::parse(path_string);

        assert_eq!(path.unwrap(), ObjectPath::Group("GroupName".to_string()));
    }

    #[test]
    fn parse_channel_path() {
        let path_string = "/'GroupName'/'ChannelName'";

        let path = ObjectPath::parse(path_string);

        assert_eq!(
            path.unwrap(),
            ObjectPath::Channel("GroupName".to_string(), "ChannelName".to_string())
        );
    }

    #[test]
    fn parse_channel_path_test_cases() {
        let test_cases = vec![
            (
                "/'Group''Name'/'Channel''Name'",
                "Group'Name",
                "Channel'Name",
            ),
            (
                "/'''GroupName'''/'''ChannelName'''",
                "'GroupName'",
                "'ChannelName'",
            ),
            ("/''''''/''''''", "''", "''"),
            ("/''''''/''''''", "''", "''"),
            ("/'Group/Name'/'Channel/Name'", "Group/Name", "Channel/Name"),
        ];

        for (path_string, expected_group, expected_channel) in test_cases {
            let path = ObjectPath::parse(path_string);
            match path {
                Ok(ObjectPath::Channel(ref group_name, ref channel_name)) => {
                    assert_eq!(group_name, expected_group);
                    assert_eq!(channel_name, expected_channel);
                }
                _ => panic!("Expected a valid channel for path {}", path_string),
            }
        }
    }
}
