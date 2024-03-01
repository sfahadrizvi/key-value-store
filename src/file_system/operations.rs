use crate::http_server::server::{Key, KeyValue};
use axum::http::StatusCode;
use glob::glob;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub(crate) struct FileGateway {
    store_location: String,
    locks: Arc<Vec<RwLock<i32>>>,
}

impl FileGateway {
    pub(crate) fn new(store_location: String) -> Self {
        let rwlocks: Vec<_> = std::iter::repeat_with(|| RwLock::new(0))
            .take(NUMBER_OF_LOCKS)
            .collect();
        let locks = Arc::new(rwlocks);
        Self {
            store_location,
            locks,
        }
    }

    pub async fn write_file(
        &self,
        key: String,
        data: String,
        create_new: bool,
    ) -> Result<(), String> {
        let lock = self.write_lock_key(&key);
        if lock.is_err() {
            return Err("Error".to_string());
        }
        debug!("Got write lock for file {}", key);

        //Uncomment this delay to simulate a very very very large file write delay
        //std::thread::sleep(std::time::Duration::from_secs(10));

        let path = format!("{}/{}", self.store_location, key);
        info!("Creating key file {}", path);
        let file = OpenOptions::new()
            .create_new(create_new)
            .create(true)
            .write(true)
            .truncate(true)
            .open(path);

        match file {
            Ok(mut file) => {
                let ret = file.write_all(data.as_bytes());
                match ret {
                    Ok(()) => Ok(()),
                    Err(err) => {
                        warn!("Failed to create key file with error {}", err);
                        Err(format!("Failed to create key file with error {}", err))
                    }
                }
            }
            Err(err) => {
                warn!("Failed to create key file open options with error {}", err);
                Err(format!(
                    "Failed to create key file open options with error {}",
                    err
                ))
            }
        }
    }

    pub(crate) async fn read_file(&self, key: String) -> Result<KeyValue, StatusCode> {
        let lock = self.read_lock_key(&key);
        if lock.is_err() {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        debug!("Got read lock for file {}", key);

        //Uncomment this delay to simulate a very very very large file read delay
        //std::thread::sleep(std::time::Duration::from_secs(7));

        let path = format!("{}/{}", self.store_location, key);
        if !std::path::Path::new(&path).exists() {
            info!("Finding nonexisting key {}", key);
            Err(StatusCode::NOT_FOUND)
        } else {
            match fs::read_to_string(path) {
                Ok(value) => Ok(KeyValue { key, value }),
                Err(err) => {
                    warn!("Key could not be read: {}", err);
                    Err(StatusCode::NOT_FOUND)
                }
            }
        }
    }

    pub(crate) async fn delete_file(&self, key: String) -> Result<String, StatusCode> {
        let lock = self.write_lock_key(&key);
        if lock.is_err() {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        debug!("Got write lock for file {}", key);

        let path = format!("{}/{}", self.store_location, key);
        if !std::path::Path::new(&path).exists() {
            info!("deleteting nonexisting key {}", key);
            Ok(key)
        } else {
            match fs::remove_file(path) {
                Ok(()) => Ok(key),
                Err(err) => {
                    warn!("Key could not be delted: {}", err);
                    Err(StatusCode::NOT_MODIFIED)
                }
            }
        }
    }

    pub(crate) async fn get_files_with_prefix(
        &self,
        prefix: String,
    ) -> Result<Vec<Key>, StatusCode> {
        let path = format!("{}/{}*", self.store_location, prefix);
        debug!("get_files_with_prefix called with path: {}", path);
        let mut keys_found: Vec<Key> = Vec::new();
        for entry in glob(&path).expect("Failed to read glob pattern") {
            match entry {
                Ok(key_path) => {
                    info!("found file {:?}", key_path.display());
                    let file_name = Path::new(&key_path)
                        .file_name()
                        .expect("Faield to get file name form path")
                        .to_string_lossy()
                        .to_string();

                    keys_found.push(Key { key: file_name });
                }
                Err(e) => warn!("{:?}", e),
            }
        }
        Ok(keys_found)
    }

    pub fn read_lock_key(
        &self,
        key: &str,
    ) -> Result<RwLockReadGuard<'_, i32>, PoisonError<RwLockReadGuard<'_, i32>>> {
        // Locks are in a bucket and each key locks a RwLock from that bucket.
        // For simplicity the lock bucket is based on the length of the key
        // other methods like hasing can also be used.
        let index = key.len() % NUMBER_OF_LOCKS;
        self.locks[index].read()
    }

    pub fn write_lock_key(
        &self,
        key: &str,
    ) -> Result<RwLockWriteGuard<'_, i32>, PoisonError<RwLockWriteGuard<'_, i32>>> {
        // Locks are in a bucket and each key locks a RwLock from that bucket.
        // For simplicity the lock bucket is based on the length of the key
        // other methods like hasing can also be used.
        let index = key.len() % NUMBER_OF_LOCKS;
        self.locks[index].write()
    }
}

static NUMBER_OF_LOCKS: usize = 10;
