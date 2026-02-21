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
    App,
    BlockerMessage,
    BlockerReply,
    Request
};


pub fn run(mut receiver: Receiver<Request<BlockerMessage, BlockerReply>>) {  
    task::spawn(async move {
        let mut system = System::new_all();
        let mut block = HashSet::new();
        let mut interval = time::interval(Duration::from_millis(500));

        loop {
            tokio::select! {
                raw_req = receiver.recv() => {
                    match raw_req {
                        Some(req) => {
                            handle_req(req, &mut block);
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

fn handle_req(req: Request<BlockerMessage, BlockerReply>, block: &mut HashSet<String>) {
    match req.data {
        BlockerMessage::Block(app) => {
            let exists = block.contains(&app.name);

            if exists {
                let reply = BlockerReply::Outcome(Err(format!("error: can not block an already blocked app ({})", app.name)));

                req.reply.send(reply).unwrap();
                return;
            }

            block.insert(app.name);
        }

        BlockerMessage::Unblock(app) => {
            let exists = block.remove(&app.name);

            if !exists {
                let reply = BlockerReply::Outcome(Err(format!("error: can not unblock an already unblocked app ({})", app.name)));

                req.reply.send(reply).unwrap();
                return;
            }
        }

        BlockerMessage::GetInfo => {
            let blocked: Vec<App> = block
                .iter()
                .map(|name| App::from(name.clone()))
                .collect();

            let reply = BlockerReply::Info(blocked);

            req.reply.send(reply).unwrap();

            return;
        }
    }

    req.reply.send(BlockerReply::Outcome(Ok(()))).unwrap();
}

fn handle_req2(req: Request<BlockerMessage, BlockerReply>, block: &mut HashSet<String>) {
    if let BlockerMessage::GetInfo = req.data {
        let blocked: Vec<App> = block
            .iter()
            .map(|name| App::from(name.clone()))
            .collect();

        let reply = BlockerReply::Info(blocked);

        req.reply.send(reply).unwrap();    
    } else {
        
    }
}