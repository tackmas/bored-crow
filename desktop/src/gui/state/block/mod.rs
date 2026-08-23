mod guigroup;
mod manage_guigroup;

pub use guigroup::SavedGUIGroup;

use std::sync::Arc;

// Dependencies (alphabetical order)

use futures::future;

use iced::{
    self, Color, Element,
    Length::*,
    Task,
    alignment::Horizontal::*,
    widget::{Button, Column, Container, Row, Scrollable, Space, Text},
};

use serde::{Deserialize, Serialize};

// Local
use crate::{
    platform::{App, Blocker},
    unwrap_variant,
};

use crate::core::block::{BlockConfig, Group};
use crate::core::block::id::Id;
use crate::gui::state::{action, handle_modal_action, Route, SCREEN_SIZE};

use guigroup::GUIGroup;
use manage_guigroup as m_gg;

pub enum CustomAction {
    Block(Arc<Group>, BlockConfig),
}

pub type CA = CustomAction;
pub type Action = action::Action<CA, Message>;

#[derive(Clone)]
pub enum Message {
    NewGUIGroup(Route<m_gg::Message>),
    GUIGroup(usize, guigroup::Message),
}

pub enum Modal {
    GUIGroup(usize),
    NewGUIGroup(m_gg::ManageGroup),
}

pub struct BlockState {
    modal: Option<Modal>,
    guigroups: Vec<GUIGroup>,
    all_apps: Arc<[App]>,
}

impl BlockState {
    pub fn new() -> Self {
        Id::init().unwrap();

        let all_apps = App::all_apps().unwrap().into();

        BlockState {
            modal: None,
            guigroups: Vec::new(),
            all_apps,
        }
    }

    pub async fn from_saved(saved: SavedBlock, blocker: &Blocker) -> Self {
        let all_apps: Arc<[App]> = App::all_apps().unwrap().into();

        let guigroups = future::join_all(
            saved.guigroups
                .into_iter()
                .map(|saved_guigroup| {
                    let all_apps = Arc::clone(&all_apps);
                    GUIGroup::from_saved(saved_guigroup, all_apps, blocker)
                })
        ).await;
            
        Id::init_with(&guigroups).unwrap();

        BlockState {
            modal: None,
            guigroups,
            all_apps,
        }
    }

    #[must_use]
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::NewGUIGroup(route) => match (&mut self.modal, route) {
                (Some(Modal::NewGUIGroup(new_guigroup)), Route::Forward(msg)) => {
                    let all_names = self.guigroups.iter().map(|g| g.name());

                    let action = new_guigroup.update(msg, all_names);

                    self.handle_m_gg_action(action)
                }
                (None, Route::Forward(_)) => Action::none(),
                (None, Route::Open(())) => {
                    let all_apps = self.all_apps.clone();
                    let new_guigroup = m_gg::ManageGroup::from(all_apps);
                    self.modal = Some(Modal::NewGUIGroup(new_guigroup));

                    Action::none().open_modal()
                }
                _ => panic!("We have a problem"),
            },

