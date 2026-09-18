mod time;

// Std
use std::path::Path;
use std::sync::Arc;

// External
use futures::future;

use interprocess::local_socket::{GenericNamespaced, ListenerOptions, Stream, ToNsName};
use interprocess::local_socket::tokio::Listener;

use futures::StreamExt;

use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;
use tokio::task::LocalSet;

use windows::core::{BSTR, Error, HRESULT, w};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Registry::{
    HKEY, HKEY_LOCAL_MACHINE, RegNotifyChangeKeyValue, REG_NOTIFY_CHANGE_LAST_SET, RegOpenKeyW
};
use wmi::{Variant, WMIConnection, WMIError};

// Local
use desktop::{APP_NAME, can_uninstall};
use desktop::group::Group;
use desktop::saved::Saved;
use desktop::ipc::{Request, Response, Signal, ServerBound, IPCServerExt};
use desktop::platform::{App, Blocker};

use crate::Shutdown;

pub struct State {
    pub blocker: Blocker,
    pub groups: Vec<Arc<Group>>,
    pub uninstall_tx: mpsc::Sender<Shutdown>
}

impl State {
    pub async fn load(uninstall_tx: mpsc::Sender<Shutdown>) -> Self {
        let blocker = Blocker::new()
            .await
            .unwrap();
        let groups = load_groups_from_disk(&blocker).await;



        Self { 
            blocker, 
            groups,
            uninstall_tx
        }
    }
    pub async fn run(&mut self) {
        //std::thread::spawn(run_time_change_watcher);
        let local_set = LocalSet::new();

        local_set.run_until(async {
            tokio::task::spawn_local(async {
                // service_deletion_event_listener().await;
            });   

            self.ipc_listener().await;
        }).await;
    }

    async fn ipc_listener(&mut self) {
        let mut server = Listener::create_server();

        loop {
            let msg = server.recieve().await;

            match msg {
                ServerBound::Signal(signal) => {
                    let action = self.handle_signal(signal).await;

                    match action {
                        Action::None => (),
                        Action::Return => break
                    };
                },
                ServerBound::Request(request, response_token) => {
                    println!("Recieved request");

                    let response = self.handle_request(request).await;

                    response_token.respond(response).await;
                }
            }
        }
    }

    async fn handle_signal(&mut self, signal: Signal) -> Action {
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

                let is_any_group_blocked = self.groups
                    .iter()
                    .any(|group| group.is_blocked());

                if !is_any_group_blocked {
                    return Action::Return;
                }

                println!("Notify GUI shutdown");
            }
        }       

        Action::None
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
    let Some(saved): Option<Saved> = Saved::load() else {
        return Vec::new();
    };

    future::join_all(
        saved.groups
            .into_iter()
            .map(|saved_group| {
                Group::from_saved(saved_group, blocker)
            })
    ).await
}

enum Action {
    None,
    Return
}


async fn service_deletion_event_listener() -> Result<(), MyError> {
    let wmi_con = WMIConnection::with_namespace_path("root\\default").unwrap_or_else(|error| panic!("{:?}", MyError::from(error)));

    let registry_key_change_event_query = "SELECT * FROM RegistryKeyChangeEvent \
                                        WHERE Hive='HKEY_LOCAL_MACHINE' \
                                        AND KeyPath='SYSTEM\\\\CurrentControlSet\\\\Services\\\\Bored Crow'";

    let mut event_reciever = wmi_con.exec_notification_query_async(registry_key_change_event_query)
        .unwrap_or_else(|error| panic!("{:?}", MyError::from(error)));

    while let Some(event_res) = event_reciever.next().await {
        let event = event_res.unwrap_or_else(|error| panic!("{:?}", MyError::from(error)));

        let (std_reg_prov, get_security_descriptor)  = ("StdRegProv", "GetSecurityDescriptor");

        let in_params = wmi_con
            .get_object(std_reg_prov)
            .unwrap_or_else(|error| panic!("{:?}", MyError::from(error)))
            .get_method(get_security_descriptor)
            .unwrap_or_else(|error| panic!("{:?}", MyError::from(error)))
            .unwrap()
            .spawn_instance()
            .unwrap_or_else(|error| panic!("{:?}", MyError::from(error)));

        in_params.put_property("hDefKey", 2147483650u32).unwrap_or_else(|error| panic!("{:?}", MyError::from(error)));
        in_params.put_property("sSubKeyName", r"SYSTEM\\\\CurrentControlSet\\\\Services\\\\Bored Crow")
            .unwrap_or_else(|error| panic!("{:?}", MyError::from(error)));

        let out = wmi_con.exec_method(std_reg_prov, get_security_descriptor, Some(&in_params))
            .unwrap_or_else(|error| panic!("{:?}", MyError::from(error)))
            .unwrap();

        let variant = out.get_property("Descriptor")
            .unwrap_or_else(|error| panic!("{:?}", MyError::from(error)));

        let Variant::Object(security_descriptor) = variant else {
                panic!("{variant:?}");
            };

        let control_flags = match security_descriptor.get_property("ControlFlags").unwrap_or_else(|error| panic!("{:?}", MyError::from(error))) {
            Variant::UI4(v) => v,
            other => panic!("{other:?}"),
        };
            let se_dacl_present_true = 0b100;

        println!("SE_DACL_PRESENT: {}", control_flags & se_dacl_present_true);

        /* 

        // Set SE_DACL_PRESENT bit to true.
        let se_dacl_present_true = 0b100;
        security_descriptor.put_property("ControlFlags", control_flags | se_dacl_present_true);

        let a = match security_descriptor.get_property("DACL").unwrap() {
            Variant::Array(variants) => variants,
            other => panic!("{other:?}")
        };

        */
    }

    Ok(())
}

fn hklm_open_reg_key(reg_key_path: &'static str) -> Result<HKEY, Error> {
    let reg_key_path = Path::new(reg_key_path);
    let mut hkey = HKEY_LOCAL_MACHINE;
    
    for subkey in reg_key_path.iter() {
        let subkey = subkey
            .to_str()
            .expect("`subkey` was originally &str, so it is valid unicode");
        let lpsubkey = BSTR::from(subkey);

        let result = unsafe { RegOpenKeyW(hkey, &lpsubkey, &mut hkey) };
        
        if result.is_err() {
            return Err(Error::from(result));
        }
    }

    Ok(hkey)
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

#[macro_export]
macro_rules! hklm_open_reg_key {
    ($($key:expr),*) => {
        let mut hkey = HKEY_LOCAL_MACHINE;
        $(
            let lpsubkey = w!($key);

            RegOpenKeyW(hkey, lpsubkey, &raw mut hkey)
        ),*

    };
}


#[derive(Debug)]
pub enum MyError {
    HResult(String),
    WMINotHRESULTError(WMIError)
}

impl From<WMIError> for MyError {
    fn from(value: WMIError) -> Self {
        match value {
            WMIError::HResultError { hres } => {
                let h_result = HRESULT(hres);

                MyError::HResult(h_result.message())
            }
            other => MyError::WMINotHRESULTError(other)
        }
    }
}