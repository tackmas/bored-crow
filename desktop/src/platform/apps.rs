use std::{
    io,
    collections::HashMap, 
    fs, 
    path::PathBuf,
};
use freedesktop_file_parser::DesktopFile;
use super::App;

pub fn run() -> io::Result<Vec<App>> {
    get_apps()
}


fn get_apps() -> io::Result<Vec<App>> {
    let mut directories = HashMap::new();

    directories.insert("system", PathBuf::from("/usr/share/applications/"));
    directories.insert("user", PathBuf::from("~/.local/share/applications/"));
    directories.insert("flatpak", PathBuf::from("/var/lib/flatpak/exports/share/applications/"));
    directories.insert("snap", PathBuf::from("/var/lib/snapd/desktop/applications/"));
    
    /* let directories = [
        PathBuf::from("/usr/share/applications/"),
        PathBuf::from("~/.local/share/applications/"),
        PathBuf::from("/var/lib/flatpak/exports/share/applications/"),
        PathBuf::from("/var/lib/snapd/desktop/applications/")
    ]; */

    let mut apps = Vec::new();

    for (_scope, dir) in directories {
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let extension = path.extension();

            if extension != Some("desktop".as_ref()) {
                continue;
            }

            let desktop_file = parse_to_desktop(&entry);

            if is_hidden_or_nodisplay(&desktop_file) {
                continue;
            }

            let app = into_app(desktop_file);

          //  lowercase_app_names(&mut desktop_file);

            apps.push(app);
        }
    }
    Ok(apps)
}

fn parse_to_desktop(file_entry: &fs::DirEntry) -> DesktopFile {
    let path = file_entry.path();
    let file_content = fs::read_to_string(&path).unwrap();
    let desktop_file = match freedesktop_file_parser::parse(&file_content) {
        Ok(file) => file,
        Err(e) => panic!("{e}; File name: {:?}; File path: {path:?}; File contents are invalid", file_entry.file_name())
    };

    desktop_file
}

fn is_hidden_or_nodisplay(desktop_file: &DesktopFile) -> bool {
    let entry = &desktop_file.entry;

    if entry.hidden == Some(true) || entry.no_display == Some(true) {
        true
    } else {
        false
    }
}

fn into_app(desktop_file: DesktopFile) -> App {
    App { 
        name: desktop_file.entry.name.default,
        is_blocked: false
    }
}
