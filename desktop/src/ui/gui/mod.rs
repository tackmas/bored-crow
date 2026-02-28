use iced::{
    self, 
    Element, 
    Length,
    widget::{
        self, button, Column, column, Container, Row, row, scrollable, Space, Stack, text
    },
};


use screen_size as f_screen_size;

use crate::platform::{App, Blocker};

mod modal;

use modal::*;

#[derive(Clone, Debug)]
enum Message {
    Main,
    CloseModal,
    OpenModal(OpenModal),
    Modal(ModalMsg)
}

struct State {
    blocker: Blocker,
    screen: Screen,
}


enum Screen {
    Main,
    Modal(ModalKind)
}

impl State {
    fn new() -> State {
        let blocker = Blocker::new();


        State { blocker, screen: Screen::Main }
    } 
    fn update(&mut self, message: Message) {
        match message {
            Message::Main => self.screen = Screen::Main,
            Message::CloseModal => self.screen = Screen::Main,
            Message::OpenModal(modal) => {
                self.screen = Screen::Modal(ModalKind::new(modal));
            }
            Message::Modal(msg) => {
                if let Screen::Modal(modal) = &mut self.screen {
                    modal.update(msg);
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let main_layout = Container::new(
            Column::with_children([
                button(text("App list")).on_press(Message::OpenModal(OpenModal::AppList)).into(),
            ]))
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        match &self.screen {
            Screen::Main => main_layout,
            Screen::Modal(modal) => {
                Stack::with_children([
                    main_layout,
                    modal.view().map(Message::Modal)
                ]).into()
            }
        }
        .into()
    }



}


pub fn run() -> iced::Result {
    iced::application(State::new, State::update, State::view).run()
}

fn screen_size() -> (u32, u32) {
    let (d_w, d_h) = f_screen_size::get_primary_screen_size().unwrap();

    (d_w as u32, d_h as u32)
}