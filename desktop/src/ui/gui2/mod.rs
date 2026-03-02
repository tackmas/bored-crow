// Modules
mod block;
mod settings;

// Dependencies
use iced::{
    self,
    advanced::Widget,
    Element,
    Length,
    Vector,
    widget::{
        Button,
        Column,
        Container,
        container,
        Float,
        Row,
        Text,
    },
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
    settings: SettingsState,
}

impl State {
    fn new() -> Self {
        let screen = Screen::Block;
        let block = BlockState::new();
        let settings = SettingsState::new();
  
        State { 
            screen, 
            block, 
            settings,
        }
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


        let window_content = 
            Column::with_children([
                current_screen.into(),
                bottom_row.into()
            ])
            .width(Length::Fill)
            .height(Length::Fill);

        window_content.into()
    }
}

trait Modal<'a, E> 
where
    E: Clone + 'static 
{
    fn create_modal(self, exit_modal_event: E) -> Float<'a, E>;
}

impl<'a, E, W> Modal<'a, E> for W
where
    E: Clone + 'static,
    W: Into<Element<'a, E>> + Widget<E, iced::Theme, iced::Renderer>
{
    fn create_modal(self, exit_modal_event: E) -> Float<'a, E> {
        let (screen_width, screen_height) = screen_size();
        let (width, height) = (screen_width / 3, screen_height / 2);

        let title_bar = {
            Container::new(
                Button::new(
                    Text::new("✕").center()
                )
                .on_press(exit_modal_event)
            )
            .align_right(Length::Fill)
            .center_y(height / 15)
            .style(|_| {
                container::background(iced::Color::from_rgb(0.0, 0.0, 0.0))
            })
        };

        {
            let iced::Size { width, height } = self.size();

            assert!(
                (width, height) == (Length::Fill, Length::Fill),
                "the main modal component size must be {:?}", Length::Fill
            );
        }

        let content = {
            Column::with_children([
                title_bar.into(),
                self.into(),
            ])
            .width(width)
            .height(height)
        };

        let modal = Float::new(content)
            .translate(|content, viewport| {
                Vector {
                    x: (viewport.width - content.width) / 2.0,
                    y: (viewport.height - content.height) / 2.0
                }
            });

        modal
    }
}

pub fn run() -> iced::Result {
    iced::application(State::new, State::update, State::view).run()
}

fn screen_size() -> (u32, u32) {
    let (f_w, f_h) = f_screen_size::get_primary_screen_size().unwrap();

    (f_w as u32, f_h as u32)
}

