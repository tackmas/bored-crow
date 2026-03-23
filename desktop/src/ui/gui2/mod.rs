// Modules
pub mod modal;

mod block;
mod settings;

// Dependencies
use iced::{
    self,
    Color,
    Element,
    Length,
    Task,
    widget::{
        Button,
        Column,
        center,
        Container,
        container,
        float,
        mouse_area,
        opaque,
        Row,
        Space,
        stack,
        Text,
    },
};
use screen_size as f_screen_size;

// Local
use block::{BlockState};
use settings::{SettingsMessage, SettingsState};

#[derive(Clone)]
enum Route<F, O = ()> {
    Forward(F),
    Open(O),
}

impl<F, O> Route<F, O> {
    fn handle_route<T>(
        self,
        forward: impl FnOnce(F) -> T,
        open: impl FnOnce(O) -> T
    ) -> T 
    {
        match self {
            Route::Forward(msg) => forward(msg),
            Route::Open(msg) => open(msg)
        } 
    }
}

#[derive(Clone)]
enum Message {
    Block(Route<block::Message>),
    Settings(Route<SettingsMessage>),
}

enum Screen {
    Block,
    Settings,
}

struct State {
    screen: Screen,
    modal: bool,
    block: BlockState,
    settings: SettingsState,
}

impl State {
    fn new() -> Self {
        let screen = Screen::Block;
        let modal = false;
        let block = BlockState::new();
        let settings = SettingsState::new();
  
        State { 
            screen, 
            modal,
            block, 
            settings,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Block(route) => {
                let forward = |msg| {
                    let action = self.block.update(msg);

                    match action {
                        block::Action::None => (),
                        block::Action::CloseModal => { self.modal = false; },
                        block::Action::OpenModal => { self.modal = true; },                       
                    };
                };

                route.handle_route(
                    forward, 
                    |_| self.screen = Screen::Block
                );
            },
            Message::Settings(route) => {
                route.handle_route(
                    |msg| self.settings.update(msg), 
                    |_| self.screen = Screen::Settings
                );
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let block = navigation_button("Block", Message::Block(Route::Open(())));

        let settings = navigation_button("Settings", Message::Settings(Route::Open(())));
            
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
                self.view_delegation()
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

        if self.modal {
            let modal_content = self.view_delegation();
            construct_modal(window_content, modal_content)               
        } else {
            window_content.into()
        }
    }

    fn view_delegation(&self) -> Element<'_, Message> {
        match &self.screen {
            Screen::Block => self.block
                .view()
                .map(|message| Message::Block(Route::Forward(message))),
            Screen::Settings => self.settings
                .view()
                .map(|message| Message::Settings(Route::Forward(message))),
            _ => unreachable!()
        }
    }
}

fn button_with_text<'a, M>(text_str: &'a str, message: M) -> Element<'a, M> 
where
    M: Clone + 'static
{
    let text = Text::new(text_str);
    let button = Button::new(text)
        .on_press(message);

    button.into()
}

#[derive(Clone)]
enum ModalRoute<O> {
    Ongoing(O),
    Close,
}

fn navigation_button(text_str: &'static str, msg: Message) -> Button<'_, Message> {
    Button::new(
        Text::new(text_str).center()
    )
    .on_press(msg)
    .width(Length::Fill)
    .height(Length::Fill)
}

fn construct_modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>
) -> Element<'a, Message> 
where
    Message: 'a + Clone
{
    let (screen_width, screen_height) = screen_size();
    let (width, height) = (screen_width / 3, screen_height / 2);

    let modal = opaque(
        mouse_area(
            center(
                opaque(
                    container(content)
                    .center_x(width)
                    .center_y(height)
                )
            )
            .style(|_theme|  {
                container::background(Color { a: 0.8, ..Color::BLACK})
            }),
        )
    );

    stack![base.into(), modal].into()
}

pub fn run() -> iced::Result {
    iced::application(State::new, State::update, State::view).run()
}

fn screen_size() -> (u32, u32) {
    let (f_w, f_h) = f_screen_size::get_primary_screen_size().unwrap();

    (f_w as u32, f_h as u32)
}

