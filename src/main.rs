pub mod http_server;
extern crate pretty_env_logger;
#[macro_use] extern crate log;

use axum::{
    routing::{get, post}, Router
};
use clap::Parser;
use crate::http_server::{
    store_values::{create_key, update_key},
    delete_keys::delete_key,
    get_values::get_value,
    get_keys::get_keys,
};


use std::path::Path;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The port to listen on
    #[arg(short, long)]
    port: i32,

    /// The path to the key-value store
    #[arg(long, default_value = "./ssd")]
    path: String,
}

#[tokio::main()]
async fn main() -> Result<(), String> { 
    pretty_env_logger::init();
    let args = Args::parse();

    if !Path::new(&args.path).exists() {
        error!("The path to key value store does not exist");
        return Err("The path to key value store does not exist".to_string())
    }

    // build our application with a route
    let app = Router::new()
        .route(INSERT_END_POINT, post(create_key))
        .route(INSERT_UPDATE_END_POINT, post(update_key))
        .route(DELETE_END_POINT, post(delete_key))
        .route(KEYS_END_POINT, post(get_keys))
        .route(GET_END_POINT, get(get_value))
        .with_state(args.path.clone());

    let uri = format!("0.0.0.0:{}", args.port);
    let listener = tokio::net::TcpListener::bind(uri).await.unwrap();
    info!("Server starting at port {}", args.port);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

static GET_END_POINT: &str = "/get";
static KEYS_END_POINT: &str = "/keys";
static INSERT_END_POINT: &str = "/insert";
static INSERT_UPDATE_END_POINT: &str = "/update";
static DELETE_END_POINT: &str = "/delete";


