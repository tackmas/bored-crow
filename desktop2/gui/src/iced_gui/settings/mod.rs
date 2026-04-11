use iced::{
    self,
    Element,
    Length,
    widget::{
        Button,
        Column,
        Row,
        Text,
    }
};

use serde::{
    Serialize,
    Deserialize
};

#[derive(Clone)]
pub enum SettingsMessage {}

pub struct Modal {

}

pub struct SettingsState {}

impl SettingsState {
    pub fn new() -> Self {
        SettingsState {  }
    }

    pub fn update(&mut self, message: SettingsMessage) {

    }

    pub fn view(&self, show_modal: bool) -> Element<'_, SettingsMessage> {
        Text::new("Settings").into()
    }
}

#[derive(Serialize, Deserialize)]
pub struct SavedSettings {

}

impl SavedSettings {
    pub fn from_settings(settings: &SettingsState) -> Self {
        todo!()
    }
    pub fn into_settings(self) -> SettingsState {
        todo!()
    }
}