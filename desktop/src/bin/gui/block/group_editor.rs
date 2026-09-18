mod apps_tab;
mod websites_tab;

// STD
use std::cmp::Ordering;
use std::{collections::HashSet, sync::Arc};
use std::hash::{Hash, Hasher};

// Dependencies
use bytes::Bytes;

use iced::advanced::text::{Fragment, IntoFragment};
use iced::{
    self, Background, Color, Element, Length, Task,
    alignment::Vertical,
        widget::{
        Button, Checkbox, Column, Row, Scrollable, Space, Text, text, TextInput, center_x, column,
        container, row, space,
    },
};
use iced::widget::image::{Handle, Image};

use sysinfo::{Process, ProcessRefreshKind, RefreshKind, System, UpdateKind};

use tokio::time::{Duration, sleep};

// Local
use desktop::impl_deref_mut_for_newtype;
use desktop::platform::ProcessName;
use crate::{Action as BaseAction, LENGTH_UNIT};

use self::apps_tab::{AppsTab, self as a_t};

use super::guigroup::GUIGroup;


pub enum CustomAction {
    Close,
    Save,
}

pub type CA = CustomAction;
pub type Action = BaseAction<CA, Message>;

#[derive(Clone, Copy)]
enum UIError {
    EmptyName,
    GroupNameAlreadyExists,
    NoProcessSelected,
}

#[derive(Clone)]
pub enum Message {
    Close,
    Save,
    AppsTab(a_t::Message),
    TabSelected(Tab),
    GroupNameInput(String),
    DisplayError(UIError),
    HideError,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum Tab {
    #[default]
    Apps,
    Websites
}


pub struct GroupEditor {
    group_name: String,
    selected_tab: Tab,
    apps_tab: AppsTab,
    error: Option<UIError>,
}

impl GroupEditor {
    pub fn new() -> Self {
        Self {
            group_name: String::new(),
            selected_tab: Tab::default(),
            apps_tab: AppsTab::new(),
            error: None
        }
    }
    pub fn from(group_name: String, selected_processes: &Vec<ProcessName>) -> Self {

        todo!()
    }

    pub fn into_parts(self) -> (String, Vec<ProcessName>) {

        todo!()
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
                    return handle_display_error(&mut self.error, UIError::GroupNameAlreadyExists);
                }

                /*

                if self.selected_process_names.is_empty() {
                    return handle_display_error(&mut self.error, UIError::NoProcessSelected);
                }

                */

                Action::none_with_custom(CA::Save)
            }
            Message::AppsTab(apps_tab_msg) => {
                let _action = self.apps_tab.update(apps_tab_msg);

                Action::none()
            }
            Message::TabSelected(selected_tab) => {
                self.selected_tab = selected_tab;

                Action::none()
            }
            Message::GroupNameInput(input) => {
                self.group_name = input;
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
        let group_name = self.group_name();
        let tab_selection = self.tab_selection();
        let apps_tab_content = self.apps_tab_content();

        let centered_x = {
            let x_space = || space().width(Length::FillPortion(1));
            let content = column![group_name, tab_selection, apps_tab_content]
                .width(Length::FillPortion(8));

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
    fn group_name(&self) -> Element<'_, Message> {
        let group_name_header = text("Group name")
            .size(*LENGTH_UNIT);

        let group_name_input = TextInput::new("Group name: ", &self.group_name)
            .on_input(Message::GroupNameInput)
            .width(Length::Fill);

        column![group_name_header, group_name_input].into()
    }

    fn tab_selection(&self) -> Element<'_, Message> {
        let current_selected_tab = self.selected_tab;

        let apps_tab_button = tab_button("Apps", Tab::Apps, current_selected_tab);
        let websites_tab_button = tab_button("Websites", Tab::Websites, current_selected_tab);

        row![apps_tab_button, websites_tab_button].into()
    }

