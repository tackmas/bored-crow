use super::{App, Platform};
use tokio::{self, time};
use sysinfo::{self, System};
use std::ffi::OsStr;
use std::sync::{Arc, Mutex};

pub fn run(platform: Arc<Mutex<Platform>>, rt_handle: &tokio::runtime::Handle) {
    rt_handle.spawn(async move {
        let mut system = System::new_all();
        let mut interval = time::interval(time::Duration::from_millis(1000));

        loop {
            interval.tick().await;
            let blocked_iter = platform.list_blocked();

            if let None = blocked_iter.clone().next() {

            }

            scan_and_kill_process(&mut system, blocked_iter);
            
        }
    });
}

fn scan_and_kill_process<'a>(system: &mut System, apps: impl Iterator<Item=&'a App>) {
    let names: Vec<&OsStr> = apps
        .map(|app| app.name.as_ref())
        .collect();

    for name in names
    {
        system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        let mut blocked_processes = system.processes_by_name(name);

        while let Some(p) = blocked_processes.next() 
        {
            p.kill();
        }        
    }


} 