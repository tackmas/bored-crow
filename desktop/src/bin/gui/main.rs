mod state;

use std::io::Write;

use iced::{self, Element, Subscription, Task, window};
use iced::widget::space;

use interprocess::local_socket::{ConnectOptions, GenericNamespaced, Stream, ToNsName};

use desktop::APP_NAME;

use desktop::ipc::{IPCClientExt, Signal};

pub use self::state::State;

pub fn main() {
    iced::application(GUI::new, GUI::update, GUI::view)
        .subscription(GUI::subscription)
        .exit_on_close_request(false)
        .run()
        .unwrap();
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
                .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP | CREATE_BREAKAWAY_FROM_JOB)
                .spawn()
                .unwrap();
        }
    }
}