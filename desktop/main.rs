mod action;
mod block;
mod id;
mod settings;
mod saved_state;


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
        mouse_area,
        opaque,
        Row,
        Space,
        stack,
        Text,
    },
};

use interprocess::{
    local_socket::{
        GenericNamespaced,
        ConnectOptions,
        Stream,
        ToNsName,
    }
};

pub use action::*;

use screen_size as f_screen_size;

use daemon::{
    SOCKET_NAME,
};

use utils::{
    unwrap_variant,
};

use block::{BlockState};
use daemon::{
    core::{
        block_config::{
            Group
        }
    },
    platform::{
        Blocker,
    }
};
use settings::{SettingsMessage, SettingsState};
use saved_state::{
    SavedState
};

#[macro_export]
macro_rules! child_modal {
    ($parent:expr, $child:path => $($variable:ident),+) => {
        match $parent {
            Some(m) => Some(unwrap_variant!(m, $child => $($variable),+)),
            None => None
        }
    };
}

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
    Refresh,
    Block(Route<block::Message>),
    Settings(Route<SettingsMessage>),
}

enum Modal {
    Block,
    Settings,
}

enum Screen {
    Block,
    Settings,
}

struct Data {
    stream: Stream,
    blocker: Blocker,
    screen: Screen,
    modal: Option<Modal>,
    block: BlockState,
    settings: SettingsState,    
}

impl Data {
    async fn new() -> Self {
        if let Some(saved) = state_disk::load::<SavedState>() {
            return Self::from_saved_state(saved).await;
        }

        Self {
            stream: new_stream(), 
            blocker: Blocker::new(),
            screen: Screen::Block, 
            modal: None,
            block: BlockState::new(), 
            settings: SettingsState::new(),
        }
    }

    async fn from_saved_state(saved_state: SavedState) -> Self {
        Data { 
            stream: new_stream(),
            blocker: Blocker::new(),
            screen: Screen::Block,
            modal: None,
            block: BlockState::from_saved(saved_state.block).await,
            settings: saved_state.settings.into_settings()
        }   
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Refresh => Task::none(),
            Message::Block(route) => {
                let mut forward = |msg| {
                    let action = self.block.update(msg);

                    let mut block_rule_opt = None;

                    let task = match action.custom_opt {
                        None => Task::none(),
                        Some(block::CustomAction::Block(group, block_rule)) => {
                            let blocker = self.blocker.clone();
                            block_rule_opt = Some(block_rule.clone());

                            Task::future(async move {
                                group.block(block_rule, blocker).await;

                                Message::Refresh
                            })
                        },
                        Some(block::CustomAction::Task(task)) => {
                            task.map(|msg| Message::Block(Route::Forward(msg)))
                        }                            
                    };

                    handle_modal_action(&mut self.modal, action.modal, || Modal::Block);

                    if action.save {
                        let saved = SavedState::from_state(&*self, block_rule_opt);

                        let save_task = Task::future(async move {
                            state_disk::async_save(&saved).await;
                        })
                        .discard();

                        return Task::batch([task, save_task]);
                    }

                    task
                };

                match route {
                    Route::Forward(msg) => forward(msg),
                    Route::Open(_) => {
                        self.screen = Screen::Block;

                        Task::none()
                    }
                }
            },
            Message::Settings(route) => {
                route.handle_route(
                    |msg| {
                        self.settings.update(msg);
                    }, 
                    |_| {
                        self.screen = Screen::Settings;
                    }
                );

                Task::none()
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
                self.view_delegation(false)
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

        if self.modal.is_some() {
            let modal_content = self.view_delegation(true);
            construct_modal(window_content, modal_content)               
        } else {
            window_content.into()
        }
    }

    fn view_delegation(&self, show_modal: bool) -> Element<'_, Message> {
        match &self.screen {
            Screen::Block => {    
                self.block
                    .view(show_modal) 
                    .map(|message| Message::Block(Route::Forward(message)))
            },
            Screen::Settings => {  
                self.settings
                    .view(show_modal)
                    .map(|message| Message::Settings(Route::Forward(message)))
            },
        }
    }    
}

enum StateMessage {
    Loaded(Data),
    Data(Message),
}

enum State {
    Loading,
    Ready(Data),
}

impl State {
    fn new() -> (Self, Task<StateMessage>) {
        (
            Self::Loading, 
            Task::perform(Data::new(), StateMessage::Loaded)
        )
    }
    fn update(&mut self, message: StateMessage) -> Task<StateMessage> {
        match (self, message) {
            (state @ State::Loading, StateMessage::Loaded(data)) => {
                *state = State::Ready(data);

                Task::none()
            },
            (State::Ready(data), StateMessage::Data(msg)) => {
                let task = data.update(msg);

                task.map(StateMessage::Data)
            }
            _ => Task::none()
        }
    }
    fn view(&self) -> Element<'_, StateMessage> {
        match self {
            State::Loading => {
                Space::new().into()
            },
            State::Ready(data) => {
                data.view().map(StateMessage::Data)
            }
        }
    }
}

fn handle_modal_action<M>(
    modal: &mut Option<M>, 
    modal_action: ModalAction,
    into_modal: impl FnOnce() -> M,
) {
    match modal_action {
        ModalAction::None => (),
        ModalAction::Close => *modal = None,
        ModalAction::Open => *modal = Some(into_modal()),
    }
}


fn button_with_text<'a, M>(text_str: &'a str, message: M) -> Element<'a, M> 
where
    M: Clone + 'a
{
    let text = Text::new(text_str);
    let button = Button::new(text)
        .on_press(message);

    button.into()
}


fn navigation_button(text_str: &'static str, msg: Message) -> Button<'static, Message> {
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
    let (width, height) = (screen_width * 2 / 5, screen_height / 2);

    let modal = opaque(
        mouse_area(
            center(
                opaque(
                    container(content)
                    .center_x(width)
                    .center_y(height)
                    .style(|_theme| {
                        container::background(Color::WHITE)
                    })
                )
            )
            .style(|_theme| {
                container::background(Color { a: 0.8, ..Color::BLACK})
            }),
        )
    );

    stack![base.into(), modal].into()
}

fn screen_size() -> (u32, u32) {
    let (f_w, f_h) = f_screen_size::get_primary_screen_size().unwrap();

    (f_w as u32, f_h as u32)
}


fn new_stream() -> Stream {
    let socket_name = SOCKET_NAME
        .to_ns_name::<GenericNamespaced>()
        .unwrap();

    ConnectOptions::new()
        .name(socket_name)
        .connect_sync()
        .unwrap()
}
        

fn main() {
    iced::application(State::new, State::update, State::view)
        .run()
        .unwrap();
}