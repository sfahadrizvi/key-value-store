use std::collections::HashSet;

use moka::future::Cache;
use serde::{Deserialize, Serialize};

use crate::file_system::operations::FileGateway;

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct KeyValue {
    pub key: String,
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct Key {
    pub key: String,
}

pub(crate) struct ServerState {
    pub cache: Cache<String, String>,
    pub file_gateway: FileGateway,
}

pub(crate) fn check_for_repeated_key(kv_vec: &[KeyValue]) -> bool {
    let keys = kv_vec
        .iter()
        .map(|key| key.key.clone())
        .collect::<HashSet<String>>();

    if keys.contains("") || kv_vec.len() != keys.len() {
        warn!("Empty or repeated keys");
        false
    } else {
        true
    }
}
