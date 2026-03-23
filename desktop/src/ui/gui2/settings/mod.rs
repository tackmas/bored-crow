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

    pub fn view(&self) -> Element<'_, SettingsMessage> {
        Text::new("Settings").into()
    }
}