use sysinfo::System;
use tokio::runtime;

mod core;
mod ui;
mod platform;

#[cfg(feature = "error")]
mod error;

fn main() {
    let _s = System::new_all();
    let _runtime = runtime::Builder::new_current_thread().build().unwrap();

    let platform = platform::create_platform();
    ui::config::display_applications(&platform);

    let blocked_input = ui::config::get_input(&platform);

    for app in &blocked_input {
        println!("{:?}", app);
    }

    /*runtime.block_on(async {
        check_processes(&s, &blocked).await;
        
    });*/

}