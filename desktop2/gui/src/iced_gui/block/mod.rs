// mod block_rules;
mod manage_guigroup;
mod guigroup;

use std::{
    sync::{
        Arc
    }
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
    widget::{
        Button,
        Column,
        Container,
        Row,
        Scrollable,
        Space,
        Text,
    },
    Task,
};

use serde::{
    Serialize,
    Deserialize
};

// Local

use utils::unwrap_variant;

use platform::{
    App, 
    Blocker,
};

use crate::iced_gui::{
    self,
    Action,
    Route, 
};


use guigroup::{
    GUIGroup
};
use manage_guigroup as m_gg;

pub enum Instruction<B: FnOnce(Blocker) -> Task<Message>> {
    None,
    Block(B),
    Task(Task<Message>),
}

#[derive(Clone)]
pub enum Message {
    NewGUIGroup(Route<manage_guigroup::Message>),
    GUIGroup(usize, guigroup::Message),
}

pub enum Modal {
    GUIGroup(usize),
    NewGUIGroup(manage_guigroup::ManageGroup)
}

pub struct BlockState {
    modal: Option<Modal>,
    guigroups: Vec<GUIGroup>,
    blocker: Blocker,
    all_apps: Arc<[App]>
}

impl BlockState {
    pub fn new() -> Self {
        BlockState { 
            modal: None,
            guigroups: Vec::new(),
            blocker: Blocker::new(), 
            all_apps: App::all_apps().unwrap().into(),
        }
    }

    pub fn from_saved(saved: Saved) -> Self {
        let all_apps: Arc<[App]> = App::all_apps()
            .unwrap()
            .into();

        let blocker = Blocker::new();

        let guigroups = saved.guigroups
            .into_iter()
            .map(|saved| GUIGroup::from_saved(saved, all_apps.clone(), blocker.clone()))
            .collect();

        BlockState {
            modal: None,
            guigroups,
            blocker,
            all_apps,
        }
    }

    pub fn update(
        &mut self, 
        show_modal: bool, 
        message: Message
    ) -> Action<Instruction>
    {
        match message {
            Message::NewGUIGroup(route) => {
                match (&mut self.modal, route) {
                    (Some(Modal::NewGUIGroup(new_guigroup)), Route::Forward(msg)) => {
                        let instruction = new_guigroup.update(msg);

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
                                let new_guigroup = unwrap_variant!(modal, Modal::NewGUIGroup => new_guigroup);

                                let guigroup = GUIGroup::from_manage_group(blocker, new_guigroup);

                                self.guigroups.push(guigroup);

                                Action::save(Instruction::CloseModal(Task::none()))
                            },
                        }
                    
                    },
                    (None, Route::Open(())) => {
                        let all_apps = self.all_apps.clone();
                        let new_guigroup = m_gg::ManageGroup::from(all_apps);
                        self.modal = Some(Modal::NewGUIGroup(new_guigroup));

                        Action::no_save(Instruction::OpenModal)
                    },
                    _ => unreachable!()
                }
            }

            Message::GUIGroup(i, msg) => {
                let action = self.guigroups[i].update(show_modal, msg);

                action.map_instruction(|ins| 
                    match action.instruction {
                        guigroup::Instruction::None => Instruction::None,
                        guigroup::Instruction::Delete => {
                            self.guigroups.remove(i); 
                            Instruction::None
                        },
                        guigroup::Instruction::CloseModal(task) => {
                            Instruction::CloseModal(task.map(move |msg| Message::GUIGroup(i, msg)))
                        },
                        guigroup::Instruction::OpenModal => {
                            self.modal = Some(Modal::GUIGroup(i));

                            Instruction::OpenModal
                        }
                        guigroup::Instruction::Task(task) => {
                            Instruction::Task(task.map(move |msg| Message::GUIGroup(i, msg)))
                        }
                    }
                )
            }
        }
    }   

    pub fn view(&self, show_modal: bool) -> Element<'_, Message> {
        if show_modal {
            return self.modal();
        }
        let first_row = self.first_row();
        let guigroup_list = self.guigroup_list();

        let column = 
            Column::with_children([
                first_row,
                guigroup_list
            ])
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
            },
            Some(Modal::NewGUIGroup(new_guigroup)) => {
                new_guigroup
                    .view()
                    .map(|msg| Message::NewGUIGroup(Route::Forward(msg)))
            },
            _ => unreachable!()
        }
    }


    fn first_row(&self) -> Element<'_, Message> {
        let (_, screen_height) = iced_gui::screen_size();

        let title = 
            Container::new(
                Text::new("App Blocks")
                .size(25)
            )
            .center_x(FillPortion(3))
            .center_y(Fill);

        let new_guigroup_button = 
            Container::new(
                Button::new(
                    Text::new("New Group")
                    .size(15)
                )
                .on_press(Message::NewGUIGroup(Route::Open(())))
            )
            .center_x(FillPortion(2))
            .center_y(Fill);

        let space = Space::new()
            .width(FillPortion(8));

        let first_row = 
            Container::new(
                Row::with_children([
                    title.into(), 
                    space.into(),
                    new_guigroup_button.into()
                ])
            )
            .center_x(Fill)
            .center_y(screen_height / 14);

        first_row.into()
    }

    fn guigroup_list(&self) -> Element<'_, Message> {
        if self.guigroups.is_empty() { 
            return Container::new(
                Text::new("No groups currently exist")
            )
            .center(Fill)
            .into()
        }

        let guigroups_element: Vec<_> = self.guigroups
            .iter()
            .enumerate()
            .map(|(i, guigroup)| {
                guigroup_element(guigroup, i, false)
            })
            .collect();

        let guigroup_list = 
            Column::with_children(
                guigroups_element
            )
            .width(Fill)
            .align_x(Center)
            .spacing(10);

        Scrollable::new(guigroup_list)
            .spacing(0)
            .into()
    }    
}

fn guigroup_element<'a>(
    guigroup: &'a guigroup::GUIGroup, 
    i: usize, 
    show_modal: bool
) -> Element<'a, Message> {
    guigroup
        .view(show_modal)
        .map(move |msg| Message::GUIGroup(i, msg))
}

#[derive(Serialize, Deserialize)]
pub struct Saved {
    guigroups: Vec<guigroup::Saved>,
}

impl Saved {
    pub fn from_block(block: &BlockState) -> Self {
        todo!()
    }

}
