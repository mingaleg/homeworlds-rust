use super::common::UpdateOneDelta;
use std::{collections::hash_map::Entry, hash::Hash, num::NonZero};

pub fn update_hashmap_count<K, E>(
    entry: Entry<K, NonZero<u8>>,
    delta: UpdateOneDelta,
    overflow_error: E,
    not_found_error: E,
) -> Result<(), E>
where
    K: Eq + Hash,
{
    match delta {
        UpdateOneDelta::AddOne => match entry {
            Entry::Occupied(mut e) => {
                let count = e.get_mut();
                *count = count.checked_add(1).ok_or(overflow_error)?;
            }
            Entry::Vacant(e) => {
                e.insert(NonZero::new(1).unwrap());
            }
        },
        UpdateOneDelta::RemoveOne => match entry {
            Entry::Occupied(mut entry) => {
                let count = entry.get_mut();
                if count.get() > 1 {
                    *count = unsafe { NonZero::new_unchecked(count.get() - 1) };
                } else {
                    entry.remove();
                }
            }
            Entry::Vacant(_) => return Err(not_found_error),
        },
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Debug, PartialEq)]
    enum TestError {
        Overflow,
        NotFound,
    }

    #[test]
    fn test_add_one_to_vacant_entry() {
        let mut map: HashMap<String, NonZero<u8>> = HashMap::new();
        let entry = map.entry("key".to_string());

        let result = update_hashmap_count(
            entry,
            UpdateOneDelta::AddOne,
            TestError::Overflow,
            TestError::NotFound,
        );

        assert!(result.is_ok());
        assert_eq!(map.get("key").unwrap().get(), 1);
    }

    #[test]
    fn test_add_one_to_occupied_entry() {
        let mut map: HashMap<String, NonZero<u8>> = HashMap::new();
        map.insert("key".to_string(), NonZero::new(5).unwrap());
        let entry = map.entry("key".to_string());

        let result = update_hashmap_count(
            entry,
            UpdateOneDelta::AddOne,
            TestError::Overflow,
            TestError::NotFound,
        );

        assert!(result.is_ok());
        assert_eq!(map.get("key").unwrap().get(), 6);
    }

    #[test]
    fn test_add_one_overflow() {
        let mut map: HashMap<String, NonZero<u8>> = HashMap::new();
        map.insert("key".to_string(), NonZero::new(255).unwrap());
        let entry = map.entry("key".to_string());

        let result = update_hashmap_count(
            entry,
            UpdateOneDelta::AddOne,
            TestError::Overflow,
            TestError::NotFound,
        );

        assert_eq!(result, Err(TestError::Overflow));
        assert_eq!(map.get("key").unwrap().get(), 255);
    }

    #[test]
    fn test_remove_one_decrement() {
        let mut map: HashMap<String, NonZero<u8>> = HashMap::new();
        map.insert("key".to_string(), NonZero::new(5).unwrap());
        let entry = map.entry("key".to_string());

        let result = update_hashmap_count(
            entry,
            UpdateOneDelta::RemoveOne,
            TestError::Overflow,
            TestError::NotFound,
        );

        assert!(result.is_ok());
        assert_eq!(map.get("key").unwrap().get(), 4);
    }

    #[test]
    fn test_remove_one_removes_entry_at_one() {
        let mut map: HashMap<String, NonZero<u8>> = HashMap::new();
        map.insert("key".to_string(), NonZero::new(1).unwrap());
        let entry = map.entry("key".to_string());

        let result = update_hashmap_count(
            entry,
            UpdateOneDelta::RemoveOne,
            TestError::Overflow,
            TestError::NotFound,
        );

        assert!(result.is_ok());
        assert!(!map.contains_key("key"));
    }

    #[test]
    fn test_remove_one_from_vacant_entry() {
        let mut map: HashMap<String, NonZero<u8>> = HashMap::new();
        let entry = map.entry("key".to_string());

        let result = update_hashmap_count(
            entry,
            UpdateOneDelta::RemoveOne,
            TestError::Overflow,
            TestError::NotFound,
        );

        assert_eq!(result, Err(TestError::NotFound));
        assert!(!map.contains_key("key"));
    }

    #[test]
    fn test_add_one_multiple_times() {
        let mut map: HashMap<String, NonZero<u8>> = HashMap::new();

        for _ in 0..10 {
            let entry = map.entry("key".to_string());
            let result = update_hashmap_count(
                entry,
                UpdateOneDelta::AddOne,
                TestError::Overflow,
                TestError::NotFound,
            );
            assert!(result.is_ok());
        }

        assert_eq!(map.get("key").unwrap().get(), 10);
    }

    #[test]
    fn test_remove_one_multiple_times() {
        let mut map: HashMap<String, NonZero<u8>> = HashMap::new();
        map.insert("key".to_string(), NonZero::new(10).unwrap());

        for i in (1..=10).rev() {
            let entry = map.entry("key".to_string());
            let result = update_hashmap_count(
                entry,
                UpdateOneDelta::RemoveOne,
                TestError::Overflow,
                TestError::NotFound,
            );
            assert!(result.is_ok());

            if i > 1 {
                assert_eq!(map.get("key").unwrap().get(), i - 1);
            } else {
                assert!(!map.contains_key("key"));
            }
        }
    }
}
