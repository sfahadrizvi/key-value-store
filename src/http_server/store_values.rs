
use super::server::KeyValue;
use std::{collections::HashSet, fs::OpenOptions, io::Write};
use futures::future;
use axum::{extract::State, http::StatusCode};
use serde_json::Value;
use std::thread;
use std::time::Duration;

////Request to create a key. It will fail if the key already exists
pub(crate) async fn create_key(State(state): State<String>, body:String) -> Result<String, StatusCode> {
    info!("create_key api called");
    create_or_update_keys(State(state.clone()), body, true).await
}

////Request to update or create a key
pub(crate) async fn update_key(State(state): State<String>, body:String) -> Result<String, StatusCode> {
    use std::time::Instant;
    let now = Instant::now();

    info!("update_key api called");
    let ret = create_or_update_keys(State(state.clone()), body, false).await;
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
    ret
}

////Get Request
pub(crate) async fn create_or_update_keys(
    State(state): State<String>, 
    body:String, create_new: bool
) -> Result<String, StatusCode>  {
    debug!("create_or_update_key called");
    let mut keys_modified: Vec<String> = Vec::new();
    let mut failed_modification: Vec<String> = vec![];
    let json_body: Value = serde_json::from_str(&body).unwrap();
    
    if json_body.is_array() {
        let json_key_values: Vec<KeyValue> = serde_json::from_value(json_body).unwrap();
        if check_for_repeated_key(&json_key_values) {
            let create_tasks: Vec<_>  = json_key_values
                                    .iter()
                                    .map(|val| tokio::spawn(
                                        create_or_update_key(val.clone(), state.clone(), create_new)
                                        )
                                    )
                                    .collect();
        
            let task_results: Vec<Result<Result<String, String>, tokio::task::JoinError>> =  future::join_all(create_tasks).await;
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
    } else {
        let json_key_value: Result<KeyValue, serde_json::Error> = serde_json::from_value(json_body);
        if let Ok(key_value) = json_key_value  {
            let clone_key_value = key_value.clone();
            let task_result = tokio::spawn(async move {    
                create_or_update_key(clone_key_value, state, create_new).await
            }).await;
            if extract_failure_success(task_result) {
                keys_modified.push(key_value.key);
            } else {
                failed_modification.push(key_value.key);
            }    
        }
    }

    Ok(create_json_response(keys_modified, failed_modification))
}

fn extract_failure_success(val: Result<Result<String, String>, tokio::task::JoinError>) -> bool{
    if let Ok(key_value) = val {
        if let Ok(_) = key_value {
                true
        } else {
            false
        }
    } else {
        false
    }
}

async fn create_or_update_key(kv: KeyValue, state: String, create_new: bool) -> Result<String, String> {
    thread::sleep(Duration::from_secs(2));

    info!("{} key {} value {} with state {}", if create_new {"Create" } else {"Update"},kv.key, kv.value, state);
    
    if kv.key.len() == 0 || kv.value.len() == 0 || kv.key.contains("/") {
        warn!("Bad request, invalid key {} or value {}", kv.key, kv.value);
        return Err(format!("Bad request, invalid key {} or value {}", kv.key, kv.value));
    }

    let path = format!("{}/{}", state, kv.key);

    let file = OpenOptions::new()
                            .create_new(create_new)
                            .create(true)
                            .write(true)
                            .truncate(true)
                            .open(path);
                            
    match file {
        Ok(mut file) => {
            let ret = file.write_all(kv.value.as_bytes());
            match ret { 
                Ok(()) => Ok(kv.key),
                Err(err) => {
                    warn!("Failed to create key file with error {}", err);
                    Err(format!("Failed to create key file with error {}", err))
                }
            }
        },
        Err(err) => {
            warn!("Failed to create key file open options with error {}", err);
            Err(format!("Failed to create key file open options with error {}", err))
        }
    }

}

fn check_for_repeated_key(kv_vec :&Vec<KeyValue>) -> bool {
    let keys = kv_vec.iter()
        .map(|key| key.key.clone())
        .collect::<HashSet<String>>();
        
    if keys.contains("") ||  kv_vec.len() !=  keys.len() {
        warn!("Empty or repeated keys");
        false
    } else {
        true
    }
}

fn create_json_response(modified: Vec<String>, failed: Vec<String>) -> String {
    let mut response = modified
        .iter()
        .map(|val| format!(r#"{{"key":"{}", "modified": "true"}}"#, val)).collect::<Vec<_>>();
    response.append(
        &mut failed
            .iter()
            .map(|val| format!(r#"{{"key":"{}", "modified": "false"}}"#, val)).collect::<Vec<_>>()
    );

    response.join(",")
}