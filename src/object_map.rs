use crate::object_path::ObjectPathId;

/// A map from object path id to values of type T, using a vector
pub struct ObjectMap<T: Copy> {
    values: Vec<Option<T>>,
}

impl<T: Copy> ObjectMap<T> {
    pub fn new() -> ObjectMap<T> {
        ObjectMap {
            values: Vec::new(),
        }
    }

    /// Set a new value or overwrite an existing value
    pub fn set(&mut self, object: ObjectPathId, value: T) {
        let index = object.as_usize();
        if index >= self.values.len() {
            let padding_length = index - self.values.len();
            self.values.reserve(1 + padding_length);
            for _ in 0..padding_length {
                self.values.push(None);
            }
            self.values.push(Some(value));
        } else {
            self.values[index] = Some(value);
        }
    }

    /// Get the value associated with an object if set
    pub fn get(&self, object: ObjectPathId) -> Option<T> {
        match self.values.get(object.as_usize()) {
            Some(option) => *option,
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
use crate::object_path::ObjectPathCache;
    use super::*;

    #[test]
    fn set_and_get() {
        let mut path_cache = ObjectPathCache::new();
        let root_obj = path_cache.get_or_create_id(String::from("/"));
        let group_obj = path_cache.get_or_create_id(String::from("/'group'"));
        let channel_1 = path_cache.get_or_create_id(String::from("/'group'/'channel_1'"));
        let channel_2 = path_cache.get_or_create_id(String::from("/'group'/'channel_2'"));
        let channel_3 = path_cache.get_or_create_id(String::from("/'group'/'channel_3'"));

        let mut object_map = ObjectMap::new();
        object_map.set(group_obj, 1);
        object_map.set(root_obj, 0);
        object_map.set(channel_2, 3);

        assert_eq!(object_map.get(root_obj), Some(0));
        assert_eq!(object_map.get(group_obj), Some(1));
        assert_eq!(object_map.get(channel_1), None);
        assert_eq!(object_map.get(channel_2), Some(3));
        assert_eq!(object_map.get(channel_3), None);
    }

    #[test]
    fn overwrite() {
        let mut path_cache = ObjectPathCache::new();
        let channel_1 = path_cache.get_or_create_id(String::from("/'group'/'channel_1'"));

        let mut object_map = ObjectMap::new();
        object_map.set(channel_1, 2);
        object_map.set(channel_1, 3);

        assert_eq!(object_map.get(channel_1), Some(3));
    }
}
