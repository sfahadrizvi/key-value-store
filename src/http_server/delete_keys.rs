use super::server::{Key, ServerState};
use axum::{extract::State, http::StatusCode};
use futures::future;
use serde_json::Value;
use std::sync::Arc;

///Request to delete keys. It will still suceed if the key does not exist
pub(crate) async fn delete_keys(
    State(state): State<Arc<ServerState>>,
    body: String,
) -> Result<String, StatusCode> {
    info!("delete_key called with keys {}", body);
    let mut keys_deleted = Vec::new();
    let mut failed_deletion = Vec::new();
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

    let create_tasks: Vec<_> = json_keys
        .iter()
        .map(|key| tokio::spawn(delete_key(key.key.clone(), State(state.clone()))))
        .collect();
    let task_results: Vec<Result<Result<String, StatusCode>, tokio::task::JoinError>> =
        future::join_all(create_tasks).await;
    for (index, val) in task_results.into_iter().enumerate() {
        if let Ok(Ok(key_value)) = val {
            keys_deleted.push(key_value);
        } else {
            failed_deletion.push(json_keys[index].key.to_owned());
        }
    }

    let mut deletions = keys_deleted
        .iter()
        .map(|val| format!(r#"{{"key":{}, "deleted": "true"}}"#, val))
        .collect::<Vec<_>>();
    deletions.append(
        &mut failed_deletion
            .iter()
            .map(|val| format!(r#"{{"key":{}, "deleted": "false"}}"#, val))
            .collect::<Vec<_>>(),
    );

    Ok(format!("[{}]", deletions.join(",")))
}

async fn delete_key(
    key: String,
    State(state): State<Arc<ServerState>>,
) -> Result<String, StatusCode> {
    state.cache.invalidate(&key).await;
    state.file_gateway.delete_file(key).await
}
