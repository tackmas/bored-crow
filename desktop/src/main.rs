use sysinfo::System;
use tokio::runtime;

mod core;
mod ui;
mod platform;

#[cfg(feature = "error")]
mod error;


fn main() {
    let runtime = runtime::Runtime::new().unwrap();
    let handle = runtime.handle();

    let platform = platform::Platform::new(handle);

    platform.lock().unwrap().refresh();


    ui::config::display_applications(&platform.lock().unwrap());

    let blocked_input = ui::config::get_input(&platform.lock().unwrap());

    for name in &blocked_input {
        let inner = platform.lock().unwrap();

        let app = inner.app(name).unwrap();

        app.block();
    }

    for app in &blocked_input {
        println!("{:?}", app);
    }

    /*runtime.block_on(async {
        check_processes(&s, &blocked).await;
        
    });*/

}