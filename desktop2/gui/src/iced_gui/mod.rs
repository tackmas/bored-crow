mod action;
mod block;
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

pub use action::{
    Action,
    ModalAction,
};

use screen_size as f_screen_size;

use utils::{
    unwrap_variant,
};

use block::{BlockState};
use platform::{
    Blocker,
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

enum SaveAction {
    Save,
    Skip,
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

struct State {
    blocker: Blocker,

    screen: Screen,
    modal: Option<Modal>,
    block: BlockState,
    settings: SettingsState,
}

impl State {
    fn new() -> Self {
        if let Some(saved) = state_disk::load::<SavedState>() {
            return State::from_saved_state(saved);
        }

        let screen = Screen::Block;
        let modal = None;
        let block = BlockState::new();
        let settings = SettingsState::new();
  
        State { 
            blocker: Blocker::new(),
            screen, 
            modal,
            block, 
            settings,
        }
    }

    fn from_saved_state(saved_state: SavedState) -> Self {
        State { 
            blocker: Blocker::new(),
            screen: Screen::Block,
            modal: None,
            block: BlockState::from_saved(saved_state.block),
            settings: saved_state.settings.into_settings()
        }   
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Block(route) => {
                let mut forward = |msg| {
                    let action = self.block.update(self.modal.is_some(), msg);

                    let task = match action.instruction {
                        block::Instruction::None => Task::none(),
                        block::Instruction::CloseModal(task) => {
                            self.modal = None; 
                            
                            task.map(|msg| Message::Block(Route::Forward(msg)))
                        },
                        block::Instruction::OpenModal => {
                            self.modal = Some(Modal::Block);

                            Task::none()
                        },
                        block::Instruction::Task(task) => {
                            task.map(|msg| Message::Block(Route::Forward(msg)))
                        }                            
                    };

                    handle_modal_action(&mut self.modal, action.modal_action, || Modal::Block);

                    if action.save {
                        let saved = SavedState::from_state(&*self);

                        return task.chain(
                            Task::future(async move {
                                state_disk::async_save(&saved).await;
                            })
                            .discard()
                        );
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

fn handle_modal_action(
    modal: &mut Option<Modal>, 
    modal_action: ModalAction,
    into_modal: impl FnOnce() -> Modal,
) {
    match modal_action {
        ModalAction::None => (),
        ModalAction::Close => *current_modal = None,
        ModalAction::Open => *current_modal = Some(into_modal),
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

pub fn run() -> iced::Result {
    iced::application(State::new, State::update, State::view).run()
}
