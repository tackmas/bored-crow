use std::io::{
    Error,
    ErrorKind,
    Result
};

use crate::platform::*;
use freedesktop_file_parser::DesktopFile;

pub mod apps;
pub mod blocker;

pub struct LinuxApp {
    name: String,
    is_blocked: bool 
}

impl App for LinuxApp {
    fn name(&self) -> &String {
        &self.name
    }
    fn is_blocked(&self) -> &bool {
        // let app = self.find_app(name)?;
        // Ok(&app.is_blocked)
        &self.is_blocked
    }
    fn block(&mut self) -> Result<()> {
        //let app = self.find_app(name)?;
        blocker::run(&self);

        Ok(())
    }
}

pub struct LinuxPlatform {
    apps: Vec<LinuxApp>
}

impl Platform for LinuxPlatform {
    fn new() -> LinuxPlatform {
        let app_names: Vec<String> = apps::run();

        let apps: Vec<LinuxApp> = app_names
            .into_iter()
            .map(|name| LinuxApp {
                name, 
                is_blocked: false
            })
            .collect();

        LinuxPlatform { apps }
    }
    fn list_apps(&self) -> Vec<&

    fn list_blocked(&self) -> Vec<&String> {
        self.apps
            .iter()
            .filter(|app| app.is_blocked)
            .map(|app| &app.name)
            .collect()
    }
}

impl LinuxPlatform {
    fn find_app(&self, name: &String) -> Result<&App> {
        self.apps
            .iter()
            .find(|&app| app.name == *name)
            .ok_or(Error::from(ErrorKind::NotFound))
    }
}