use axum::extract::Path;

//PUT Request
pub(crate) async fn create_key(Path(key): Path<String>, value: String) {
    println!("Create key {} value {}", key, value);
    ()
}

//Patch Request
pub(crate) async fn update_key(Path(key): Path<String>, value: String) {
    println!("Update key {} value {}", key, value);
    ()
}

//Delete Request
pub(crate) async fn delete_key(Path(key): Path<String>, value: String) {
    println!("delete key {} value {}", key, value);
    ()
}

////Get Request
pub(crate) async fn get_key_value(Path(key): Path<String>, value: String) {
    println!("Get key {} value {}", key, value);
    ()
}

////Get Request
pub(crate) async fn get_prefix_key_value(Path(key): Path<String>, value: String) {
    println!("Get key with prefix {} value {}", key, value);
    ()
}
////Get Request with query parameter ?all=true
pub(crate) async fn get_keys() {
    println!("Get all key");
    ()
}