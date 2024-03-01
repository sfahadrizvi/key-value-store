use super::server::{KeyValue, Key};
use axum::{extract::State, http::StatusCode};
use serde_json::Value;
use std::fs;

////Request to get key value.
pub(crate) async fn get_value(State(state): State<String>, body:String) -> Result<String, StatusCode> {
    info!("get_value api called with keys {}", body);
    let mut keys_found = Vec::new();
    let mut keys_not_found = Vec::new();
    let r: Value = serde_json::from_str(&body).unwrap();

    if r.is_array() {
        let arr: Vec<Key> = serde_json::from_value(r).unwrap();
        for key in arr {
            let keyvalue = get_key(State(state.clone()), key.key.to_owned()).await;
            if keyvalue.is_ok() {
                keys_found.push(keyvalue.unwrap());
            } else {
                keys_not_found.push(KeyValue{key: key.key, value: "".to_string()});
            }
        }
    } else {
        let key: Key = serde_json::from_value(r).unwrap();
        let res = get_key(State(state.clone()), key.key.to_owned()).await;
        if res.is_ok() {
            keys_found.push(res.unwrap());
        } else {
            keys_not_found.push(KeyValue{key: key.key, value: "".to_string()});
        }
    }
    
    let mut key_values = keys_found
        .iter()
        .map(|val| format!(r#"{{"key":"{}", "value":"{}" "found": "true"}}"#, val.key, val.value)).collect::<Vec<_>>();
    key_values.append(
        &mut keys_not_found
            .iter()
            .map(|val| format!(r#"{{"key":"{}", "value":"", "found": "false"}}"#, val.key)).collect::<Vec<_>>()
        );

    let response_string = key_values.join(",");
    Ok(response_string)
}

async fn get_key(State(state): State<String>, key:String) -> Result<KeyValue, StatusCode> {
    let path = format!("{}/{}", state, key);
    if !std::path::Path::new(&path).exists() {
        info!("Finding nonexisting key {}", key);
        Err(StatusCode::NOT_FOUND)
    } else {
        match fs::read_to_string(path) {
            Ok(value) => Ok(
                KeyValue {key, value }),
            Err(err) => { 
                warn!("Key could not be read: {}", err);
                Err(StatusCode::NOT_FOUND)
            }
        }
    }
}
