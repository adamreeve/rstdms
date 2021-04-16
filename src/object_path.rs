use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ObjectPathId(u32);

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
}
