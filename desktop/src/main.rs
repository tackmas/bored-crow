use std::thread;
use std::time::Duration;

use iced;
use tokio::runtime::Runtime;

mod platform;
mod ui;

use ui::gui;

fn main() {
    let runtime = Runtime::new().unwrap();
    let _guard = runtime.enter();
    
    let blocker = platform::Blocker::new();

    let apps = platform::App::all_apps().unwrap();

    ui::configg::display_applications(&apps);

    let i = apps.iter().position(|app| app.name() == "Discord").unwrap();

    blocker.block(&apps[i]).unwrap();


    // ui::configg::prompt_block_selection(&apps);

    gui::run().unwrap();

    thread::sleep(Duration::from_secs(20));
}

// ./target/x86_64-pc-windows-gnu/debug/desktop.exe