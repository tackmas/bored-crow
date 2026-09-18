use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;

use serde::{Deserialize, Serialize};

use serde_json;

use crate::{APP_NAME, USERNAME};
use crate::group::{BlockRuleKind, Id};

#[derive(Debug, Deserialize, Serialize)]
pub struct Saved {
    pub groups: Vec<Group>
}

impl Saved {
    fn new() -> Self {
        Self {
            groups: Vec::new()
        }
    }
    pub fn load() -> Self {
        let path = state_path();
        let json = fs::read_to_string(&path).unwrap();

        Self::from_json(&json)
            .unwrap_or_else(Self::new)
    }

    pub async fn async_load() -> Self {
        let path = async_state_path().await;
        let json = tokio::fs::read_to_string(&path).await.unwrap();

        Self::from_json(&json)
            .unwrap_or_else(Self::new)
    }

    fn from_json(json: &str) -> Option<Self> {
        if json.is_empty() {
            return None;
        }

        Some(serde_json::from_str(json).unwrap())
    }

    pub async fn async_save(&self) {
        let path = async_state_path().await;
        tokio::fs::create_dir_all(path.parent().unwrap())
            .await
            .unwrap();

        let json = serde_json::to_string_pretty(self).unwrap();
        tokio::fs::write(path, json).await.unwrap();
    }
}

fn state_path() -> PathBuf {
    let proj_dirs =
        ProjectDirs::from("", USERNAME, APP_NAME).expect("could not determine directories");

    let dir_path = proj_dirs.data_local_dir();
    let state_path = dir_path.join("state.json");

    println!("Saved State path: '{}'", state_path.to_str().unwrap());

    fs::create_dir_all(dir_path).expect("Failed to create data dir");

    fs::File::options()
        .write(true)
        .create(true)
        .open(&state_path)
        .unwrap();

    state_path
}

async fn async_state_path() -> PathBuf {
    let proj_dirs =
        ProjectDirs::from("", USERNAME, APP_NAME).expect("could not determine directories");

    let dir_path = proj_dirs.data_local_dir();
    let state_path = dir_path.join("state.json");

    println!("Saved State path: '{}'", state_path.to_str().unwrap());

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

#[derive(Debug, Deserialize, Serialize)]
pub struct Group {
    pub id: Id,
    pub name: String,
    pub apps: Vec<String>,
    pub block_config: Option<BlockRuleKind>
}