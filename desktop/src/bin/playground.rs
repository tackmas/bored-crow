use tokio;
use tokio_stream::{self, StreamExt};
use std::io;

use 

use libcopes::{
    ProcessEventsConnector,
    PEvent,
    PID,
    get_process_executed_file,
    io::{
        cmdline_reader,
        exe_reader
    }
};

use futures;
use sysinfo as sys;


#[tokio::main]
async fn main() {
    let handle = test();

    handle.await.unwrap();
}

fn test() -> tokio::task::JoinHandle<()> {
    tokio::spawn(async {
        let mut system = sysinfo::System::new_all();
        let connector = ProcessEventsConnector::try_new().unwrap();
        let sync_events = connector.into_iter();
        let mut events = tokio_stream::iter(sync_events);

        while let Some(event) = events.next().await {
            let event = event.unwrap();

            if let PEvent::Exec(pid) = event {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                let exe = try_read_proc_entry(pid, libcopes::io::exe_reader)
                    .unwrap_or_else(|e| panic!("{e}"));

                let cmd_line = try_read_proc_entry(pid, libcopes::io::cmdline_reader)
                    .unwrap_or_else(|e| panic!("{e}"));

                system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
                
                let pid = *pid.as_ref() as usize;
                let pid = sysinfo::Pid::from(pid);
                
                let pro = match system.process(pid) {
                    Some(v) => v,
                    None => {
                        for (pid, process) in system.processes() {
                            println!("{} {:?}", pid, process.name());
                        }

                        continue
                    }
                };

                println!("{:?}", pro.name());
                

                let t = (exe, cmd_line);

                if let (Some(e), Some(c)) = t {
                    let exe = get_process_executed_file(e, &c);
                    println!("{exe}");
                } else if let (Some(e), None) = t {
                    println!("{e:?}");
                }   
                
            }
        };
    })
}


fn test2() {
    let connector = ProcessEventsConnector::try_new().unwrap();
    let mut sync_events = connector.into_iter();

    loop {
        if let Some(event) = sync_events.next() {
            let event = event.unwrap();

            if let PEvent::Exec(pid) = event {
                let exe = try_read_proc_entry(pid, libcopes::io::exe_reader)
                    .unwrap_or_else(|e| panic!("{e}"));

                let cmd_line = try_read_proc_entry(pid, libcopes::io::cmdline_reader)
                    .unwrap_or_else(|e| panic!("{e}"));

                let t = (exe, cmd_line);

                if let (Some(e), Some(c)) = t {
                    let exe = get_process_executed_file(e, &c);
                    println!("{exe}");
                } else if let (Some(e), None) = t {
                    println!("{e:?}");
                }     
            }
        }        
    }

}

fn test3() {
    let system = sys::System::new_all();



}


fn try_read_proc_entry<T, F>(pid: libcopes::PID, reader: F) -> io::Result<Option<T>> 
where 
    F: FnOnce(libcopes::PID) -> io::Result<T>,
{ 

    match reader(pid) {
        Ok(v) => return Ok(Some(v)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => return Err(e)
    }
}