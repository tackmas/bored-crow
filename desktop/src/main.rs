use std::thread;
use std::time::Duration;

use tokio::runtime::Runtime;

use desktop::{
    ui::{
        self,
        gui2
    },
    platform
};

fn main() {
    let runtime = Runtime::new().unwrap();
    let _guard = runtime.enter();
    


    let apps = platform::App::all_apps().unwrap();

    ui::configg::display_applications(&apps);


    // ui::configg::prompt_block_selection(&apps);

    gui2::run().unwrap();
}

// ./target/x86_64-pc-windows-gnu/debug/desktop.exe