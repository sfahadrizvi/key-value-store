use super::server::{Key, KeyValue, ServerState};
use crate::file_system::operations::read_file;
use axum::{extract::State, http::StatusCode};
use serde_json::Value;
use std::sync::Arc;
use futures::future;

////Request to get key value.
pub(crate) async fn get_values(State(state): State<Arc<ServerState>>, body:String) -> Result<String, StatusCode> {
    debug!("get_value called");
    let mut keys_found = Vec::new();
    let mut keys_not_found = Vec::new();
    let json_body: Value = serde_json::from_str(&body).unwrap();

    if json_body.is_array() {
        let json_keys: Vec<Key> = serde_json::from_value(json_body).unwrap();
        let create_tasks: Vec<_>  = json_keys
                                        .iter()
                                        .map(|key| tokio::spawn(
                                        get_key(key.key.clone(), State(state.clone()))
                                        )
                                    )
                                    .collect();
        let task_results: Vec<Result<Result<KeyValue, StatusCode>, tokio::task::JoinError>> =  future::join_all(create_tasks).await;
        for (index, val) in task_results.into_iter().enumerate() {
            if let Ok(key_value_res) = val {
                if let Ok(key_value) = key_value_res {
                    keys_found.push(key_value);
                } else {
                    keys_not_found.push(KeyValue{key: json_keys[index].key.to_owned(), value: "".to_string()});
                }
            } else {
                keys_not_found.push(KeyValue{key: json_keys[index].key.to_owned(), value: "".to_string()});
            }
        }

    } else {
        let json_key: Result<Key, serde_json::Error> = serde_json::from_value(json_body);
        if let Ok(key_value) = json_key  {
            let clone_key = key_value.clone();
            let task_result = tokio::spawn(async move {    
                get_key(clone_key.key, State(state.clone())).await
            }).await;
            if let Ok(key_value_res) = task_result {
                if let Ok(key_value) = key_value_res {
                    keys_found.push(key_value);
                } else {
                    keys_not_found.push(KeyValue{key: key_value.key, value: "".to_string()});
                }
            } else {
                keys_not_found.push(KeyValue{key: key_value.key, value: "".to_string()});
            }
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

async fn get_key(key: String, State(state):State<Arc<ServerState>>) -> Result<KeyValue, StatusCode> {
    if let Some(cache_value) = state.cache.get(&key).await {
        info!("Getting {} from cache", key);
        return Ok(KeyValue{key, value: cache_value});
    } else {
        warn!("Getting {} from disk", key);
        read_file(state.path.clone(), key).await
    }
}
