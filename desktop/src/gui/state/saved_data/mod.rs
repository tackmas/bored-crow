use std::{fs, path::PathBuf};

use directories::ProjectDirs;

use iced::Task;

use serde::{Deserialize, Serialize};

use serde_json;

use crate::{APP_NAME, USERNAME};

use crate::core::block::BlockConfig;
use crate::gui::state::block::SavedBlock;
use crate::gui::state::settings::SavedSettings;
use crate::gui::state::State;

#[derive(Serialize, Deserialize)]
pub struct SavedData {
    pub block: SavedBlock,
    pub settings: SavedSettings,
}

impl SavedData {
    pub(super) fn from_state(state: &State, block_rule: Option<BlockConfig>) -> Self {
        Self {
            block: SavedBlock::from_block(&state.block, block_rule),
            settings: SavedSettings::from_settings(&state.settings),
        }
    }
    pub fn load() -> Option<Self> {
        let path = state_path();
        let json = fs::read_to_string(&path).unwrap();

        Self::from_json(&json)
    }

    pub async fn async_load() -> Option<Self> {
        let path = async_state_path().await;
        let json = tokio::fs::read_to_string(&path).await.unwrap();

        Self::from_json(&json)
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
