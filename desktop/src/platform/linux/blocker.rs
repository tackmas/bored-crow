use std::{
    io::{
        Write,
        ErrorKind
    },
    fs,
    path,
    env,
    thread
};
use sysinfo::Process;
use freedesktop_file_parser::DesktopFile;
use tokio;
use tokio_stream::{self, StreamExt};
use super::App;

pub fn run(app: &App) {
    let name = &app.name;

    create_desktop_entry(&name);
}

fn detect_blocked_apps() {
    
} 


fn create_desktop_entry(file_name: &String) {
    let home = env::var("HOME").expect("$HOME enviroment variable is invalid or not set");

    let directory = format!("{}.local/share/applications", home);
    fs::create_dir_all(&directory).expect("Failed to create \"~/.local/share/applications\" directory.");

    let file_path = format!("{directory}{file_name:?}.desktop");

    let mut file = fs::File::create(file_path).unwrap();

    let content = format!{
"# This file is created by Bored Crow to override desktop entries in order to block them.
[Desktop Entry]
Type=Application
Name=Bored Crow
NoDisplay=True
Hidden=True
Exec=/usr/local/bin/ {file_name}_name_arg
"};

    if let Err(e) = file.write_all(content.as_bytes()) {
        if e.kind() != ErrorKind::Interrupted {
            panic!("Failed to write to {file:?}");
        }
    }
}

async fn check_for_desktop_entry() {
    loop {
        
    }
}

