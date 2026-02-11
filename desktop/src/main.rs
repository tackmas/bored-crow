use std::thread;

use tokio::runtime::Runtime;

mod platform;

fn main() {
    let runtime = Runtime::new().unwrap();
    let _guard = runtime.enter();
    
    let blocker = platform::Blocker::new();

    let mut apps = platform::App::all_apps().unwrap();

    blocker.block(&mut apps[0]).unwrap();

    for app in &apps {
        println!{"{app:?}"};
    }

    thread::sleep(std::time::Duration::from_secs(20));
}
