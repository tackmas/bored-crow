mod action;
mod block;
mod grid;
mod settings;
mod modal;

use std::io::Write;
use std::sync::LazyLock;
use std::thread;
use std::time::{Duration, Instant};

use iced::{self, Background, Color, Element, Length, Padding, Size, Subscription, Task, window};
use iced::alignment::{Horizontal, Vertical};
use iced::font::{Font, Weight};
use iced::widget::{
    Button, button, 
    Column, 
    Container, container, center,
    MouseArea, mouse_area,
    opaque,
    Row, row,
    Space, space, 
    Stack, stack,
    Text, text
};

use interprocess::local_socket::{ConnectOptions, GenericNamespaced, Stream, ToNsName};

use tokio::runtime::{Builder, Runtime};

use action::*;

use screen_size as f_screen_size;

use block::BlockState;

use desktop::{APP_NAME};

use desktop::group::{BlockConfig, Group};
use desktop::ipc::{IPCClientExt, Signal};
use desktop::platform::Blocker;
use desktop::saved::{Saved};

use self::modal::construct_modal;

use settings::{SettingsMessage, SettingsState};

fn main() {
    notify_startup();

    iced::application(Gui::new, Gui::update, Gui::view)
        .subscription(Gui::subscription)
        .exit_on_close_request(false)
        .run()
        .unwrap();
}

fn notify_startup() {
    let mut client = match Stream::create_client() {
        Ok(stream) => {
            println!("Daemon is already running");

            stream
        }
        Err(_) => {
            println!("Daemon is not currently running");

            run_daemon();

            let timeout = Duration::from_secs(10);
            let start = Instant::now();

            loop {
                thread::sleep(Duration::from_secs(2));

                if let Ok(stream) = Stream::create_client() {
                    break stream;
                } else if start.elapsed() > timeout {
                    panic!("Timed out waiting for daemon");
                }
            }
        }        
    };

    client.write_all(&[Signal::GUIStarted as u8]).unwrap();
}


cfg_select! {
    windows => { 
        fn run_daemon() {
            use std::env::current_exe;
            use std::os::windows::process::CommandExt;
            use std::process::Command;

            const DETACHED_PROCESS: u32 = 0x00000008;
            const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
            const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x01000000;

            let current_exe = current_exe().unwrap();
            let dir = current_exe.parent().unwrap();
            let daemon_exe_path = dir.join("daemon.exe");

            Command::new(daemon_exe_path)
                .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
                .spawn()
                .unwrap();
        }
    }
}

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
enum Message {
    ExitRequest,
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

struct Gui {
    blocker: Blocker,
    screen: Screen,
    modal: Option<Modal>,
    block: BlockState,
    settings: SettingsState,
}

impl Gui {
    fn new() -> Self {
        let runtime = Builder::new_current_thread()
            .build()
            .expect("ff15");

        runtime.block_on(
            Self::from_saved(Saved::load())
        )
    }

