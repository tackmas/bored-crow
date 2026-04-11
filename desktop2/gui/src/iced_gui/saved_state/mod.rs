use std::{
    fs,
    path::{
        PathBuf,
    }
};

use directories::{
    ProjectDirs
};

use iced::{
    Task,
};

use serde::{
    Serialize,
    Deserialize
};

use serde_json;

use crate::{
    iced_gui::{
        State,

        block,
        settings::{
            SavedSettings
        },
    },
};

const USERNAME: &str = "Tackmas";
const APP_NAME: &str = "Bored Crow";

#[derive(Serialize, Deserialize)]
pub struct SavedState {
    pub block: block::Saved,
    pub settings: SavedSettings,
}

impl SavedState {
    pub fn from_state(state: &State) -> Self {
        Self {
            block: block::Saved::from_block(&state.block),
            settings: SavedSettings::from_settings(&state.settings),
        }
    }
}