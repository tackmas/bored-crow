mod block_config;

use std::{
    sync::{
        Arc,
    },
};

// Dependencies (alphabetical order)
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
};

use crate::{
    core::{
        block_config::{
            Group
        }
    },
    platform::{
        App,
        Blocker,
    },
    gui2::{
        button_with_text,
        Route,
    }
};

use crate::{
    unwrap_variant,
};

use super::{
    manage_guigroup::{
        self as m_gg,
        ManageGroup,
    },
};

use block_config as b_c;

#[derive(Clone)]
pub enum Action {
    None,
    Delete,
    CloseModal,
    OpenModal,
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
    is_blocked: bool,
    group: Arc<Group>,
    blocker: Blocker,
    block_config: b_c::BlockConfig,
}

impl GUIGroup {
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn group(&self) -> &Group {
        &self.group
    }
    pub fn update(&mut self, msg: Message) -> Action {
        match msg {
            Message::Delete => Action::Delete,
            Message::Edit(route) => {
                match (&mut self.modal, route) {
                    (Some(Modal::EditSelf(edit_guigroup)), Route::Forward(msg)) => {
                        let action = edit_guigroup.update(msg);

                        match action {
                            m_gg::Action::None => Action::None,
                            m_gg::Action::Close => {
                                self.modal = None;
                                Action::CloseModal
                            },
                            m_gg::Action::Save => {
                                let modal = self.modal
                                    .take()
                                    .unwrap();

                                let edit_guigroup = unwrap_variant!(modal, Modal::EditSelf);

                                let (name, apps) = edit_guigroup.into_name_and_apps();
                                let apps = apps
                                    .into_iter()
                                    .collect();

                                let group = crate::core::block_config::Group::new_with(apps);
                                let blocker = self.blocker.clone();
                                
                                *self = GUIGroup::from((name, group, blocker));

                                Action::CloseModal
                            }
                        }
                    },
                    (None, Route::Open(())) => {
                        let edit_guigroup = m_gg::ManageGroup::from(&*self); // &*self is to make it an immutable reference
                        self.modal = Some(Modal::EditSelf(edit_guigroup));

                        Action::OpenModal
                    },
                    _ => unreachable!()
                }
            },
            Message::BlockConfig(msg) => {
                let action = self.block_config.update(msg);

                match action {
                    b_c::Action::None => Action::None,
                    b_c::Action::Close => {
                        self.modal = None;
                        Action::CloseModal
                    }
                }
            }
            Message::Block => {
                self.modal = Some(Modal::BlockConfig);
                
                Action::OpenModal
            },
            Message::Unblock => unimplemented!(),
        }
    }
    pub fn view(&self) -> Element<'_, Message> {
        if let Some(m) = &self.modal {
            return self.modal(m);
        }

        self.guigroups_element()
    }
    fn modal<'a>(&'a self, modal: &'a Modal) -> Element<'a, Message> {
        match modal {
            Modal::EditSelf(edit_guigroup) => edit_guigroup
                .view()
                .map(|msg| Message::Edit(Route::Forward(msg))),
            Modal::BlockConfig => self.block_config
                .view()
                .map(Message::BlockConfig)
        }
    }


    fn guigroups_element(&self) -> Element<'_, Message> {
        Responsive::new(|size| {
            let (_, screen_height) = crate::gui2::screen_size();

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
    /* fn block_config_overlay(&self) -> Element<'_, Message> {
        let block_rules = self.0
            .as_ref()
            .unwrap();

        let tabs = Row::with_children([
            button_with_text("Timer", BlockRulesAction::Timer),
        ]);

        let tab_content = 
            match &block_rules.screen {
                BlockRulesScreen::Timer => block_rules.timer
                    .view()
                    .map(|_msg| BlockRulesAction::Timer)
            };

        let finish_buttons =
            Container::new(
                Row::with_children([
                    button_with_text("Close", BlockRulesAction::Close),
                    button_with_text("Block without lock", BlockRulesAction::BlockWithoutLock),
                    button_with_text("Block and lock", BlockRulesAction::BlockWithLock),
                    Space::new().into(),
                ])
                .spacing(20)
            )
            .align_right(Fill);


        let column = Column::with_children([
            tabs.into(),
            tab_content,
            finish_buttons.into(),
        ]);

        column.create_modal();

        todo!();
    }
    */
}

impl From<(String, Group, Blocker)> for GUIGroup {
    fn from(value: (String, Group, Blocker)) -> Self {
        let (name, group, blocker) = value;
        let group = Arc::new(group);

        GUIGroup {
            modal: None,
            name,
            group: group.clone(),
            is_blocked: false,
            blocker: blocker.clone(),
            block_config: b_c::BlockConfig::from((blocker, group)),
        }
    }
}