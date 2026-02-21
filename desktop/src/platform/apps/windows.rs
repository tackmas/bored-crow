use anyhow::Result;
use app_finder::{App as F_App, AppCommon, AppFinder};

use crate::platform::App;

pub fn apps() -> Result<Vec<App>> {
    let f_apps: Vec<F_App> = AppFinder::list();

    let apps = f_apps
        .into_iter()
        .map(|f_app| App { name: f_app.name })
        .collect();

    Ok(apps)
}



// x86_64-pc-windows-gnu