use super::server::Key;
use axum::{extract::State, http::StatusCode};
use serde_json::Value;
use std::fs;

////Request to delete a key. It will still suceed if the key does not exist
pub(crate) async fn delete_key(State(state): State<String>, body:String) -> Result<String, StatusCode> {
    info!("delete_key called with keys {}", body);
    let mut keys_deleted = Vec::new();
    let mut failed_deletion = Vec::new();
    let r: Value = serde_json::from_str(&body).unwrap();

    if r.is_array() {
        let arr: Vec<Key> = serde_json::from_value(r).unwrap();
        for key in arr {
            let key = delete_single_key(State(state.clone()), key.key).await;
            if key.is_ok() {
                keys_deleted.push(key.unwrap());
            } else {
                failed_deletion.push(key.unwrap_err());
            }
        }
    } else {
        let key: Key = serde_json::from_value(r).unwrap();
        let res = delete_single_key(State(state.clone()), key.key).await;
        if res.is_ok() {
            keys_deleted.push(res.unwrap());
        } else {
            failed_deletion.push(res.unwrap_err());
        }
    }
    
    let mut deletions = keys_deleted
    .iter()
    .map(|val| format!(r#"{{"key":{}, "deleted": "true"}}"#, val)).collect::<Vec<_>>();
    deletions.append(
        &mut failed_deletion
            .iter()
            .map(|val| format!(r#"{{"key":{}, "deleted": "false"}}"#, val)).collect::<Vec<_>>()
        );

    let response_string = deletions.join(",");
    Ok(response_string)
    
}

async fn delete_single_key(State(state): State<String>, key:String) -> Result<String, StatusCode> {
    let path = format!("{}/{}", state, key);
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
