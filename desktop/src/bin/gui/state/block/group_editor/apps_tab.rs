
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

use bytes::Bytes;

use iced::{Element, Length};
use iced::widget::{button, column, row, scrollable, Space, text};
use iced::widget::image::{Handle, Image};

use crate::make_uninteractable;
use crate::state::{action};

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

    scrollable(column(incorporable_apps)).into()
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
    app: &'a App, button_text: &'a str, on_select: impl Fn() -> Message + 'a
) -> Element<'a, Message> {
    let icon = image(app.icon.clone());
    let button = button(button_text)
        .on_press_with(on_select);

    row![
        icon, 
        text(&app.name), 
        Space::new().width(Length::Fill), 
        button
    ].into()
} 

fn selected_apps(selected_apps: &[App]) -> Element<'_, Message> {
    let header = iter::once(Element::from("Selected"));

    let removable_apps = selected_apps
        .iter()
        .enumerate()
        .map(|(i, app)| display_selectable_app(app, "Remove", move || Message::RemoveSelectedApp { idx: i }));

    scrollable(
        column(header.chain(removable_apps))
    ).into()
}

fn image(bytes: Bytes) -> Image {
    Image::new(Handle::from_bytes(bytes))
}

#[derive(Clone, Copy, Debug, Default)]
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

        let mut incorporable_apps = Self {
            inner: installed_apps,
            running_apps_offset
        };

        platform::list_running_apps(&mut incorporable_apps);

        incorporable_apps.inner.shrink_to_fit();

        incorporable_apps
    }
    fn installed_apps(&self) -> &[IncorporableApp] {
        &self.inner[..self.running_apps_offset]
    }
    fn running_apps(&self) -> &[IncorporableApp] {
        &self.inner[self.running_apps_offset..]
    }
    // `app` must exist inside `self`, otherwise the function will panic
    fn set_app_is_selected_flag(&mut self, app: &App, flag: bool){ 
        self.inner
            .iter_mut()
            .find(|incorporable_app| incorporable_app.inner == *app)
            .unwrap()
            .is_selected = flag
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