    async fn from_saved(saved: Saved) -> Self {
        let blocker = Blocker::new().await.unwrap();

        Self {
            blocker: blocker.clone(),
            screen: Screen::Block,
            modal: None,
            block: BlockState::from_saved(saved, &blocker).await,
            settings: SettingsState {  },
        }
    }
    fn to_saved_data(&self, block_config: Option<BlockConfig>) -> Saved {
        self.block.into_saved(block_config)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ExitRequest => {
                let stream = Stream::create_client()
                    .unwrap();

                stream.send_signal(Signal::GUIExited);

                iced::exit()
            }
            Message::Refresh => Task::none(),
            Message::Block(route) => {
                let mut forward = |msg| {
                    let action = self
                        .block
                        .update(msg)
                        .map_task(|msg| Message::Block(Route::Forward(msg)));

                    let mut block_config_opt = None;

                    let task = match action.custom_opt {
                        None => Task::none(),
                        Some(block::CA::Block(group, block_config)) => {
                            let blocker = self.blocker.clone();
                            block_config_opt = Some(block_config.clone());

                            Task::perform(
                                group.block(block_config, blocker),
                                |_| Message::Refresh
                            )
                        }
                    };

                    handle_modal_action(&mut self.modal, action.modal, || Modal::Block);

                    if action.save {
                        let saved = self.to_saved_data(block_config_opt);

                        let save_task = Task::future(async move {
                            Saved::async_save(&saved).await;
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

    fn view(&self) -> Element<'_, Message> {
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
    fn subscription(&self) -> Subscription<Message> {
        window::close_requests().map(|_| Message::ExitRequest)
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

// Roughly 5 pixels on a 1080/720 px screen
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

#[derive(Debug, Clone, Copy)]
struct IsTabActive(bool);

fn tab_button<'a, T, M>(
    button_appearance: impl Fn(IsTabActive) -> Element<'a, M>,
    current_selected_tab: T,
    this_tab: T,
    on_press: impl Fn() -> M + 'a
) -> Element<'a, M> 
where 
    M: 'a + Clone,
    T: Eq
{
    let is_tab_active = IsTabActive(this_tab == current_selected_tab);

    if is_tab_active.0 {
        button_appearance(is_tab_active)
    } else {
        Button::new(button_appearance(is_tab_active))
            .on_press_with(on_press)
            .padding(0)
            .into()
    }
}

fn tab_selection<'a, F, M, const N: usize, T>(
    button_appearances: [(F, T); N],
    current_selected_tab: T,
    on_press: impl Fn(T) -> M + 'a
) -> Element<'a, M> 
where 
    F: FnOnce(IsTabActive) -> Element<'a, M>,
    M: 'a + Clone,
    T: Eq
{   
    let tab_selection = button_appearances
        .into_iter()
        .map(|(button_appearance, this_tab)| {
            let is_tab_active = IsTabActive(this_tab == current_selected_tab);

            if is_tab_active.0 {
                button_appearance(is_tab_active)
            } else {
                button(button_appearance(is_tab_active))
                    .on_press(on_press(this_tab))
                    .into()
            }                  
        });

    row(tab_selection).into()
}

#[macro_export]
macro_rules! tab_selection {
    ($currently_selected_tab:expr, $on_press:expr, $(($button_appearance:expr, $this_tab:expr)),+) => {
        let len = 0;

       $(
            len += 1;

       ),+ 

       
    };
}

#[macro_export]
macro_rules! make_uninteractable {
    ($($element:expr),+; $into_type:ty) => {
        ( $({
            let semi_transparent_layer = iced::widget::container(
                iced::widget::space().width(Length::Fill).height(Length::Fill)
            )                           
            .style(|_theme| {
                iced::widget::container::Style {
                    background: Some(
                        iced::Background::Color(iced::Color {
                            a: 0.4,
                            ..iced::Color::WHITE
                        })
                    ),
                    ..iced::widget::container::Style::default()
                }
            });

            let opaque = iced::widget::opaque(semi_transparent_layer);

            let stack = iced::widget::stack![
                $element,
                opaque
            ];

            <$into_type>::from(stack)
        }),+ )
    };
}

/*
mod state;

use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};

use iced::{self, Element, Subscription, Task, window};
use iced::application::{BootFn};
use iced::widget::space;

use interprocess::local_socket::{ConnectOptions, GenericNamespaced, Stream, ToNsName};

use tokio::runtime::{Builder, Runtime};

use desktop::APP_NAME;

use desktop::ipc::{IPCClientExt, Signal};

use self::state::{Message as Message2, State};

impl BootFn<State, Message2> for fn(State) -> State {
    fn boot(&self) -> (State, Task<Message2>) {
        self()
    }
}

fn main() {
    notify_startup();

    iced::application(load_state, State::update, State::view)
        .subscription(State::subscription)
        .exit_on_close_request(false)
        .run()
        .unwrap();
}

fn load_state() -> State {
    let runtime = Builder::new_current_thread()
        .build()
        .expect("ff15");

    runtime.block_on(State::new())
}

enum Message {
    StateLoaded(State),
    State(state::Message),
    ExitRequest,
}

enum GUI {
    Unloaded,
    Loaded(State)
}

impl GUI {
    fn new() -> (Self, Task<Message>) {
        let state = Self::Unloaded;

        let task = Task::perform(State::new(), Message::StateLoaded);

        let stream = Stream::create_client()
            .unwrap_or_else(|_| {
                run_daemon();

                Stream::create_client().unwrap()
            });

        stream.send_signal(Signal::GUIStarted);

        (state, task)
    }
    fn update(&mut self, message: Message) -> Task<Message> {
        match (&mut *self, message) {
            (Self::Unloaded, Message::StateLoaded(state)) => {
                *self = Self::Loaded(state);

                Task::none()
            }
            (Self::Loaded(state), Message::State(msg)) => {
                let task = state.update(msg);

                task.map(Message::State)
            }
            (_, Message::ExitRequest) => {
                let stream = Stream::create_client().unwrap();

                stream.send_signal(Signal::GUIExited);

                iced::exit()
            }
            _ => Task::none(),
        }
    }
    fn view(&self) -> Element<'_, Message> {
        match self {
            Self::Unloaded => space().into(),
            Self::Loaded(data) => data.view().map(Message::State),
        }
    }
    fn subscription(&self) -> Subscription<Message> {
        window::close_requests().map(|_| Message::ExitRequest)
    }
}

fn send_ipc_message(message: u8) -> Result<Stream, ()> {
    use std::thread;
    use std::time::{Duration, Instant};

    let mut stream = match Stream::create_client() {
        Ok(stream) => {
            println!("Daemon is already running");

            stream
        }
        Err(_) => {
            println!("Daemon is not currently running");

            run_daemon();

            let timeout = Duration::from_secs(10);
            let start = Instant::now();

            loop {
                if let Ok(stream) = Stream::create_client() {
                    break stream;
                }

                if start.elapsed() > timeout {
                    panic!("Timed out waiting for daemon");
                }

                thread::sleep(Duration::from_secs(1));
            }
        }
    };

    stream.write_all(&[message]).unwrap();

    Ok(stream)
}

fn notify_startup() {
    let mut client = match Stream::create_client() {
        Ok(stream) => {
            println!("Daemon is already running");

            stream
        }
        Err(_) => {
            println!("Daemon is not currently running");

            run_daemon();

            let timeout = Duration::from_secs(10);
            let start = Instant::now();

            loop {
                thread::sleep(Duration::from_secs(2));

                if let Ok(stream) = Stream::create_client() {
                    break stream;
                } else if start.elapsed() > timeout {
                    panic!("Timed out waiting for daemon");
                }
            }
        }        
    };

    client.write_all(&[Signal::GUIStarted as u8]).unwrap();
}

cfg_select! {
    windows => { 
        fn run_daemon() {
            use std::env::current_exe;
            use std::os::windows::process::CommandExt;
            use std::process::Command;

            const DETACHED_PROCESS: u32 = 0x00000008;
            const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
            const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x01000000;

            let current_exe = current_exe().unwrap();
            let dir = current_exe.parent().unwrap();
            let daemon_exe_path = dir.join("daemon.exe");

            Command::new(daemon_exe_path)
                .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
                .spawn()
                .unwrap();
        }
    }
}

*/