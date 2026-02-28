use iced::{
    Background,
    Color,
    Length
};


use iced::widget::Button;
use iced::widget::container;


mod app_list;

use super::*;

use app_list::*;

pub struct ModalInfo {
    pub width: u32,
    pub height: u32
}

#[derive(Clone, Debug)]
pub enum OpenModal {
    AppList
}

#[derive(Clone, Debug)]
pub enum ModalMsg {
    Close,

    AppListMsg(AppListMsg),
}

pub enum ModalKind {
    AppList(AppList)
}

impl ModalKind {
    pub fn new(modal: OpenModal) -> ModalKind {
        match modal {
            OpenModal::AppList => ModalKind::AppList(AppList::new())
        }
    }

    pub fn update(&mut self, message: ModalMsg) {
        match (self, message) {
            (_, ModalMsg::Close) => (),
            (ModalKind::AppList(state), ModalMsg::AppListMsg(msg)) => state.update(msg)
        }
    }

    pub fn view(&self) -> Element<'_, ModalMsg> {
        let (screen_width, screen_height) = crate::gui::screen_size();
        let (width, height) = (screen_width / 3, screen_height / 2);

        let info = ModalInfo { width, height };

        let content = match self {
            ModalKind::AppList(state) => state.view(info).map(ModalMsg::AppListMsg) 
        };

        let top = Container::new(
            Button::new(
                text("✕").center()
            )
            .height(Length::Fill)
            .on_press(ModalMsg::Close)
        )
        .align_right(Length::Fill)
        .center_y(width / 20)
        .style(|_| {
            container::Style {
                background: Some(Background::Color(Color::from_rgb(0.0, 0.0, 0.0))),
                ..Default::default()
            }
        })
        .into();

        let complete = Column::with_children([
            top,
            content
        ])
        .width(width)
        .height(height);

        let screen_container = Container::new(complete)
            .center(Length::Fill)
            .into();

        screen_container
    }
}