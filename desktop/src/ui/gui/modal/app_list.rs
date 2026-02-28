use std::collections::HashSet;

use iced::{Length, Alignment};
use iced::widget::{Checkbox, Container, Float, Scrollable, Text};

use crate::gui::*;
use super::ModalInfo;

#[derive(Clone, Debug)]
pub enum AppListMsg {
    Block(App),
    Unblock(App)
}

#[derive(Clone, Debug)]
pub struct AppList {
    apps: Vec<App>,
    selected: HashSet<App>
}

impl AppList {
    pub fn new() -> AppList {
        let apps = App::all_apps().unwrap();
        let selected = HashSet::new();

        AppList { apps, selected }
    }

    pub fn view(&self, info: ModalInfo) -> Element<'_, AppListMsg> {
        let rows = self.apps
            .iter()
            .map(|app| {   
                let is_selected = self.selected.contains(app);

                let checkbox = Checkbox::new(is_selected).on_toggle(|toggled| {
                    match toggled {
                        true => AppListMsg::Block(app.clone()),
                        false => AppListMsg::Unblock(app.clone())
                    }
                });

                let text = Text::new(app.name());

                row![
                    checkbox,
                    text,
                ]
                .into()
            });

        let column: Column<'_ , AppListMsg> = Column::with_children(rows);

        let scrollable = Scrollable::new(column)
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill);

        let float = Float::new(scrollable);

        float.into()
    }

    pub fn update(&mut self, message: AppListMsg) {
        match message {
            AppListMsg::Block(app) => { self.selected.insert(app); },
            AppListMsg::Unblock(app) => {
                let a = self.selected.remove(&app);
            }
        };   

        println!{"{:?}", self.selected};
    }
}

