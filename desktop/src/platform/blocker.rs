use std::{
    collections::HashSet,
    ffi::OsStr
};

use sysinfo::{System, ProcessesToUpdate};
use tokio::{
    self, 
    sync::mpsc::Receiver,
    task,
    time::{self, Duration}
};

use super::{
    BlockAction::{self, Block, Unblock},
    Request
};


pub fn run(mut receiver: Receiver<Request<BlockAction, Result<(), String>>>) {  
    task::spawn(async move {
        let mut system = System::new_all();
        let mut block = HashSet::new();
        let mut interval = time::interval(Duration::from_millis(1000));

        loop {
            tokio::select! {
                raw_msg = receiver.recv() => {
                    match raw_msg {
                        Some(msg) => {
                            handle_req(msg, &mut block);
                        }
                        None => {
                            println!{"blocker channel has been closed"};
                            break;
                        }
                    }
                }
                _ = interval.tick() => scan_and_kill_process(&mut system,  &block),
            }
        }
    });
}

fn scan_and_kill_process(system: &mut System, apps: &HashSet<String>) {
    let names: Vec<&OsStr> = apps
        .iter()
        .map(|app| app.as_ref())
        .collect();

    for name in names
    {
        system.refresh_processes(ProcessesToUpdate::All, true);
        let mut blocked_processes = system.processes_by_name(name);

        while let Some(p) = blocked_processes.next() 
        {
            p.kill();
        }        
    }
} 

fn handle_req(req: Request<BlockAction, Result<(), String>>, block: &mut HashSet<String>) {
    match req.data {
        Block(app) => {
            let exists = block.get(&app.name).is_some();

            if exists {
                req.reply.send(Err(format!("error: can not block an already blocked app ({})", app.name))).unwrap();
                return;
            }

            block.insert(app.name);
        }

        Unblock(app) => {
            let exists = block.remove(&app.name);

            if !exists {
                req.reply.send(Err(format!("error: can not unblock an already unblocked app ({})", app.name))).unwrap();
                return;
            }
        }
    }

    req.reply.send(Ok(())).unwrap();
}