use super::server::{Key, ServerState};
use axum::{extract::State, http::StatusCode};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, Debug, Clone)]
struct KeyPrefix {
    prefix: String
}

////Request to get key matching a prefix
pub(crate) async fn get_keys(State(state): State<Arc<ServerState>>, body: String) -> Result<String, StatusCode> {
    info!("get_keys api called with keys {}", body);
    let json_key: Result<KeyPrefix, _> = serde_json::from_str(&body);
    
    if let Ok(key) = json_key  {
        let task_result = tokio::spawn(async move {    
            get_keys_with_prefix(key.prefix.clone(), State(state.clone())).await
        }).await;

        if let Ok(keys_found_res) = task_result {
            if let Ok(keys_found) = keys_found_res {
                let response_string = serde_json::to_string::<Vec<Key>>(&keys_found).unwrap();
                Ok(response_string)
            } else {
                Err(StatusCode::NOT_FOUND)    
            }
        } else {
            Err(StatusCode::NOT_FOUND)
        }
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn get_keys_with_prefix(key: String, State(state): State<Arc<ServerState>>) -> Result<Vec<Key>, StatusCode> {
    state.file_gateway.get_files_with_prefix(key).await
}
