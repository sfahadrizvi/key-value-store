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
    let json_body: Value = serde_json::from_str(&body).unwrap();

    if json_body.is_array() {
        let json_keys: Vec<Key> = serde_json::from_value(json_body).unwrap();
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
    } else {
        let json_key: Result<Key, serde_json::Error> = serde_json::from_value(json_body);
        if let Ok(key_value) = json_key {
            let clone_key = key_value.clone();
            let task_result = tokio::spawn(async move {
                delete_key(clone_key.key.clone(), State(state.clone())).await
            })
            .await;
            if let Ok(Ok(key_value)) = task_result {
                keys_deleted.push(key_value);
            } else {
                failed_deletion.push(key_value.key);
            }
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
