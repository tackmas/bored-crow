// mod block_rules;
mod manage_guigroup;
mod guigroup;

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
};

// Local
use crate::{
    unwrap_variant,
    platform::{
        App, 
        Blocker,
    },
    ui::gui2::{
        Route, 
    },
};

#[derive(Clone)]
pub enum Message {
    NewGUIGroup(Route<manage_guigroup::Message>),
    GUIGroup(usize, guigroup::Message),
}

pub enum Action {
    None,
    CloseModal,
    OpenModal,
}

pub enum Modal {
    GUIGroup(usize, guigroup::Modal),
    NewGUIGroup(manage_guigroup::ManageGroup)
}

pub struct BlockState {
    modal: Option<Modal>,
    guigroups: Vec<guigroup::GUIGroup>,
    blocker: Blocker,
}

impl BlockState {
    pub fn new() -> Self {
        BlockState { 
            modal: None,
            guigroups: Vec::new(),
            blocker: Blocker::new(), 
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::NewGUIGroup(route) => {
                match (&mut self.modal, route) {
                    (Some(Modal::NewGUIGroup(new_guigroup)), Route::Forward(msg)) => {
                        let action = new_guigroup.update(msg);

                        match action {
                            manage_guigroup::Action::None => Action::None,
                            manage_guigroup::Action::Close => {
                                self.modal = None;
                                Action::CloseModal
                            },
                            manage_guigroup::Action::Save => {
                                let modal = self.modal
                                    .take()
                                    .unwrap();

                                let new_guigroup = unwrap_variant!(modal, Modal::NewGUIGroup);
                                
                                let (name, apps) = new_guigroup.into_name_and_apps();
                                let apps = apps
                                    .into_iter()
                                    .collect();

                                let group = crate::core::block_config::Group::new_with(apps);
                                let blocker = self.blocker.clone();

                                let guigroup = guigroup::GUIGroup::from((name, group, blocker));

                                self.guigroups.push(guigroup);

                                Action::CloseModal
                            },
                        }
                    },
                    (None, Route::Open(())) => {
                        let new_guigroup = manage_guigroup::ManageGroup::new();
                        self.modal = Some(Modal::NewGUIGroup(new_guigroup));

                        Action::OpenModal
                    },
                    _ => unreachable!()
                }
            }

            Message::GUIGroup(i, msg) => {
                let guigroups = &mut self.guigroups;

                let action = guigroups[i].update(msg);

                match action {
                    guigroup::Action::None => Action::None,
                    guigroup::Action::Delete => {
                        guigroups.remove(i); 
                        
                        Action::None
                    },
                    guigroup::Action::CloseModal => Action::CloseModal,
                    guigroup::Action::OpenModal => Action::OpenModal,
                };

                Action::None
            }
        }
    }   

    pub fn view(&self) -> Element<'_, Message> {
        if let Some(m) = &self.modal {
            return self.modal(m);
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
    fn modal<'a>(&'a self, modal: &'a Modal) -> Element<'a, Message> {
        match modal {
            Modal::GUIGroup(i, m) => {
                let i = *i;

                self.guigroups[i]
                    .view()
                    .map(move |msg| Message::GUIGroup(i, msg))
            },
            Modal::NewGUIGroup(new_guigroup) => new_guigroup
                .view()
                .map(|msg| Message::NewGUIGroup(Route::Forward(msg)))
        }
    }


    fn first_row(&self) -> Element<'_, Message> {
        let (_, screen_height) = crate::gui2::screen_size();

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
                guigroup.view().map(move |msg| Message::GUIGroup(i, msg))
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