            Message::GUIGroup(i, msg) => {
                let (left, right) = self.guigroups.split_at_mut(i);
                let (mid, right) = right.split_first_mut().unwrap();

                let action = mid.update(msg, [&*left, &*right]);

                self.handle_guigroup_action(i, action)
            }
        }
    }

    fn handle_guigroup_action(&mut self, i: usize, mut action: guigroup::Action) -> Action {
        handle_modal_action(&mut self.modal, action.modal, || Modal::GUIGroup(i));

        match action.custom_opt.take() {
            None => action.with_custom(None),
            Some(guigroup::CA::Delete) => {
                self.guigroups.remove(i);

                action.with_custom(None).save()
            }
            Some(guigroup::CA::Block(group, block_rule)) => {
                action.with_custom(Some(CustomAction::Block(group, block_rule)))
            }
        }
        .map_task(move |msg| Message::GUIGroup(i, msg))
    }

    fn handle_m_gg_action(&mut self, mut action: m_gg::Action) -> Action {
        match action.custom_opt.take() {
            None => action.with_custom(None),
            Some(m_gg::CA::Close) => {
                self.modal = None;

                action.with_custom(None).close_modal()
            }
            Some(m_gg::CA::Save) => {
                let new_guigroup = {
                    let modal = self.modal.take().unwrap();

                    unwrap_variant!(modal, Modal::NewGUIGroup => new_guigroup)
                };
                let id = Id::new_unique_id().unwrap();

                let guigroup = GUIGroup::from_manage_group(new_guigroup, id);

                self.guigroups.push(guigroup);

                action.with_custom(None).close_modal().save()
            }
        }
        .map_task(|msg| Message::NewGUIGroup(Route::Forward(msg)))
    }

    pub fn view(&self, show_modal: bool) -> Element<'_, Message> {
        if show_modal {
            return self.modal();
        }
        let first_row = self.first_row();
        let guigroup_list = self.guigroup_list();

        let column = Column::with_children([first_row, guigroup_list])
            .width(Fill)
            .height(Fill)
            .align_x(Center)
            .spacing(20);

        column.into()
    }
    fn modal(&self) -> Element<'_, Message> {
        match &self.modal {
            Some(Modal::GUIGroup(i)) => {
                let i = *i;

                guigroup_element(&self.guigroups[i], i, true)
            }
            Some(Modal::NewGUIGroup(new_guigroup)) => new_guigroup
                .view()
                .map(|msg| Message::NewGUIGroup(Route::Forward(msg))),
            _ => unreachable!(),
        }
    }

    fn first_row(&self) -> Element<'_, Message> {
        let title = Container::new(Text::new("App Blocks").size(25))
            .center_x(FillPortion(3))
            .center_y(Fill);

        let new_guigroup_button = Container::new(
            Button::new(Text::new("New Group").size(15))
                .on_press(Message::NewGUIGroup(Route::Open(()))),
        )
        .center_x(FillPortion(2))
        .center_y(Fill);

        let space = Space::new().width(FillPortion(8));

        let first_row = Container::new(Row::with_children([
            title.into(),
            space.into(),
            new_guigroup_button.into(),
        ]))
        .center_x(Fill)
        .center_y(SCREEN_SIZE.height / 14);

        first_row.into()
    }

    fn guigroup_list(&self) -> Element<'_, Message> {
        if self.guigroups.is_empty() {
            return Container::new(Text::new("No groups currently exist"))
                .center(Fill)
                .into();
        }

        let guigroups_element: Vec<_> = self
            .guigroups
            .iter()
            .enumerate()
            .map(|(i, guigroup)| guigroup_element(guigroup, i, false))
            .collect();

        let guigroup_list = Column::with_children(guigroups_element)
            .width(Fill)
            .align_x(Center)
            .spacing(10);

        Scrollable::new(guigroup_list).spacing(0).into()
    }
}

fn guigroup_element<'a>(
    guigroup: &'a guigroup::GUIGroup,
    i: usize,
    show_modal: bool,
) -> Element<'a, Message> {
    guigroup
        .view(show_modal)
        .map(move |msg| Message::GUIGroup(i, msg))
}

#[derive(Serialize, Deserialize)]
pub struct SavedBlock {
    pub guigroups: Vec<SavedGUIGroup>,
}

impl SavedBlock {
    pub fn from_block(block: &BlockState, mut block_rule_opt: Option<BlockConfig>) -> Self {
        let guigroups = block
            .guigroups
            .iter()
            .map(|guigroup| {
                let block_rule_opt = match block_rule_opt {
                    Some(ref block_rule) if guigroup.group().id == block_rule.id => {
                        block_rule_opt.take()
                    }
                    _ => None,
                };

                SavedGUIGroup::from_guigroup(guigroup, block_rule_opt)
            })
            .collect();

        SavedBlock { guigroups }
    }
}

