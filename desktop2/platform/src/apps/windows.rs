
use winreg::{
    enums::{
        HKEY_CURRENT_USER,
        HKEY_LOCAL_MACHINE,
    },
    reg_key::{
        HKEY,
        RegKey,
    }
};

use crate::App;

pub fn apps() -> Vec<App> {
    // let mut apps: Vec<App> = app_finder::AppFinder::list().into_iter().map(|app| App { name: app.name }).collect();
    let mut apps = list_apps();

    apps.sort_by(|a, b| a.name.cmp(&b.name) );
    apps.dedup_by(|a, b| a.name == b.name);

    apps
}

// x86_64-pc-windows-gnu

fn list_apps() -> Vec<App> {
    let mut apps = Vec::new();

    apps_in_hive(HKEY_LOCAL_MACHINE, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall", &mut apps);
    apps_in_hive(HKEY_LOCAL_MACHINE, "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall", &mut apps);
    apps_in_hive(HKEY_CURRENT_USER, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall", &mut apps);
    apps_in_hive(HKEY_CURRENT_USER, "SOFTWARE\\Wow6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall", &mut apps);

    apps
}

fn apps_in_hive(hive: HKEY, path: &str, apps: &mut Vec<App>) {
    let hive_key = RegKey::predef(hive);

    let Ok(software_key) = hive_key.open_subkey(path) else { return };

    let hive_apps = software_key.enum_keys().filter_map(|key| {
        let key = key.ok()?;
        let app_key = software_key.open_subkey(&key).ok()?;

        let name = app_key.get_value::<String, _>("DisplayName").ok()?;

        if name.is_empty() {
            return None;
        }

        Some(App { name })
    });

    apps.extend(hive_apps);
}