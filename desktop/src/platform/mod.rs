use std::io::Result;

use crate::platform::linux::LinuxPlatform;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

pub trait App {
    fn name(&self) -> &String;
    fn is_blocked(&self) -> &bool;
    fn block(&mut self) -> Result<()>;
}

pub trait Platform {
    fn new() -> Self where Self: Sized;
    fn list_apps(&self) -> Vec<&impl App>;
    fn app(&self, name: &str) -> &impl App;
    fn list_blocked(&self) -> Vec<&String>;
}

pub struct App2 {
    name: String,
    is_blocked: bool
}

impl App2 {
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn is_blocked(&self) -> &bool {
        &self.is_blocked
    }
    pub fn block(&mut self) -> Result<()> {

    }
}





pub struct Platform2 {
    apps: Vec<App2>
}

impl Platform2 {
    pub fn new() -> Platform2 {
        let apps: Vec<App2> = get_app_names()
            .into_iter()
            .map(|name| App2 {
                 name,
                is_blocked: false})
            .collect();

        Platform2 { apps }
        
    }
    pub fn list_apps(&self) -> &Vec<App2> {
        &self.apps
    }
    pub fn app(&self, name: &str) -> Option<&App2> {
        self.apps
            .iter()
            .find(|&app| app.name == *name)
    }
    pub fn list_blocked(&self) -> Vec<&App2> {
        self.apps
            .iter()
            .filter(|app| app.is_blocked)
            .collect()
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


#[cfg(target_os = "linux")]
pub type CurrentPlatform = LinuxPlatform;

#[cfg(target_os = "windows")]
pub type CurrentPlatform = WindowsPlatform;


pub fn create_platform() -> impl Platform {
    #[cfg(target_os = "linux")]
    { LinuxPlatform::new() }

    #[cfg(target_os = "windows")]
    { WindowPlatform::new() }
}

