use sysinfo::System;
use tokio::runtime;

mod logic;
mod ui;

#[cfg(feature = "error")]
mod error;

fn main() {
    let _s = System::new_all();
    let _runtime = runtime::Builder::new_current_thread().build().unwrap();

    let desktop_files = logic::apps::get_desktop_files().unwrap();
    ui::config::display_applications(&desktop_files);

    let blocked_input = ui::config::get_input(&desktop_files);

    for app in &blocked_input {
        println!("{:?}", app);
    }

    /*runtime.block_on(async {
        check_processes(&s, &blocked).await;
        
    });*/

}