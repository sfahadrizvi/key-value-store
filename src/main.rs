pub mod http_server;

use axum::{
    routing::{delete, get, patch, put}, Router,
};

use http_server::server::{
    create_key, 
    update_key, 
    delete_key, 
    get_key_value, 
    get_prefix_key_value, 
    get_keys
};


#[tokio::main()]
async fn main() {

    // build our application with a route
    let app = Router::new()
        .route("/key/:key", put(create_key))
        .route("/key/:key", patch(update_key))
        .route("/key/:key", delete(delete_key))
        .route("/key/:key", get(get_key_value))
        .route("/key_prefix/:key", get(get_prefix_key_value))
        .route("/keys", get(get_keys));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


