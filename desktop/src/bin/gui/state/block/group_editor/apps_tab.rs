
#[cfg(target_os = "windows")]
mod windows;

cfg_select! {
    windows => {
        use windows as platform;
    }
    _ => {
        unreachable!();
    }
}

use std::collections::HashSet;
use std::iter;
use std::ops::Range;

use bytes::Bytes;

use iced::{Element, Length, Padding};
use iced::alignment::{Vertical};
use iced::widget::{button, column, row, scrollable, Space, text};
use iced::widget::image::{Handle, Image};

use crate::make_uninteractable;
use crate::state::{self, action, IsTabActive, LENGTH_UNIT, Pad};

use super::{App};

pub struct CustomAction {

}

pub type CA = CustomAction;
pub type Action = action::Action<CA, Message>;

#[derive(Clone, Debug)]
pub enum Message {
    TabSelected(AppListTab),
    AddInstalledApp(App),
    RemoveSelectedApp { idx: usize }
}

pub struct AppsTab {
    active_tab: AppListTab,
    selected_apps: Vec<App>,
    incorporable_apps: IncorporableApps,
} 

impl AppsTab {
    pub fn new() -> Self {
        Self {
            active_tab: AppListTab::default(),
            selected_apps: Vec::new(),
            incorporable_apps: IncorporableApps::get()
        }
    }
}

impl AppsTab {
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::TabSelected(tab) => {
                self.active_tab = tab;

                Action::none()
            },
            Message::AddInstalledApp(app) => {
                if self.selected_apps.contains(&app) {
                    return Action::none();
                }
                self.incorporable_apps.set_app_is_selected_flag(&app, true);

                self.selected_apps.push(app);

                Action::none()
            },
            Message::RemoveSelectedApp { idx } => {
                let removed_app = self.selected_apps.remove(idx);
                self.incorporable_apps.set_app_is_selected_flag(&removed_app, false);

                Action::none()
            }
        }
    }
}

impl AppsTab {
    pub fn view(&self) -> Element<'_, Message> {
        let app_list_tab_selection_header = app_list_tab_selection_header(self.active_tab);

        let app_list_content = match self.active_tab {
            AppListTab::Installed => {
                let installed_apps = self.incorporable_apps.installed_apps();
                incorporable_apps_column(installed_apps)
            },
            AppListTab::Running => {
                let running_apps = self.incorporable_apps.running_apps();
                incorporable_apps_column(running_apps)
            }
        };

        let selected_apps = selected_apps(&self.selected_apps);

        column![
            app_list_tab_selection_header, 
            app_list_content, 
            selected_apps
        ].into()
    }
}

fn app_list_tab_selection_header<'a>(current_selected_tab: AppListTab) -> Element<'a, Message> {
    let installed_tab_button = app_list_tab_button("Installed", AppListTab::Installed, current_selected_tab);
    let running_tab_button = app_list_tab_button("Running", AppListTab::Running, current_selected_tab);

    let installed_tab_button = state::tab_button(
        |_is_tab_active| "Installed".into(),
        current_selected_tab, 
        AppListTab::Installed, 
        || Message::TabSelected(AppListTab::Installed)
    );

    let running_tab_button = state::tab_button(
        |_is_tab_active| "Running".into(),
        current_selected_tab, 
        AppListTab::Running, 
        || Message::TabSelected(AppListTab::Running)
    );

    row![installed_tab_button, running_tab_button].into()
}

fn app_list_tab_button(tab_name: &'static str, tab: AppListTab, current_selected_tab: AppListTab) -> Element<'_, Message> {
    let tab_button = button(tab_name).on_press(Message::TabSelected(tab));

    tab_button.into()
}

fn incorporable_apps_column(incorporable_apps: &[IncorporableApp]) -> Element<'_, Message> {
    let incorporable_apps = incorporable_apps
        .iter()
        .map(|incorporable_app| {
            display_incorporable_app(
                incorporable_app, "Add", || Message::AddInstalledApp(incorporable_app.inner.clone())
            )

        });

    scrollable(column(incorporable_apps))
        .spacing(0)
        .height(*LENGTH_UNIT * 25.0)
        .into()
}

