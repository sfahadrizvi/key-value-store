use std::collections::HashMap;

use axum::{
    extract::{Path, Query, Request},
    http::StatusCode,
    routing::{delete, get, patch, post, put},
    Json, Router,
};

#[tokio::main()]
async fn main() {
    // initialize tracing

    // build our application with a route
    let app = Router::new()
        .route("key/:key", put(create_key))
        .route("key/:key", patch(update_key))
        .route("key/:key", delete(delete_key))
        .route("key/:key", get(get_key_value))
        .route("key_prefix/:key", get(get_prefix_key_value))
        .route("keys", get(get_keys));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

//PUT Request
async fn create_key(Path(key): Path<String>, value: String) {
    println!("Create key {} value {}", key, value);
    ()
}

//Patch Request
async fn update_key(Path(key): Path<String>, value: String) {
    println!("Update key {} value {}", key, value);
    ()
}

//Delete Request
async fn delete_key(Path(key): Path<String>, value: String) {
    println!("delete key {} value {}", key, value);
    ()
}

////Get Request
async fn get_key_value(Path(key): Path<String>, value: String) {
    println!("Get key {} value {}", key, value);
    ()
}

////Get Request
async fn get_prefix_key_value(Path(key): Path<String>, value: String) {
    println!("Get key with prefix {} value {}", key, value);
    ()
}
////Get Request with query parameter ?all=true
async fn get_keys(Path(key): Path<String>, value: String) {
    println!("Get all key {} value {}", key, value);
    ()
}
