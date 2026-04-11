use anyhow::Result;

use applications::{AppInfo, AppInfoContext};

use super::App;

pub fn apps() -> Result<Vec<App>> {
    let mut ctx = AppInfoContext::new(Vec::new());
    ctx.refresh_apps().unwrap();

    let apps = ctx
        .get_all_apps()
        .into_iter()
        .map(|a| App {name: a.name})
        .collect();

    Ok(apps)
}