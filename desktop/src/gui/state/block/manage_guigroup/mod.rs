// STD
use std::{collections::HashSet, sync::Arc};

// Dependencies
use iced::{
    self, Color, Element, Length, Task,
    alignment::Vertical,
        widget::{
        Button, Checkbox, Column, Row, Scrollable, Space, Text, text, TextInput, center_x, column,
        container, row, space,
    },
};

use tokio::time::{Duration, sleep};

// Local
use crate::gui::state::action;
use crate::platform::App;

use super::guigroup::GUIGroup;

pub enum CustomAction {
    Close,
    Save,
}

pub type CA = CustomAction;
pub type Action = action::Action<CA, Message>;

#[derive(Clone, Copy)]
enum UIError {
    EmptyName,
    NameAlreadyExists,
    NoAppsSelected,
}

#[derive(Clone)]
pub enum Message {
    Close,
    Save,

    Select(usize),
    Unselect(usize),
    GroupNameInput(String),
    DisplayError(UIError),
    HideError,
}

pub struct ManageGroup {
    group_name: String,
    all_apps: Arc<[App]>,
    selected: HashSet<usize>,
    error: Option<UIError>,
}

impl ManageGroup {
    pub fn from_guigroup(guigroup: &GUIGroup) -> Self {
        let group_name = guigroup.name().clone();
        let group = guigroup.group();

        let all_apps = group.all_apps.clone();
        let selected: HashSet<usize> = group.apps_i.iter().copied().collect();

        ManageGroup {
            group_name,
            all_apps,
            selected,
            error: None,
        }
    }

    pub fn into_parts(self) -> (String, Arc<[App]>, HashSet<usize>) {
        let ManageGroup {
            group_name,
            all_apps,
            selected,
            ..
        } = self;

        (group_name, all_apps, selected)
    }
    // all_other_names are all group names except self
    #[must_use]
    pub fn update<'a>(
        &mut self,
        message: Message,
        mut all_other_names: impl Iterator<Item = &'a String>,
    ) -> Action {
        match message {
            Message::Close => Action::none_with_custom(CA::Close),
            Message::Save => {
                if self.group_name.is_empty() {
                    return handle_display_error(&mut self.error, UIError::EmptyName);
                }
                let name_already_exists = all_other_names.any(|name| *name == self.group_name);

                if name_already_exists {
                    return handle_display_error(&mut self.error, UIError::NameAlreadyExists);
                }

                if self.selected.is_empty() {
                    return handle_display_error(&mut self.error, UIError::NoAppsSelected);
                }

                Action::none_with_custom(CA::Save)
            }
            Message::GroupNameInput(input) => {
                self.group_name = input;
                Action::none()
            }
            Message::Select(apps_i) => {
                self.selected.insert(apps_i);
                Action::none()
            }
            Message::Unselect(apps_i) => {
                self.selected.remove(&apps_i);
                Action::none()
            }
            Message::DisplayError(ui_error) => {
                self.error = Some(ui_error);

                Action::none().with_task(error_task())
            }
            Message::HideError => {
                self.error = None;

                Action::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let group_name = self.get_group_name();
        let app_list = self.app_list();

        let centered_x = {
            let x_space = || space().width(Length::FillPortion(1));
            let content = column![group_name, app_list].width(Length::FillPortion(8));

            row![x_space(), content, x_space()]
        };

        let bottom = {
            let row = if let Some(e) = &self.error {
                row![error_text(e)]
            } else {
                row![]
            };

            let row = row
                .extend([self.close_button(), self.save_button(), Space::new().into()])
                .spacing(20);

            container(row)
                .align_right(Length::Fill)
                .align_y(Vertical::Center)
        };

        let content = column![
            centered_x.height(Length::FillPortion(6)),
            bottom.height(Length::FillPortion(1))
        ];

        content.into()
    }
    fn get_group_name(&self) -> Element<'_, Message> {
        TextInput::new("Group name: ", &self.group_name)
            .on_input(|input| Message::GroupNameInput(input))
            .width(Length::Fill)
            .into()
    }

    fn app_list(&self) -> Element<'_, Message> {
        let rows = self.all_apps.iter().enumerate().map(|(i, app)| {
            let is_selected = self.selected.contains(&i);

            let checkbox = Checkbox::new(is_selected).on_toggle(move |toggled| match toggled {
                true => Message::Select(i),
                false => Message::Unselect(i),
            });

            let text = Text::new(app.name());

            Row::with_children([checkbox.into(), text.into()]).into()
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

        let save_button = Button::new(text).on_press(Message::Save);

        save_button.into()
    }

    fn close_button(&self) -> Element<'_, Message> {
        let text = Text::new("Close without saving");

        let close_button = Button::new(text).on_press(Message::Close);

        close_button.into()
    }
}

fn error_text(error: &UIError) -> Element<'_, Message> {
    let text_str = match error {
        UIError::EmptyName => "Name is empty; must enter a unique name",
        UIError::NameAlreadyExists => "Name already is taken by another group",
        UIError::NoAppsSelected => "No selected apps",
    };

    text(text_str).color(Color::from_rgb(1.0, 0.0, 0.0)).into()
}

fn handle_display_error(current_error: &mut Option<UIError>, new_error: UIError) -> Action {
    *current_error = Some(new_error);

    return Action::none().with_task(error_task());
}

fn error_task() -> Task<Message> {
    let sleep = sleep(Duration::from_secs(3));

    Task::perform(sleep, move |_| Message::HideError)
}

impl From<Arc<[App]>> for ManageGroup {
    fn from(from: Arc<[App]>) -> Self {
        ManageGroup {
            group_name: String::new(),
            all_apps: from,
            selected: HashSet::new(),
            error: None,
        }
    }
}
