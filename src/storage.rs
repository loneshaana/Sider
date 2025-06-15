use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

use crate::{
    set::{KeyExipry, KeyExistence, SetArgs},
    storage_result::StorageResult,
};

#[derive(Debug, PartialEq)]
pub enum StorageValue {
    String(String),
}

#[derive(Debug)]
pub struct StorageData {
    pub value: StorageValue,
    pub creation_time: SystemTime,
    pub expiry: Option<Duration>,
}

pub struct Storage {
    store: HashMap<String, StorageData>,
    expiry: HashMap<String, SystemTime>,
    active_expiry: bool,
}

impl StorageData {
    pub fn add_expiry(&mut self, expiry: Duration) {
        self.expiry = Some(expiry);
    }
}

impl From<String> for StorageData {
    fn from(s: String) -> StorageData {
        StorageData {
            value: StorageValue::String(s),
            creation_time: SystemTime::now(),
            expiry: None,
        }
    }
}

impl PartialEq for StorageData {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.expiry == other.expiry
    }
}

impl Storage {
    pub fn new() -> Self {
        let store: HashMap<String, StorageData> = HashMap::new();
        Self {
            store,
            expiry: HashMap::<String, SystemTime>::new(),
            active_expiry: true,
        }
    }

    pub fn set_active_expiry(&mut self, value: bool) {
        self.active_expiry = value;
    }

    pub fn expire_keys(&mut self) {
        if !self.active_expiry {
            return;
        }
        let now = SystemTime::now();

        // iterate over the keys which has expiry set
        let expired_keys: Vec<String> = self
            .expiry
            .iter()
            .filter_map(|(key, &value)| if value < now { Some(key.clone()) } else { None })
            .collect();

        // Remove the keys
        for k in expired_keys {
            self.store.remove(&k);
            self.expiry.remove(&k);
        }
    }

    pub fn set(&mut self, key: String, value: String, args: SetArgs) -> StorageResult<String> {
        let mut data = StorageData::from(value);
        let mut should_insert = true;

        let key_present = match self.store.get(&key) {
            None => false,
            _ => true,
        };

        if let Some(value) = args.existence {
            match value {
                KeyExistence::NX => {
                    // set if not exists
                    should_insert = !key_present;
                }
                KeyExistence::XX => {
                    // set if exists
                    should_insert = key_present;
                }
            }
        }

        if let Some(value) = args.expiry {
            let expiry = match value {
                KeyExipry::EX(v) => Duration::from_secs(v),
                KeyExipry::PX(v) => Duration::from_millis(v),
            };
            data.add_expiry(expiry);
            self.expiry
                .insert(key.clone(), data.creation_time.checked_add(expiry).unwrap());
        }
        if should_insert {
            self.store.insert(key, data);
            return Ok(String::from("OK"));
        }
        Ok(format!("Key is present {}", key_present))
    }

    pub fn get(&mut self, key: String) -> StorageResult<Option<String>> {
        if let Some(&expiry) = self.expiry.get(&key) {
            if SystemTime::now() >= expiry {
                self.expiry.remove(&key);
                self.store.remove(&key);
                return Ok(None);
            }
        }
        match self.store.get(&key) {
            Some(StorageData {
                value: StorageValue::String(v),
                creation_time: _,
                expiry: _,
            }) => return Ok(Some(v.to_owned())),
            None => return Ok(None),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_create_new() {
        let storage: Storage = Storage::new();
        assert_eq!(storage.store.len(), 0);
        assert_eq!(storage.expiry.len(), 0);
        assert_eq!(storage.expiry, HashMap::<String, SystemTime>::new());
        assert!(storage.active_expiry);
    }

    #[test]
    fn test_set_value() {
        let mut storage: Storage = Storage::new();
        let avalue = StorageData::from(String::from("avalue"));
        let output = storage
            .set(String::from("akey"), String::from("avalue"), SetArgs::new())
            .unwrap();
        assert_eq!(output, String::from("OK"));
        assert_eq!(storage.store.len(), 1);
        match storage.store.get(&String::from("akey")) {
            Some(value) => assert_eq!(value, &avalue),
            None => panic!(),
        }
    }
    #[test]
    fn test_get_value() {
        let mut storage: Storage = Storage::new();
        storage.store.insert(
            String::from("akey"),
            StorageData::from(String::from("avalue")),
        );
        let result = storage.get(String::from("akey")).unwrap();
        assert_eq!(storage.store.len(), 1);
        assert_eq!(result, Some(String::from("avalue")));
    }
    #[test]
    fn test_get_value_key_does_not_exist() {
        let mut storage: Storage = Storage::new();
        let result = storage.get(String::from("akey")).unwrap();
        assert_eq!(storage.store.len(), 0);
        assert_eq!(result, None);
    }

    #[test]
    fn test_expire_keys() {
        let mut storage: Storage = Storage::new();
        storage
            .set(String::from("akey"), String::from("avalue"), SetArgs::new())
            .unwrap();
        storage.expiry.insert(
            String::from("akey"),
            SystemTime::now() - Duration::from_secs(5),
        );
        storage.expire_keys();
        assert_eq!(storage.store.len(), 0);
    }
    #[test]
    fn test_expire_keys_deactivated() {
        let mut storage: Storage = Storage::new();
        storage.set_active_expiry(false);
        storage
            .set(String::from("akey"), String::from("avalue"), SetArgs::new())
            .unwrap();
        storage.expiry.insert(
            String::from("akey"),
            SystemTime::now() - Duration::from_secs(5),
        );
        storage.expire_keys();
        assert_eq!(storage.store.len(), 1);
    }
}
