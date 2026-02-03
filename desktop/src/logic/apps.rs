use std::{
    io,
    collections::HashMap, 
    fs, 
    path::PathBuf,
};
use freedesktop_file_parser::DesktopFile;

pub fn get_desktop_files() -> io::Result<Vec<DesktopFile>> {
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

    let mut desktop_files = Vec::new();

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

            let mut desktop_file = parse_to_desktop(&entry);

            if is_hidden_or_nodisplay(&desktop_file) {
                continue;
            }

            lowercase_app_names(&mut desktop_file);

            desktop_files.push(desktop_file);
        }
    }
    Ok(desktop_files)
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

fn lowercase_app_names(desktop_file: &mut DesktopFile) {
    let name = &mut desktop_file.entry.name.default;

    name.make_ascii_lowercase();
}