fn display_incorporable_app<'a>(
    incorporable_app: &'a IncorporableApp, button_text: &'a str, on_select: impl Fn() -> Message + 'a
) -> Element<'a, Message> {
    let IncorporableApp { inner: ref app, is_selected } = *incorporable_app;
    let incorporable_app = display_selectable_app(app, button_text, on_select);

    if is_selected {
        make_uninteractable!(incorporable_app; Element<_>)
    } else {
        incorporable_app
    }
}

fn display_selectable_app<'a>(
    app: &'a App, button_str: &'a str, on_press: impl Fn() -> Message + 'a
) -> Element<'a, Message> {
    let icon = Image::new(app.icon.clone());

    let custom_text = |str| {
        text(str)
            .size(*LENGTH_UNIT * 1.5)
            .align_y(Vertical::Center)
            .height(*LENGTH_UNIT * 2.0)
    };

    let button = {
        let button_text = custom_text(button_str);

        button(button_text)
            .on_press_with(on_press)
            .padding(Padding::ZERO.horizontal(*LENGTH_UNIT / 2.0))
    };

    row![
        icon, 
        custom_text(&app.name), 
        Space::new().width(Length::Fill), 
        button
    ]
    .align_y(Vertical::Center)
    .spacing(*LENGTH_UNIT / 2.0)
    .padding(*LENGTH_UNIT / 2.0)
    .into()
} 

fn selected_apps(selected_apps: &[App]) -> Element<'_, Message> {
    let header = iter::once(Element::from("Selected apps"));

    let removable_apps = selected_apps
        .iter()
        .enumerate()
        .map(|(i, app)| display_selectable_app(app, "Remove", move || Message::RemoveSelectedApp { idx: i }));

    scrollable(column(header.chain(removable_apps)))
        .spacing(0)
        .into()
}

fn image(bytes: Bytes) -> Image {
    Image::new(Handle::from_bytes(bytes))
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum AppListTab {
    #[default]
    Installed,
    Running
}

// `apps` is split into two parts, where the first part apps[..`running_apps_offset`]
// is the list of installed apps on the computer, and the second part apps[`running_apps_offset`..]
// is the list of running apps
struct IncorporableApps {
    inner: Vec<IncorporableApp>,
    running_apps_offset: usize
}

impl IncorporableApps {
    fn get() -> Self {
        let installed_apps = platform::list_installed_apps();
        let running_apps_offset = installed_apps.len();

        let incorporable_apps = Self {
            inner: installed_apps,
            running_apps_offset
        };

        let mut incorporable_apps = platform::list_running_apps(incorporable_apps);

        incorporable_apps.inner[running_apps_offset..]
            .sort_by(|a, b| desktop::ordering_by_alphabetical(&a.inner.name, &b.inner.name));

        incorporable_apps.inner.shrink_to_fit();

        incorporable_apps
    }
    fn installed_apps(&self) -> &[IncorporableApp] {
        &self.inner[..self.running_apps_offset]
    }
    fn installed_apps_mut(&mut self) -> &mut [IncorporableApp] {
        &mut self.inner[..self.running_apps_offset]
    }
    fn running_apps(&self) -> &[IncorporableApp] {
        &self.inner[self.running_apps_offset..]
    }
    fn running_apps_mut(&mut self) -> &mut [IncorporableApp] {
        &mut self.inner[self.running_apps_offset..]
    }

    fn set_app_is_selected_flag(&mut self, app: &App, flag: bool){ 
        let set_app_is_selected_flag_in_slice = |apps: &mut [IncorporableApp]| {
            let incorporable_app_opt = apps
                .iter_mut()
                .find(|incorporable_app| incorporable_app.inner.exe_path == app.exe_path);

            if let Some(incorporable_app) = incorporable_app_opt {
                incorporable_app.is_selected = flag;
            }           
        };

        set_app_is_selected_flag_in_slice(self.installed_apps_mut());
        set_app_is_selected_flag_in_slice(self.running_apps_mut());
    }
}

struct IncorporableApp {
    inner: App,
    is_selected: bool,
}

impl IncorporableApp {
    fn new_with(app: App) -> Self {
        Self {
            inner: app,
            is_selected: false
        }
    }
}
