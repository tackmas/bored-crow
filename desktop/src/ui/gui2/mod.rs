// Modules
mod block;
mod settings;

// Dependencies
use iced::{
    self,
    Element,
    Length,
    widget::{
        Button,
        Column,
        Container,
        Row,
        Text,
    }
};
use screen_size as f_screen_size;

// Local
use block::{BlockEvent, BlockState};
use settings::{SettingsEvent, SettingsState};


#[derive(Clone)]
enum Route<T> {
    Forward(T),
    Open,
}

#[derive(Clone)]
enum Event {
    Block(Route<BlockEvent>),
    Settings(Route<SettingsEvent>),
}

enum Screen {
    Block,
    Settings,
}

struct State {
    screen: Screen,
    block: BlockState,
    settings: SettingsState
}

impl State {
    fn new() -> Self {
        let screen = Screen::Block;
        let block = BlockState::new();
        let settings = SettingsState::new();
  
        State { screen, block, settings }
    }

    fn update(&mut self, event: Event) {
        match event {
            Event::Block(route) => {
                match route {
                    Route::Forward(event) => self.block.update(event),
                    Route::Open => self.screen = Screen::Block,
                }
            },
            Event::Settings(route) => {
                match route {
                    Route::Forward(event) => self.settings.update(event),
                    Route::Open => self.screen = Screen::Settings,
                }
            }
        };
    }

    fn view(&self) -> Element<'_, Event> {
        let block = 
            Button::new(
                Text::new("Block").center()
            )
            .on_press(Event::Block(Route::Open))
            .width(Length::Fill)
            .height(Length::Fill);

        let settings = 
            Button::new(
                Text::new("Settings").center()
            )
            .on_press(Event::Settings(Route::Open))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(0);
            

        let (_, screen_height) = screen_size();

        let bottom_row = 
            Row::with_children([
                block.into(),
                settings.into()
            ])
            .width(Length::Fill)
            .height(screen_height / 20);

        let current_screen = 
            Container::new(
                match &self.screen {
                    Screen::Block => self.block.view().map(|event| Event::Block(Route::Forward(event))),
                    Screen::Settings => self.settings.view().map(|event| Event::Settings(Route::Forward(event))) 
                }
            )
            .width(Length::Fill)
            .height(Length::Fill);


        let column = 
            Column::with_children([
                current_screen.into(),
                bottom_row.into()
            ]);

        column.into()
    }
}

pub fn run() -> iced::Result {
    iced::application(State::new, State::update, State::view).run()
}

fn screen_size() -> (u32, u32) {
    let (f_w, f_h) = f_screen_size::get_primary_screen_size().unwrap();

    (f_w as u32, f_h as u32)
}
