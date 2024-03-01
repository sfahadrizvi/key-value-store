use super::server::{Key, KeyValue, ServerState};
use axum::{extract::State, http::StatusCode};
use futures::future;
use serde_json::Value;
use std::sync::Arc;

///Request to get key value.
pub(crate) async fn get_values(
    State(state): State<Arc<ServerState>>,
    body: String,
) -> Result<String, StatusCode> {
    debug!("get_value called");
    let mut keys_found = Vec::new();
    let mut keys_not_found = Vec::new();
    let json_body_res: Result<Value, _> = serde_json::from_str(&body);
    if json_body_res.is_err() {
        return Err(StatusCode::BAD_REQUEST);
    } 
    let json_body = json_body_res.unwrap();
    
    // Parse the body to create an array of keys to update/create
    let json_keys_res: Result<Vec<Key>, _> = if json_body.is_array() {    
        serde_json::from_value(json_body)
    } else {            
        serde_json::from_value(json_body).map_or_else(|e| Err(e),|val| Ok(vec![val]))
    };
    if json_keys_res.is_err() {
        return Err(StatusCode::BAD_REQUEST);
    } 
    let json_keys = json_keys_res.unwrap();

    // If there are no repreated keys then create a async task for each key
    let create_tasks: Vec<_> = json_keys
        .iter()
        .map(|key| tokio::spawn(get_key(key.key.clone(), State(state.clone()))))
        .collect();

    // Execute tasks async and loop through the results to create the final resultant array
    let task_results: Vec<Result<Result<KeyValue, StatusCode>, tokio::task::JoinError>> =
        future::join_all(create_tasks).await;
    for (index, val) in task_results.into_iter().enumerate() {
        if let Ok(Ok(key_value)) = val {
            keys_found.push(key_value);
        } else {
            keys_not_found.push(KeyValue {
                key: json_keys[index].key.to_owned(),
                value: "".to_string(),
            });
        }
    }

    let mut key_values = keys_found
        .iter()
        .map(|val| {
            format!(
                r#"{{"key":"{}", "value":"{}" "found": "true"}}"#,
                val.key, val.value
            )
        })
        .collect::<Vec<_>>();
    key_values.append(
        &mut keys_not_found
            .iter()
            .map(|val| format!(r#"{{"key":"{}", "value":"", "found": "false"}}"#, val.key))
            .collect::<Vec<_>>(),
    );

    Ok(format!("[{}]", key_values.join(",")))
}

async fn get_key(
    key: String,
    State(state): State<Arc<ServerState>>,
) -> Result<KeyValue, StatusCode> {
    if let Some(cache_value) = state.cache.get(&key).await {
        info!("Getting {} from cache", key);
        Ok(KeyValue {
            key,
            value: cache_value,
        })
    } else {
        warn!("Getting {} from disk", key);
        let kv_ret = state.file_gateway.read_file(key).await;
        match kv_ret {
            Ok(kv) => {
                state.cache.insert(kv.key.clone(), kv.value.clone()).await;
                Ok(kv)
            }
            Err(err) => Err(err),
        }
    }
}
