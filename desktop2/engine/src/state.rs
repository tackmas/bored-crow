use std::{
    collections::{
        HashMap
    },
    fs,
    path::{
        PathBuf
    },
    sync::{
        Arc
    }
};

use directories::{
    ProjectDirs,
};

use platform::{
    App
};

use serde::{
    Serialize,
    Deserialize,
    de::{
        DeserializeOwned,
    },
};

use serde_json;

use crate::{
    block_config::{
        BlockRule,
        BlockRuleKind,
        Group,
        SavedGroup,
    }
};

const USERNAME: &str = "Tackmas";
const APP_NAME: &str = "Bored Crow";

#[derive(Serialize, Deserialize)]
pub struct SavedState {
    groups: Vec<SavedGroup>
}

impl SavedState {
    pub async fn load() -> Self {
        let path = state_path().await;
        let json = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&json).unwrap()
    }

    pub async fn save(&self) {
        let path = state_path().await;
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let json = serde_json::to_string_pretty(self).unwrap();
        fs::write(path, json).unwrap();
    }
}

async fn state_path() -> PathBuf {
    let proj_dirs = ProjectDirs::from("", USERNAME, APP_NAME)
        .expect("could not determine directories");

    let data_local_dir = proj_dirs.data_local_dir();

    tokio::fs::create_dir_all(data_local_dir)
        .await
        .expect("Failed to create data dir");

    data_local_dir.join("state.json")
}