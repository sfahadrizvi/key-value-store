use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct KeyValue {
    pub key: String,
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct Key {
    pub key: String
}
