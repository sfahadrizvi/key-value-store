use super::server::Key as MatchedKeys;
use axum::{extract::State, http::StatusCode};
use serde_json::Value;

use glob::glob;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Debug, Clone)]
struct Key {
    prefix: String
}

////Request to get key value.
pub(crate) async fn get_keys(State(state): State<String>, body:String) -> Result<String, StatusCode> {
    info!("get_keys api called with keys {}", body);
    let mut keys_found = Vec::new();
    let r: Value = serde_json::from_str(&body).unwrap();
    
    if r.is_array() {
        let arr: Vec<Key> = serde_json::from_value(r).unwrap();
        for key in arr {
            keys_found.append(&mut get_key(State(state.clone()), key.prefix.to_owned()).await);
        }
    } else {
        let key: Key = serde_json::from_value(r).unwrap();
        keys_found.append(&mut get_key(State(state.clone()), key.prefix.to_owned()).await);
    }
    

    let response_string = serde_json::to_string::<Vec<MatchedKeys>>(&keys_found).unwrap();
    Ok(response_string)
}

async fn get_key(State(state): State<String>, key:String) -> Vec<MatchedKeys> {
    let path = format!("{}/{}*", state, key);
    let mut keys_found: Vec<MatchedKeys> = Vec::new();
    for entry in glob(&path).expect("Failed to read glob pattern") {
        match entry {
            Ok(key_path) => {
                info!("found file {:?}", key_path.display());
                let file_name = Path::new(&key_path)
                    .file_name()
                    .expect("Faield to get file name form path")
                    .to_string_lossy()
                    .to_string();
                
                keys_found.push(MatchedKeys {key:file_name});
        },
            Err(e) => warn!("{:?}", e),
        }
    }
    keys_found
}