    fn apps_tab_content(&self) -> Element<'_, Message> {
        self.apps_tab.view().map(Message::AppsTab)
    }

    /*
    fn app_list(&self) -> Element<'_, Message> {
        let rows = self.all_processes
            .into_iter()
            .map(|process_info| {
                let selected_process_name_index = self.selected_process_names
                    .iter()
                    .position(|selected_pn| *selected_pn == process_info.name);

                let checkbox = {
                    let process_name = process_info.name.clone();

                    Checkbox::new(selected_process_name_index.is_some())
                        .on_toggle(move |toggled| match toggled {
                            true => Message::Select(process_name.clone()),
                            false => Message::Unselect{ idx: selected_process_name_index.unwrap() },
                        })
                };

                let text = Text::new(process_info.name.to_string());

                Row::with_children([
                    checkbox.into(), 
                    text.into(), 
                    process_info.logo.into()
                ])
                .align_y(Vertical::Center)
                .into()
            });

        let column = Column::with_children(rows);

        let scrollable = Scrollable::new(column)
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill);

        scrollable.into()
    }
    */

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
        UIError::GroupNameAlreadyExists => "Name already is taken by another group",
        UIError::NoProcessSelected => "No selected apps",
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

fn tab_button<'a>(
    tab_name: &'static str,
    tab: Tab, 
    current_selected_tab: Tab, 
) -> Element<'a, Message> {
    let tab_button = Button::new(tab_name).on_press(Message::TabSelected(tab));
    let tab_is_selected_indicator = container(space())
        .height(*LENGTH_UNIT / 5.0)
        .style(move |_theme| {
            use container::Style;

            let is_tab_currently_selected = current_selected_tab == tab;

            let indicator = if is_tab_currently_selected {
                Color::WHITE
            } else {
                let mut transperent = Color::WHITE;
                let full_transperent = 1.0;
                // a field is transperency scale
                transperent.a = full_transperent;

                transperent
            };

            Style {
                background: Some(Background::Color(indicator)),
                ..Style::default()
            }
        });

    column![tab_button, tab_is_selected_indicator]
        .width(Length::Shrink)
        .into()
}

// REFACTOR ME: Return a default image instead of panicking/unwrapping
fn process_logo(process: &Process) -> Image {
    let exe_file_path = match process.exe() {
        Some(exe_file_path) => exe_file_path.to_string_lossy(),
        None => panic!("Process name: {:?}", process.name())
    };

    let logo_in_bytes = systemicons::get_icon(&exe_file_path, 16)
        .unwrap();

    let handle = Handle::from_bytes(logo_in_bytes);

    Image::new(handle)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct App {
    name: String,
    exe_path: String,
    icon: Handle
}

impl App {
    fn from(name: String, exe_path: String, icon: Handle) -> Self {
        Self {
            name,
            exe_path,
            icon
        }
    }
}

#[derive(Debug)]
struct Website {
    name: String
}

struct ProcessInfo {
    logo: Image,
    name: ProcessName
}

impl From<(Image, ProcessName)> for ProcessInfo {
    fn from(value: (Image, ProcessName)) -> Self {
        Self {
            logo: value.0,
            name: value.1
        }
    }
}

impl Hash for ProcessInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for ProcessInfo {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for ProcessInfo {}

// Expensive function
fn all_processes() -> Vec<ProcessInfo> {
    let refresh_kind = {
        let process_refresh_kind = ProcessRefreshKind::nothing()
            .with_exe(UpdateKind::Always);

        RefreshKind::nothing()
            .with_processes(process_refresh_kind)
    };

    let mut all_processes: Vec<_> = System::new_with_specifics(refresh_kind)
        .unwrap()
        .processes()
        .values()
        .filter_map(|process| {
            let process_name = process.name();

            if process_name == "[System Process]" && cfg!(windows) {
                return None;
            }

            let process_logo = process_logo(process);
            let process_name = ProcessName::from_os_str(process_name);

            Some(ProcessInfo::from((process_logo, process_name)))
        }
        )
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    all_processes.sort_by(|a, b| ordering_by_alphabetical(&a.name, &b.name));

    all_processes
}

fn ordering_by_alphabetical(a: &str, b: &str) -> Ordering {
    let (mut a_bytes, mut b_bytes) = (a.bytes(), b.bytes());
    loop {
        match (a_bytes.next(), b_bytes.next()) {
            (Some(a_byte), Some(b_byte)) => {
                let case_insensitive_ordering = a_byte.to_ascii_lowercase()
                    .cmp(&b_byte.to_ascii_lowercase());

                if case_insensitive_ordering != Ordering::Equal {
                    return case_insensitive_ordering;
                } 

                let case_sensitive_ordering = b_byte.cmp(&a_byte);

                if case_sensitive_ordering != Ordering::Equal {
                    return case_sensitive_ordering;
                }
            },
            (Some(_), None) => return Ordering::Greater,
            (None, Some(_)) => return Ordering::Less,
            (None, None) => return Ordering::Equal
        } 
    }
}

