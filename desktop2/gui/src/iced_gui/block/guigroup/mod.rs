mod block_config;

use std::{
    sync::{
        Arc,
    },
};

// Dependencies
use iced::{
    self,
    alignment::{
        Horizontal::*,
    },
    Color,
    Element,
    Length::*,
    Size,
    widget::{
        Button,
        Column,
        Container,
        container,
        Responsive,
        Row,
        Scrollable,
        Space,
        Stack,
        Text,
    },
    Task,
};

use serde::{
    Serialize,
    Deserialize
};

use engine::{
    block_config::{
        Group
    }
};
use platform::{
    App,
    Blocker,
};

use crate::iced_gui::{
    Action,
    Route,

    button_with_text,
    screen_size,
};

use utils::{
    unwrap_variant,
};

use super::{
    manage_guigroup::{
        self as m_gg,
        ManageGroup,
    },
};

use block_config as b_c;

pub enum Instruction {
    None,
    Delete,
    CloseModal(Task<Message>),
    OpenModal,
    Task(Task<Message>)
}

#[derive(Clone)]
pub enum Message {
    Delete,
    Edit(Route<m_gg::Message>),
    BlockConfig(b_c::Message),
    Block,
    Unblock,
}

pub enum Modal {
    EditSelf(ManageGroup),
    BlockConfig,
}

pub struct GUIGroup {
    modal: Option<Modal>,
    name: String,
    group: Arc<Group>,
    is_blocked: bool,
    blocker: Blocker,
    block_config: b_c::BlockConfig,
}

impl GUIGroup {
    pub fn new(blocker: Blocker, name: String, group: Group) -> Self {
        let group = Arc::new(group);

        GUIGroup {
            modal: None,
            name,
            group: group.clone(),
            is_blocked: false,
            blocker: blocker.clone(),
            block_config: b_c::BlockConfig::new(blocker, group),
        }
    }
    pub fn from_manage_group(blocker: Blocker, manage_group: ManageGroup) -> Self {
        let (name, all_apps, apps_i) = manage_group.into_parts();
        let apps_i: Vec<usize> = apps_i
            .into_iter()
            .collect();

        let group = Group::from_apps_i(apps_i, all_apps);
        let group = Arc::new(group);

        GUIGroup {
            modal: None,
            name,
            group: group.clone(),
            is_blocked: false,
            blocker: blocker.clone(),
            block_config: b_c::BlockConfig::new(blocker, group),
        }             
    }

    pub fn from_saved(saved: Saved, all_apps: Arc<[App]>, blocker: Blocker) -> Self {
        let group = Arc::new(
            Group::from_app_names(saved.app_names, all_apps.clone())
        );

        GUIGroup {
            modal: None,
            name: saved.name,
            group: group.clone(),
            is_blocked: group.is_blocked(),
            blocker: blocker.clone(),
            block_config: b_c::BlockConfig::new(blocker, group),
        }
    }

    pub fn update(&mut self, show_modal: bool, msg: Message) -> Action<Instruction> {
        match msg {
            Message::Delete => {
                if self.is_blocked {
                    return Action::no_save(Instruction::None);
                }

                Action::no_save(Instruction::None)
            },
            Message::Edit(route) => {
                match (&mut self.modal, route) {
                    (Some(Modal::EditSelf(edit_guigroup)), Route::Forward(msg)) => {
                        let instruction = edit_guigroup.update(msg);

                        self.handle_m_gg_instruction(instruction)
                    },
                    (None, Route::Open(())) => {
                        if self.is_blocked {
                            return Action::no_save(Instruction::None);
                        }

                        let edit_guigroup = m_gg::ManageGroup::from_guigroup(&*self); // &*self is to make it an immutable reference
                        self.modal = Some(Modal::EditSelf(edit_guigroup));

                        Action::no_save(Instruction::OpenModal)
                    },
                    _ => unreachable!()
                }
            },
            Message::BlockConfig(msg) => {
                let instruction = self.block_config.update(msg);

                self.handle_b_c_instruction(instruction)
            }
            Message::Block => {
                self.modal = Some(Modal::BlockConfig);
                
                Action::no_save(Instruction::OpenModal)
            },
            Message::Unblock => {
                if self.group.unblock().is_ok() {
                    self.is_blocked = false;
                }

                return Action::save(Instruction::None);
            },
        }
    }
    pub fn view(&self, show_modal: bool) -> Element<'_, Message> {
        if show_modal {
            return self.modal();
        }

