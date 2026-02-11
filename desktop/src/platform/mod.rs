use std::io::Result;
use std::sync::{Arc, Mutex};
use tokio::{
    runtime::Handle,
    sync::mpsc
};

mod blocker;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

pub struct Blocker<T> {
    sender: mpsc::Sender<T>,
    reciever: mpsc::Receiver<T>
}

#[derive(Debug)]
pub struct App {
    name: String,
    is_blocked: bool
}

impl App {
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn is_blocked(&self) -> &bool {
        &self.is_blocked
    }
    pub fn block(&mut self) {
        self.is_blocked = true;
    }
    pub fn unblock(&mut self) {
        self.is_blocked = false;
    }
}

#[derive(Debug)]
pub struct Platform {
    apps: Vec<App>
}

impl Platform {
    pub fn new(rt_handle: &Handle) -> Arc<Mutex<Platform>> {
        let apps: Vec<App> = get_app_names()
            .into_iter()
            .map(|name| {
                App {
                    name,
                    is_blocked: false
                }
            })
            .collect();

        let platform = Arc::new(Mutex::new(Platform { apps }));
        
        blocker::run(Arc::clone(&platform), rt_handle);

        platform
    }
    pub fn refresh(&mut self) {
        let apps: Vec<App> = get_app_names()
            .into_iter()
            .map(|name| {
                App {
                    name,
                    is_blocked: false
                }
            })
            .collect();

        self.apps = apps;
    }
    pub fn list_apps(&self) -> &Vec<App> {
        &self.apps
    }
    pub fn app(&self, name: &str) -> Option<&App> {
        self.apps
            .iter()
            .find(|&app| app.name == *name)
    }
    pub fn list_blocked(&self) -> impl Clone + Iterator<Item=&App> {
        self.apps
            .iter()
            .filter(|app| app.is_blocked)
    }
}

fn get_app_names() -> Vec<String> {
    #[cfg(target_os = "linux")]
    let names = linux::apps::run();

    #[cfg(target_os = "windows")]
    let names = windows::apps::run();

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    panic!("unsupported OS");

    names
}

