mod time;

// Std
use std::sync::Arc;

// External
use chrono::Local;

use futures::future;

use interprocess::local_socket::{GenericNamespaced, ListenerOptions, Stream, ToNsName};
use interprocess::local_socket::tokio::Listener;

use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;

// Local
use desktop::{APP_NAME, can_uninstall};
use desktop::core::block::Group;
use desktop::gui::saved_data::SavedData;
use desktop::ipc::{Request, Response, Signal, ServerBound, IPCServerExt};
use desktop::platform::{App, Blocker};

use crate::Shutdown;

pub struct State {
    pub time_zone: Local,
    pub blocker: Blocker,
    pub groups: Vec<Arc<Group>>,
    pub uninstall_tx: mpsc::Sender<Shutdown>
}

impl State {
    pub async fn load(uninstall_tx: mpsc::Sender<Shutdown>) -> Self {
        let time_zone = Local::now().timezone();
        let blocker = Blocker::new()
            .await
            .unwrap();
        let groups = load_groups_from_disk(&blocker).await;

        Self { 
            time_zone,
            blocker, 
            groups,
            uninstall_tx
        }
    }
    pub async fn run(&mut self) {
        //std::thread::spawn(run_time_change_watcher);

        self.ipc_listener().await;
    }

    async fn ipc_listener(&mut self) {
        let mut server = Listener::create_server();

        loop {
            let msg = server.recieve().await;

            match msg {
                ServerBound::Signal(signal) => self.handle_signal(signal).await,
                ServerBound::Request(request, response_token) => {
                    println!("Recieved request");

                    let response = self.handle_request(request).await;

                    response_token.respond(response).await;
                }
            }
        }
    }

    async fn handle_signal(&mut self, signal: Signal) {
        match signal {
            Signal::GUIStarted => {
                for g in &self.groups {
                    g.force_unblock();
                }
                self.groups.clear();

                println!("Notify GUI startup");
            },
            Signal::GUIExited => {
                self.groups = load_groups_from_disk(&self.blocker).await;
                println!("Notify GUI shutdown");
            }
        }       
    }

    async fn handle_request(&mut self, request: Request) -> Response {
         match request {
            Request::CanUninstall => {
                println!("Attempting to handle request: 'Request::CanUninstall'");

                let response = async || {
                    if can_uninstall(){
                        let is_ok = self.uninstall_tx
                            .send(Shutdown::Uninstall)
                            .await
                            .is_ok();

                        if is_ok {
                            return Response::CanUninstall;
                        }
                    }

                    Response::CanNotUninstall
                };

                let desktop_is_running = self.groups.is_empty();

                let response = if desktop_is_running {
                    self.groups = load_groups_from_disk(&self.blocker).await;
                    let response = response().await;
                    self.idle();

                    response
                } else {
                    response().await
                };

                println!("Handled request: 'Request::CanUninstall'");

                response
            }
        }       
    }
    
    fn idle(&mut self) {
        for g in &self.groups {
            g.force_unblock();
        }

        self.groups.clear();
    } 

}


pub async fn load_groups_from_disk(blocker: &Blocker) -> Vec<Arc<Group>> {
    let Some(saved): Option<SavedData> = SavedData::load() else {
        return Vec::new();
    };

    let all_apps: Arc<[App]> = App::all_apps()
        .unwrap()
        .into();

    let groups = future::join_all(
        saved.block.guigroups
            .into_iter()
            .map(|s_guigroup| {
                let all_apps = all_apps.clone();
                Group::from_saved(s_guigroup.group, all_apps, &blocker)
            })
    ).await;

    groups
}


use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW,
    DefWindowProcW,
    DispatchMessageW,
    GetMessageW,
    MSG,
    RegisterClassW,
    TranslateMessage,
    WINDOW_EX_STYLE, 
    WINDOW_STYLE, 
    WNDCLASSW,
    WM_TIMECHANGE
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::core::w;

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_TIMECHANGE {
        // handle it: re-read local time, notify your logic, etc.
        println!("time change detected");
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

pub fn run_time_change_watcher() {
    unsafe {
        let hinstance = GetModuleHandleW(None).unwrap().into();
        let class_name = w!("Bored Crow Time Watcher");

        let wc = WNDCLASSW {
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinstance,
            lpszClassName: class_name,
            ..Default::default()
        };
        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            None,
            WINDOW_STYLE::default(), // no WS_VISIBLE
            0, 0, 0, 0,
            None,
            None,
            Some(hinstance),
            None,
        ).unwrap();

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, Some(hwnd), 0, 0).into() {
            TranslateMessage(&msg).ok().unwrap();
            DispatchMessageW(&msg);
        }
    }
}