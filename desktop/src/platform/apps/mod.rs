use anyhow::Result;

use crate::platform::App;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(not(target_os = "windows"))]
mod rest;


pub fn apps() -> Result<Vec<App>> {
    #[cfg(target_os = "windows")]
    {
        windows::apps()
    }

    #[cfg(not(target_os = "windows"))]
    { 
        rest::apps()
    }
}