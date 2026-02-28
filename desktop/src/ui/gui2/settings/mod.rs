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
pub enum SettingsEvent {}

pub struct SettingsState {}

impl SettingsState {
    pub fn new() -> Self {
        SettingsState {  }
    }

    pub fn update(&mut self, event: SettingsEvent) {

    }

    pub fn view(&self) -> Element<'_, SettingsEvent> {
        Text::new("Settings").into()
    }
}