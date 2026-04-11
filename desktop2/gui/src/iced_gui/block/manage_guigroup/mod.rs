// Std
use std::{
    collections::{
        HashSet
    },
    sync::{
        Arc
    }
};

// Dependencies 
use iced::{
    self,
    alignment::{
        Horizontal,
        Vertical,
    },
    Element,
    Length,
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
use platform::{
    App,
};

use crate::{
    iced_gui::{
        Action,
    }
};

use super::{
    guigroup::{
        GUIGroup,
    },
};

pub enum Instruction {
    None,
    Close,
    Save,
}

#[derive(Clone)]
pub enum Message {
    Close,
    Save,

    Select(usize),
    Unselect(usize),
    GroupNameInput(String)
}

pub struct ManageGroup {
    group_name: String,
    all_apps: Arc<[App]>,
    selected: HashSet<usize>,
}

impl ManageGroup {
    pub fn from_guigroup(guigroup: &GUIGroup) -> Self {
        let group_name = guigroup.name().clone();
        let group = guigroup.group();

        let all_apps = group.all_apps.clone();
        let selected: HashSet<usize> = {
            let apps_i = group.apps_i.clone(); aa
                .into_iter()
                .collect()
        };

        ManageGroup { 
            group_name,
            all_apps,
            selected,
        }        
    }

    pub fn into_parts(self) -> (String, Arc<[App]>, HashSet<usize>) {
        let ManageGroup { group_name, all_apps, selected } = self;

        (group_name, all_apps, selected)
    }

    pub fn update(&mut self, message: Message) -> Instruction {
        match message {
            Message::Close => Instruction::Close,
            Message::Save => {
                if self.group_name.is_empty() {
                    return Instruction::None;
                } 
                Instruction::Save
            },
            // No Action
            _ => { 
                match message {
                    Message::GroupNameInput(input) => self.group_name = input,
                    Message::Select(apps_i) => { self.selected.insert(apps_i); },
                    Message::Unselect(apps_i) => { self.selected.remove(&apps_i); },
                    _ => unreachable!()
                }

                Instruction::None
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
            .align_right(Length::Fill);

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
            .width(Length::Fill)
            .into()
    }

    fn app_list(&self) -> Element<'_, Message> {
        let rows = self.all_apps
            .iter()
            .enumerate()
            .map(|(i, app)| {   
                let is_selected = self.selected.contains(&i);

                let checkbox = Checkbox::new(is_selected)
                    .on_toggle(move |toggled| {
                        match toggled {
                            true => Message::Select(i),
                            false => Message::Unselect(i)
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
            .width(Length::Fill)
            .height(Length::Fill);

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

impl From<Arc<[App]>> for ManageGroup {
    fn from(from: Arc<[App]>) -> Self {
        ManageGroup {
            group_name: String::new(),
            all_apps: from,
            selected: HashSet::new()
        }
    }
}
