// Std
use std::collections::HashSet;

// Dependencies 
use iced::{
    self,
    alignment::{
        Horizontal,
        Vertical,
    },
    Element,
    Length::*,
    widget::{
        Button,
        Checkbox,
        Column,
        Container,
        Row,
        Scrollable,
        Space,
        Text,
        TextInput,
    }
};


// Local
use crate::{
    platform::{
        App,
    },
};

use super::{
    guigroup::{
        GUIGroup,
    },
};

pub enum Action {
    None,
    Close,
    Save,
}

#[derive(Clone)]
pub enum Message {
    Close,
    Save,

    Select(App),
    Unselect(App),
    GroupNameInput(String)
}

pub struct ManageGroup {
    group_name: String,
    apps: Vec<App>,
    selected: HashSet<App>,
}

impl ManageGroup {
    pub fn new() -> Self {
        let group_name = String::new();
        let apps = App::all_apps().unwrap();
        let selected = HashSet::new();

        ManageGroup {
            group_name, 
            apps,
            selected,
        }
    }
    pub fn into_name_and_apps(self) -> (String, HashSet<App>) {
        let ManageGroup {group_name, selected, ..} = self;

        (group_name, selected)
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::Close => Action::Close,
            Message::Save => {
                if self.group_name.is_empty() {
                    return Action::None;
                } 
                Action::Save
            },
            // No Action
            _ => { 
                match message {
                    Message::GroupNameInput(input) => self.group_name = input,
                    Message::Select(app) => { self.selected.insert(app); },
                    Message::Unselect(app) => { self.selected.remove(&app); },
                    _ => unreachable!()
                }

                Action::None
            }
        }
    }   

    pub fn view(&self) -> Element<'_, Message> {
        let group_name = self.get_group_name();
        let app_list = self.app_list();
        
        let save_close_buttons = 
            Container::new(
                Row::with_children([                
                    self.close_button(),
                    self.save_button(),
                    Space::new().into(),
                ])
                .spacing(20)               
            )
            .align_right(Fill);

        let content = 
            Column::with_children([
                group_name,
                app_list,
                save_close_buttons.into(),
            ]);

        content.into()
    }
    fn get_group_name(&self) -> Element<'_, Message> {
        TextInput::new("Group name: ", &self.group_name)
            .on_input(|input| Message::GroupNameInput(input))
            .width(Fill)
            .into()
    }

    fn app_list(&self) -> Element<'_, Message> {
        let rows = self.apps
            .iter()
            .map(|app| {   
                let is_selected = self.selected.contains(app);

                let checkbox = Checkbox::new(is_selected).on_toggle(|toggled| {
                    match toggled {
                        true => Message::Select(app.clone()),
                        false => Message::Unselect(app.clone())
                    }
                });

                let text = Text::new(app.name());

                Row::with_children([
                    checkbox.into(),
                    text.into(),
                ])
                .into()
            });

        let column = Column::with_children(rows);

        let scrollable = Scrollable::new(column)
            .spacing(0)
            .width(Fill)
            .height(Fill);

        scrollable.into()
    }

    fn save_button(&self) -> Element<'_, Message> {
        let text = Text::new("Save");

        let save_button = Button::new(text)
            .on_press(Message::Save);

        save_button.into()
    }

    fn close_button(&self) -> Element<'_, Message> {
        let text = Text::new("Close without saving");

        let close_button = Button::new(text)
            .on_press(Message::Close);

        close_button.into()
    }
}

impl From<&GUIGroup> for ManageGroup {
    fn from(value: &GUIGroup) -> Self {
        let (group_name, group) = (
            value
                .name()
                .clone(), 
            value
                .group()
                .clone()
        );

        let apps = App::all_apps().unwrap();
        let selected: HashSet<App> = {
            let apps = group.apps.clone();
            apps
                .into_iter()
                .collect()
        };

        ManageGroup { 
            group_name,
            apps,
            selected,
        }
    }
}