        self.guigroups_element()
    }
}


// Helper functions for update
impl GUIGroup {
    fn handle_b_c_instruction(&mut self, instruction: b_c::Instruction) -> Action<Instruction> {
        match instruction {
            b_c::Instruction::None => Action::no_save(Instruction::None),
            b_c::Instruction::Close => {
                self.modal = None;

                Action::no_save(Instruction::CloseModal(Task::none()))
            }
            b_c::Instruction::Block(task) => {
                self.modal = None;
                self.is_blocked = true;

                Action::save(Instruction::CloseModal(task.map(Message::BlockConfig)))
            }
            b_c::Instruction::Task(task) => {
                Action::no_save(Instruction::Task(task.map(Message::BlockConfig)))
            }
        }
    }

    fn handle_m_gg_instruction(&mut self, instruction: m_gg::Instruction) -> Action<Instruction> {
        match instruction {
            m_gg::Instruction::None => Action::no_save(Instruction::None),
            m_gg::Instruction::Close => {
                self.modal = None;

                Action::no_save(Instruction::CloseModal(Task::none()))
            },
            m_gg::Instruction::Save => {
                let blocker = self.blocker.clone();
                let modal = self.modal
                    .take()
                    .unwrap();
                let edit_guigroup = unwrap_variant!(modal, Modal::EditSelf => yo);
                
                *self = GUIGroup::from_manage_group(blocker, edit_guigroup);

                Action::save(Instruction::CloseModal(Task::none()))
            }
        }
    }
}

// Helper functions for view
impl GUIGroup {
    fn modal(&self) -> Element<'_, Message> {
        match &self.modal {
            Some(Modal::EditSelf(edit_guigroup)) => edit_guigroup
                .view()
                .map(|msg| Message::Edit(Route::Forward(msg))),
            Some(Modal::BlockConfig) => self.block_config
                .view()
                .map(Message::BlockConfig),
            _ => unreachable!()
        }
    }

    fn guigroups_element(&self) -> Element<'_, Message> {
        Responsive::new(|size| {
            let (_, screen_height) = screen_size();

            let left_empty_space = Space::new()
                .width(size.width / 12.0);

            let guigroup_name_text = Text::new(&self.name)
                .size(20)
                .align_x(Left)
                .width(Fill)
                .color(Color::WHITE);

            let row = Row::with_children([
                left_empty_space.into(),
                guigroup_name_text.into(),
                self.guigroup_buttons()
            ])
            .height(Fill);

            Container::new(row)
                .width(size.width / 1.2)
                .height(screen_height / 10)
                .style(|_| {
                    container::background(Color::from_rgb(0.0, 0.0, 0.0))
                })
                .into()
        })
        .into()
    }

    fn guigroup_buttons(&self) -> Element<'_, Message> {
        let buttons_row = 
            Row::with_children([
                self.delete_guigroup_button(),
                self.edit_guigroup_button(),
                self.block_guigroup_button(),
            ])
            .spacing(20);


        buttons_row.into()
    }

    fn delete_guigroup_button(&self) -> Element<'_, Message> {
        let text = Text::new("Delete")
            .size(20);

        let button = Button::new(text)
            .on_press(Message::Delete);

        button.into()
    }

    fn edit_guigroup_button(&self) -> Element<'_, Message> {
        let text = Text::new("Edit")
            .size(20);

        let button = Button::new(text)
            .on_press(Message::Edit(Route::Open(())));

        button.into()
    }

    fn block_guigroup_button(&self) -> Element<'_, Message> {
        let (text_str, message) =
            if !self.is_blocked {
                ("Block", Message::Block)
            } else {
                ("Unblock", Message::Unblock)
            };

        let text = Text::new(text_str)
            .align_x(Right)
            .size(20);
            
        let button = Button::new(text)
            .on_press(message);

        button.into()        
    }    
}

// Arbitrary functions
impl GUIGroup {
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn group(&self) -> &Group {
        &self.group
    }
}

#[derive(Serialize, Deserialize)]
pub struct Saved {
    name: String,
    is_blocked: bool,
    app_names: Vec<String>
}

impl Saved {
    pub fn from_guigroup(guigroup: &GUIGroup) -> Self {
        Saved {
            name: guigroup.name.clone(),
            is_blocked: guigroup.is_blocked,
            app_names: guigroup.group.app_names_owned()
        }
    }
}

