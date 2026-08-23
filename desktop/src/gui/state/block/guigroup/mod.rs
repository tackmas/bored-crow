mod block_config;

use std::sync::Arc;

// Dependencies
use iced::{
    self, Color, Element,
    Length::*,
    Size, Task,
    alignment::Horizontal::*,
    widget::{
        Button, Column, Container, Responsive, Row, Scrollable, Space, Stack, Text, container,
    },
};

use serde::{Deserialize, Serialize};

use crate::{
    core::block::{BlockConfig, Group, SavedGroup},
    platform::{App, Blocker},
    unwrap_variant,
};

use crate::core::block::id::{AsId, Id};
use crate::gui::state::{action, button_with_text, Route, SCREEN_SIZE};

use super::manage_guigroup::{self as m_gg, ManageGroup};

use block_config as b_c;

pub enum CustomAction {
    Delete,
    Block(Arc<Group>, BlockConfig),
}

pub type CA = CustomAction;
pub type Action = action::Action<CA, Message>;

#[derive(Clone)]
pub enum Message {
    Delete,
    EditGUIGroup(Route<m_gg::Message>),
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
    // Non-atomic bool when code needs to load is_blocked for View for performance.
    // This should always be synced with Group's AtomicBool because either GUIGroup mutates is_blocked, 
    // or Group notifies that it has mutated its AtmoicBool has changed, 
    // therefore GUIGroup is always aware and can sync up.
    is_blocked: bool,
    block_config: b_c::BlockConfig,

}

impl GUIGroup {
    pub fn new(name: String, group: Group) -> Self {
        let group = Arc::new(group);

        Self {
            modal: None,
            name,
            group: group.clone(),
            is_blocked: false,
            block_config: b_c::BlockConfig::new(),
        }
    }
    pub fn from_manage_group(manage_group: ManageGroup, id: Id) -> Self {
        let (name, all_apps, apps_i) = manage_group.into_parts();
        let apps_i: Vec<usize> = apps_i.into_iter().collect();

        let group = Group::from_apps_i(apps_i, all_apps, id);
        let group = Arc::new(group);

        Self {
            modal: None,
            name,
            group: group.clone(),
            is_blocked: false,
            block_config: b_c::BlockConfig::new(),
        }
    }

