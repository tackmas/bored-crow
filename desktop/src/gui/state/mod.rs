mod action;
mod block;
mod grid;
pub mod saved_data;
mod settings;
mod modal;

pub use saved_data::SavedData;

pub use block::SavedBlock;
pub use settings::SavedSettings;

use std::io::Write;
use std::sync::LazyLock;

use iced::{self, Background, Color, Element, Length, Padding, Size, Subscription, Task, window};
use iced::alignment::{Horizontal, Vertical};
use iced::font::{Font, Weight};
use iced::widget::{
    Button, 
    Column, 
    Container, container, center,
    MouseArea, mouse_area,
    opaque,
    Row, 
    Space, space, 
    Stack, stack,
    Text, text
};

use interprocess::local_socket::{ConnectOptions, GenericNamespaced, Stream, ToNsName};

pub use action::*;

use screen_size as f_screen_size;

use block::BlockState;

use crate::{APP_NAME};

use crate::{
    core::block::{BlockConfig, Group},
    platform::Blocker,
};

use self::modal::construct_modal;

use settings::{SettingsMessage, SettingsState};

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
    fn handle_route<T>(self, forward: impl FnOnce(F) -> T, open: impl FnOnce(O) -> T) -> T {
        match self {
            Route::Forward(msg) => forward(msg),
            Route::Open(msg) => open(msg),
        }
    }
}

#[derive(Clone)]
pub(super) enum Message {
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

pub struct State {
    blocker: Blocker,
    screen: Screen,
    modal: Option<Modal>,
    block: BlockState,
    settings: SettingsState,
}

impl State {
    pub(super) async fn new() -> Self {
        if let Some(saved) = SavedData::load() {
            return Self::from_saved_data(saved).await;
        }

        *SCREEN_SIZE;

        let blocker = Blocker::new().await.unwrap();

        Self {
            blocker,
            screen: Screen::Block,
            modal: None,
            block: BlockState::new(),
            settings: SettingsState::new(),
        }
    }

    async fn from_saved_data(saved_state: SavedData) -> Self {
        let blocker = Blocker::new().await.unwrap();

        Self {
            blocker: blocker.clone(),
            screen: Screen::Block,
            modal: None,
            block: BlockState::from_saved(saved_state.block, &blocker).await,
            settings: saved_state.settings.into_settings(),
        }
    }
    pub fn into_saved_data(&self, block_rule: Option<BlockConfig>) -> SavedData {
        SavedData {
            block: SavedBlock::from_block(&self.block, block_rule),
            settings: SavedSettings::from_settings(&self.settings),
        }
    }

