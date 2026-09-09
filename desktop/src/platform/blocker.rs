use std::{collections::HashMap, ffi::OsStr};
use std::ffi::OsString;
use std::sync::Arc;

use sysinfo::{ProcessesToUpdate, System};
use tokio::{
    self,
    sync::mpsc::{self, Sender, Receiver},
    task,
    time::{self, Duration},
};

use super::{BlockerMessage, ProcessName};

pub async fn run(mut receiver: Receiver<BlockerMessage>) {
    task::spawn(async move {
        let mut system = System::new_all().unwrap();
        let mut process_names = HashMap::new();
        let mut half_sec_interval = time::interval(Duration::from_millis(500));

        loop {
            tokio::select! {
                raw_req = receiver.recv() => {
                    match raw_req {
                        Some(req) => {
                            handle_req(req, &mut process_names);
                        }
                        None => {
                            println!{"Blocker channel has been closed"};
                            break;
                        }
                    }
                }
                _ = half_sec_interval.tick() => scan_and_kill_process(&mut system,  process_names.keys()),
            }
        }
    });
}

fn handle_req(req: BlockerMessage, process_names: &mut HashMap<ProcessName, usize>) {
    match req {
        BlockerMessage::Block(process_name) => {
            let process_name_counter = process_names.entry(process_name).or_insert(0);

            *process_name_counter += 1;
        },
        BlockerMessage::Unblock(process_name) => {
            let Some(process_name_counter) = process_names.get_mut(&process_name) else {
                eprintln!("{:?} is already not blocked (blocker.rs)", process_name);

                return;
            };

            *process_name_counter -= 1;

            if *process_name_counter == 0 {
                process_names.remove(&process_name);
            }
        },  
    }
}

fn scan_and_kill_process<'a>(system: &mut System, process_names: impl IntoIterator<Item = &'a ProcessName>) {
    for process_name in process_names {
        system.refresh_processes(ProcessesToUpdate::All, true);
        let mut blocked_processes = system.processes_by_name(process_name.as_ref().as_ref());

        while let Some(p) = blocked_processes.next() {
            p.kill();
        }
    }
}

