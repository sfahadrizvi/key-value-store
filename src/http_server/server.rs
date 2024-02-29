use std::{collections::HashMap, fs::{self, OpenOptions}, io::Write};

use axum::{extract::{Path, Query, State}, http::StatusCode};

//PUT Request
pub(crate) async fn create_key(Query(params): Query<HashMap<String, String>>, State(state): State<String>, body:String)
 -> Result<(), StatusCode> {
    create_or_update_key(Query(params), State(state), true)
}

//Patch Request
pub(crate) async fn update_key(Query(params): Query<HashMap<String, String>>, State(state): State<String>, value: String)
-> Result<(), StatusCode> {
    create_or_update_key(Query(params), State(state), false)
}

//Delete Request
pub(crate) async fn delete_key(Path(key): Path<String>, State(state): State<String>, value: String)
-> Result<(), StatusCode> {
    println!("delete key {} value {}", key, value);
    let path = format!("{}/{}", state, key);
    if !std::path::Path::new(&path).exists() {
        println!("deleteting nonexisting key {}", key);
        Ok(())
    } else {
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(err) => {
                println!("Key could not be delted: {}", err);
                Err(StatusCode::NOT_MODIFIED)
            }
        }
    }
}

////Get Request
pub(crate) async fn get_value(Path(key): Path<String>, State(state): State<String>, value: String)
-> Result<(), StatusCode> {
    println!("Get key {} value {}", key, value);
    Ok(())
}

////Get Request with query parameter ?all=true
pub(crate) async fn get_keys() -> Result<(), StatusCode> {
    println!("Get all key");
    Ok(())
}

fn create_or_update_key(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<String>,
    create_new: bool
)
 -> Result<(), StatusCode> {
    println!("{} key {} value {} with state {}", if create_new {"Create" } else {"Update"}, params["key"], params["value"], state);
    
    if params.get("key").is_none() || params.get("value").is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let path = format!("{}/{}", state, params["key"]);
    if path.split("/").count() != 2 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let file = OpenOptions::new()
                            .create_new(create_new)
                            .create(true)
                            .write(true)
                            .truncate(true)
                            .open(path);
                            
    match file {
        Ok(mut file) => {
            let ret = file.write_all(params["value"].as_bytes());
            match ret { 
                Ok(()) => Ok(()),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        },
        Err(err) => {
            println!("Error: {}", err);
            Err(StatusCode::NOT_MODIFIED)
        }
    }
}