    pub(super) fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Refresh => Task::none(),
            Message::Block(route) => {
                let mut forward = |msg| {
                    let action = self
                        .block
                        .update(msg)
                        .map_task(|msg| Message::Block(Route::Forward(msg)));

                    let mut block_rule_opt = None;

                    let task = match action.custom_opt {
                        None => Task::none(),
                        Some(block::CA::Block(group, block_rule)) => {
                            let blocker = self.blocker.clone();
                            block_rule_opt = Some(block_rule.clone());

                            Task::perform(
                                group.block(block_rule, blocker),
                                |_| Message::Refresh
                            )
                        }
                    };

                    handle_modal_action(&mut self.modal, action.modal, || Modal::Block);

                    if action.save {
                        let saved = SavedData::from_state(&*self, block_rule_opt);

                        let save_task = Task::future(async move {
                            SavedData::async_save(&saved).await;
                        })
                        .discard();

                        return Task::batch([task, save_task]);
                    }

                    Task::batch([task, action.task])
                };

                match route {
                    Route::Forward(msg) => forward(msg),
                    Route::Open(_) => {
                        self.screen = Screen::Block;

                        Task::none()
                    }
                }
            }
            Message::Settings(route) => {
                route.handle_route(
                    |msg| {
                        self.settings.update(msg);
                    },
                    |_| {
                        self.screen = Screen::Settings;
                    },
                );

                Task::none()
            }
        }
    }

    pub(super) fn view(&self) -> Element<'_, Message> {
        let block = navigation_button("Block", Message::Block(Route::Open(())));

        let settings = navigation_button("Settings", Message::Settings(Route::Open(())));

        let Size { height: screen_height, .. } = *SCREEN_SIZE;

        let bottom_row = Row::with_children([block.into(), settings.into()])
            .width(Length::Fill)
            .height(screen_height / 20);

        let current_screen = Container::new(self.view_delegation(false))
            .width(Length::Fill)
            .height(Length::Fill);

        let all_content = Column::with_children([current_screen.into(), bottom_row.into()])
            .width(Length::Fill)
            .height(Length::Fill);

        if self.modal.is_some() {
            let modal_content = self.view_delegation(true);
            construct_modal(all_content, modal_content)
        } else {
            all_content.into()
        }
    }

    fn view_delegation(&self, show_modal: bool) -> Element<'_, Message> {
        match &self.screen {
            Screen::Block => self
                .block
                .view(show_modal)
                .map(|message| Message::Block(Route::Forward(message))),
            Screen::Settings => self
                .settings
                .view(show_modal)
                .map(|message| Message::Settings(Route::Forward(message))),
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
    M: Clone + 'a,
{
    let text = Text::new(text_str);
    let button = Button::new(text).on_press(message);

    button.into()
}

fn navigation_button(text_str: &'static str, msg: Message) -> Button<'static, Message> {
    Button::new(Text::new(text_str).center())
        .on_press(msg)
        .width(Length::Fill)
        .height(Length::Fill)
}

fn semi_bold_text<'a>(text_str: &'a str) -> Text<'a> {
    text(text_str)
        .font(Font {
            weight: Weight::Semibold,
            ..Font::DEFAULT
        })
}

fn bold_text<'a>(text_str: &'a str) -> Text<'a> {
    text(text_str)
        .font(Font {
            weight: Weight::Bold,
            ..Font::DEFAULT
        })
}

static SCREEN_SIZE: LazyLock<Size<u32>> = LazyLock::new(|| {
    let (w, h) = f_screen_size::get_primary_screen_size().unwrap();

    Size {
        width: w as u32,
        height: h as u32
    }
});

static LENGTH_UNIT: LazyLock<f32> = LazyLock::new(|| {
    SCREEN_SIZE.width as f32 / 200.0
});

trait RedBackground<'a, M>: Into<Element<'a, M>> {
    fn red_background(self) -> Container<'a, M> {
        container(self).style(|_theme| {
            use container::Style;

            Style {
                background: Some(Background::Color(Color::from_rgba8(255, 0, 0, 0.7))),
                ..Style::default()
            }
        })
    }
}

impl<'a, M, T> RedBackground<'a, M> for T 
where 
    T: Into<Element<'a, M>>
{}

const DARK_BEIGE: Color = Color::from_rgb8(205, 170, 125);

trait Pad<'a, M>: Into<Element<'a, M>> + Sized {
    fn pad(self, padding: impl Into<Padding>) -> Container<'a, M> {
        container(self).padding(padding)
    }
    fn pad_x(self, padding: f32) -> Container<'a, M> {
        self.pad([0.0, padding])
    }
    fn pad_y(self, padding: f32) -> Container<'a, M> {
        self.pad([padding, 0.0])
    }
} 

impl<'a, M, T> Pad<'a, M> for T 
where 
    T: Into<Element<'a, M>>
{}



#[macro_export]
macro_rules! make_semi_transparent {
    ($($element:expr),+; $into_type:ty) => {
        ( $({
            let semi_transparent_layer = container(
                space().width(Length::Fill).height(Length::Fill)
            )                           
            .style(|_theme| {
                use container::Style;

                Style {
                    background: Some(
                        Background::Color(Color {
                            a: 0.4,
                            ..Color::WHITE
                        })
                    ),
                    ..Style::default()
                }
            });

            let opaque = opaque(semi_transparent_layer);

            let stack = stack![
                $element,
                opaque
            ];

            <$into_type>::from(stack)
        }),+ )
    };
}

