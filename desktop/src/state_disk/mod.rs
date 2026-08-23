use std::{
    fs,
    path::{
        PathBuf,
    }
};

use directories::{
    ProjectDirs
};

use serde::{
    Serialize,
    
    de::{
        DeserializeOwned
    }
};

pub const USERNAME: &str = "Tackmas";
pub const APP_NAME: &str = "Bored_Crow";

pub fn load<T: DeserializeOwned>() -> Option<T> {
    let path = state_path();
    let json = fs::read_to_string(&path)
        .unwrap();

    from_json(&json)
}

pub async fn async_load<T: DeserializeOwned>() -> Option<T> {
    let path = async_state_path().await;
    let json = tokio::fs::read_to_string(&path)
        .await
        .unwrap();

    from_json(&json)
}

fn from_json<T: DeserializeOwned>(json: &str) -> Option<T> {
    if json.is_empty() {
        return None;
    }

    Some(
        serde_json::from_str(json)
            .unwrap()
    )        
}

pub async fn async_save<T: Serialize>(t: &T) {
    let path = async_state_path().await;
    tokio::fs::create_dir_all(path.parent().unwrap())
        .await
        .unwrap();

    let json = serde_json::to_string_pretty(t).unwrap();
    tokio::fs::write(path, json)
        .await
        .unwrap();
}


fn state_path() -> PathBuf {
    let proj_dirs = ProjectDirs::from("", USERNAME, APP_NAME)
        .expect("could not determine directories");

    let dir_path = proj_dirs.data_local_dir();
    let state_path = dir_path.join("state.json");

    fs::create_dir_all(dir_path)
        .expect("Failed to create data dir");

    fs::File::options()
        .write(true)
        .create(true)
        .open(&state_path)
        .unwrap();

    state_path
}

async fn async_state_path() -> PathBuf {
    let proj_dirs = ProjectDirs::from("", USERNAME, APP_NAME)
        .expect("could not determine directories");

    let dir_path = proj_dirs.data_local_dir();
    let state_path = dir_path.join("state.json");

    tokio::fs::create_dir_all(dir_path)
        .await
        .expect("Failed to create data dir");

    tokio::fs::File::options()
        .write(true)
        .create(true)
        .open(&state_path)
        .await
        .unwrap();

    state_path
}