    pub async fn from_saved(saved: SavedGUIGroup, all_apps: Arc<[App]>, blocker: &Blocker) -> Self {
        let group = Group::from_saved(saved.group, all_apps, blocker).await;
        let is_blocked = group.is_blocked();

        let guigroup = Self {
            modal: None,
            name: saved.name,
            group: group.clone(),
            is_blocked,
            block_config: b_c::BlockConfig::new(),
        };

        guigroup
    }
    // all_guigroups are two slices that contains every GUIGroup, except self
    // This is to avoid having a mutable and immutable reference to self.
    #[must_use]
    pub fn update(&mut self, msg: Message, all_guigroups: [&[GUIGroup]; 2]) -> Action {
        match msg {
            Message::Delete => {
                if self.group.is_blocked() {
                    return Action::none();
                }

                Action::none_with_custom(CustomAction::Delete)
            }
            Message::EditGUIGroup(route) => match (&mut self.modal, route) {
                (Some(Modal::EditSelf(edit_guigroup)), Route::Forward(msg)) => {
                    let all_names = all_guigroups
                        .into_iter()
                        .flatten()
                        .map(|g| &g.name);

                    let action = edit_guigroup.update(msg, all_names);

                    self.handle_m_gg_action(action)
                }
                (None, Route::Open(())) => {
                    if self.group.is_blocked() {
                        return Action::none();
                    }

                    let edit_guigroup = m_gg::ManageGroup::from_guigroup(&*self);
                    self.modal = Some(Modal::EditSelf(edit_guigroup));

                    Action::none().open_modal()
                }
                _ => unreachable!(),
            },
            Message::BlockConfig(msg) => {
                let action = self.block_config.update(msg);

                self.handle_b_c_action(action)
            }
            Message::Block => {
                self.modal = Some(Modal::BlockConfig);

                Action::none().open_modal()
            }
            Message::Unblock => {
                if self.group.unblock().is_err() {
                    return Action::none().save();
                }

                self.is_blocked = false;

                Action::none()
            }
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
    fn handle_b_c_action(&mut self, mut action: b_c::Action) -> Action {
        match action.custom_opt.take() {
            None => action.with_custom(None),
            Some(b_c::CA::Close) => {
                self.modal = None;

                action.with_custom(None).close_modal()
            }
            Some(b_c::CA::Block(block_rule_kind)) => {
                self.modal = None;
                self.is_blocked = true;

                let group = self.group.clone();
                let block_rule = BlockConfig::from_parts(self.as_id(), block_rule_kind);

                action
                    .with_custom(Some(CA::Block(group, block_rule)))
                    .close_modal()
                    .save()
            }
        }
        .map_task(Message::BlockConfig)
    }

    fn handle_m_gg_action(&mut self, mut action: m_gg::Action) -> Action {
        match action.custom_opt.take() {
            None => action.with_custom(None),
            Some(m_gg::CustomAction::Close) => {
                self.modal = None;

                action.with_custom(None).close_modal()
            }
            Some(m_gg::CustomAction::Save) => {
                let modal = self.modal.take().unwrap();
                let edit_guigroup = unwrap_variant!(modal, Modal::EditSelf => yo);

                *self = GUIGroup::from_manage_group(edit_guigroup, self.group.id);

                action.with_custom(None).close_modal().save()
            }
        }
        .map_task(|msg| Message::EditGUIGroup(Route::Forward(msg)))
    }
}

// Helper functions for view
impl GUIGroup {
    // This function should only be called if self.modal is Some(...)
    fn modal(&self) -> Element<'_, Message> {
        let modal = self.modal.as_ref().unwrap(); 

        match modal {
            Modal::EditSelf(edit_guigroup) => edit_guigroup
                .view()
                .map(|msg| Message::EditGUIGroup(Route::Forward(msg))),
            Modal::BlockConfig => self.block_config.view().map(Message::BlockConfig)
        }
    }

    fn guigroups_element(&self) -> Element<'_, Message> {
        Responsive::new(|size| {
            let left_empty_space = Space::new().width(size.width / 12.0);

            let guigroup_name_text = Text::new(&self.name)
                .size(20)
                .align_x(Left)
                .width(Fill)
                .color(Color::WHITE);

            let row = Row::with_children([
                left_empty_space.into(),
                guigroup_name_text.into(),
                self.guigroup_buttons(),
            ])
            .height(Fill);

            Container::new(row)
                .width(size.width / 1.2)
                .height(SCREEN_SIZE.height / 10)
                .style(|_| container::background(Color::from_rgb(0.0, 0.0, 0.0)))
                .into()
        })
        .into()
    }

    fn guigroup_buttons(&self) -> Element<'_, Message> {
        let buttons_row = Row::with_children([
            self.delete_guigroup_button(),
            self.edit_guigroup_button(),
            self.block_guigroup_button(),
        ])
        .spacing(20);

        buttons_row.into()
    }

    fn delete_guigroup_button(&self) -> Element<'_, Message> {
        let text = Text::new("Delete").size(20);

        let button = Button::new(text).on_press(Message::Delete);

        button.into()
    }

    fn edit_guigroup_button(&self) -> Element<'_, Message> {
        let text = Text::new("Edit").size(20);

        let button = Button::new(text).on_press(Message::EditGUIGroup(Route::Open(())));

        button.into()
    }

    fn block_guigroup_button(&self) -> Element<'_, Message> {
        let (text_str, message) = if !self.is_blocked {
            ("Block", Message::Block)
        } else {
            ("Unblock", Message::Unblock)
        };

        let text = Text::new(text_str).align_x(Right).size(20);

        let button = Button::new(text).on_press(message);

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

impl AsId for GUIGroup {
    fn as_id(&self) -> Id {
        self.group.as_id()
    }
}

#[derive(Serialize, Deserialize)]
pub struct SavedGUIGroup {
    pub name: String,
    pub group: SavedGroup,
}

impl SavedGUIGroup {
    pub fn from_guigroup(guigroup: &GUIGroup, block_rule_opt: Option<BlockConfig>) -> Self {
        SavedGUIGroup {
            name: guigroup.name.clone(),
            group: SavedGroup::new(&guigroup.group, block_rule_opt),
        }
    }
}

