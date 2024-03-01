use super::server::{check_for_repeated_key, KeyValue, ServerState};
use axum::{extract::State, http::StatusCode};
use futures::future;
use serde_json::Value;
use std::sync::Arc;

///Request to create a key. It will fail if the key already exists
pub(crate) async fn create_key(
    State(state): State<Arc<ServerState>>,
    body: String,
) -> Result<String, StatusCode> {
    info!("create_key api called");
    create_or_update_keys(State(state), body, true).await
}

///Request to update or create a key. It will create a key if it does not exist
pub(crate) async fn update_key(
    State(state): State<Arc<ServerState>>,
    body: String,
) -> Result<String, StatusCode> {
    info!("update_key api called");
    create_or_update_keys(State(state), body, false).await
}

///This funtion creates checks if the request body if json or json array and creates/updates each key
async fn create_or_update_keys(
    State(state): State<Arc<ServerState>>,
    body: String,
    create_new: bool,
) -> Result<String, StatusCode> {
    debug!("create_or_update_key called");
    let mut keys_modified: Vec<String> = Vec::new();
    let mut failed_modification: Vec<String> = vec![];
    let json_body_res: Result<Value, _> = serde_json::from_str(&body);
    if json_body_res.is_err() {
        return Err(StatusCode::BAD_REQUEST);
    } 
    let json_body = json_body_res.unwrap();
    
    // Parse the body to create an array of keys to update/create
    let json_key_values_res: Result<Vec<KeyValue>, _> = if json_body.is_array() {    
        serde_json::from_value(json_body)
    } else {            
        serde_json::from_value(json_body).map_or_else(|e| Err(e),|val| Ok(vec![val]))
    };
    if json_key_values_res.is_err() {
        return Err(StatusCode::BAD_REQUEST);
    } 
    let json_key_values = json_key_values_res.unwrap();

    if check_for_repeated_key(&json_key_values) {
        // If there are no repreated keys then create a async task for each key
        let create_tasks: Vec<_> = json_key_values
            .iter()
            .map(|val| {
                tokio::spawn(create_or_update_key(
                    State(state.clone()),
                    val.clone(),
                    create_new,
                ))
            })
            .collect();

        // Execute tasks async and loop through the results to create the final resultant array
        let task_results: Vec<Result<Result<String, String>, tokio::task::JoinError>> =
            future::join_all(create_tasks).await;
        for (index, val) in task_results.into_iter().enumerate() {
            if extract_failure_success(val) {
                keys_modified.push(json_key_values[index].key.clone());
            } else {
                failed_modification.push(json_key_values[index].key.clone());
            }
        }
    } else {
        warn!("create_or_update_key called with bad request, duplicate keuys");
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(create_json_response(keys_modified, failed_modification))
}

fn extract_failure_success(val: Result<Result<String, String>, tokio::task::JoinError>) -> bool {
    if let Ok(key_value) = val {
        key_value.is_ok()
    } else {
        false
    }
}

async fn create_or_update_key(
    State(state): State<Arc<ServerState>>,
    kv: KeyValue,
    create_new: bool,
) -> Result<String, String> {
    info!(
        "{} key {} value {}",
        if create_new { "Create" } else { "Update" },
        kv.key,
        kv.value
    );

    if kv.key.is_empty() || kv.value.is_empty() || kv.key.contains('/') {
        warn!("Bad request, invalid key {} or value {}", kv.key, kv.value);
        return Err(format!(
            "Bad request, invalid key {} or value {}",
            kv.key, kv.value
        ));
    }

    let write_res = state
        .file_gateway
        .write_file(kv.key.clone(), kv.value.clone(), create_new)
        .await;
    match write_res {
        Ok(_) => {
            debug!(
                "storing in cache from write {},{}",
                kv.key,
                kv.value.clone()
            );
            state.cache.insert(kv.key.clone(), kv.value.clone()).await;
            Ok(kv.key)
        }
        Err(err) => Err(err),
    }
}

fn create_json_response(modified: Vec<String>, failed: Vec<String>) -> String {
    let mut response = modified
        .iter()
        .map(|val| format!(r#"{{"key":"{}", "modified": "true"}}"#, val))
        .collect::<Vec<_>>();
    response.append(
        &mut failed
            .iter()
            .map(|val| format!(r#"{{"key":"{}", "modified": "false"}}"#, val))
            .collect::<Vec<_>>(),
    );

    format!("[{}]", response.join(","))
}
