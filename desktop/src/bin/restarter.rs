/*  
    This binary deregisters, registers, starts and configures the service behind the daemon. 
    It is supposed to be run when the service has been compromised, such as terminating the service. 
    This binary is to be run by the daemon when it recieves a signal to shutdown, 
    which is possible if the user stops the service.
*/


use std::env::current_exe;
use std::ffi::OsStr;
use std::io;
use std::process::Command;
use std::time::Duration;
use std::thread;

fn main() {
    thread::sleep(Duration::from_secs(2));

    // The compiled binary `restarter.exe` of this binary 
    // source code must be in the same directory as `daemon.exe`
    let daemon_exe_path = {
        let mut current_exe_path = current_exe().unwrap();
        current_exe_path.pop();
        current_exe_path.push("daemon.exe");
        current_exe_path
    };

    let run_daemon_with_arg = |arg: &str| {
        run_process_with_args(&daemon_exe_path, [arg]).unwrap();
    };

    run_daemon_with_arg("deregister-service");
    run_daemon_with_arg("register-service");

    #[cfg(target_os = "windows")]
    {
        let sc_with_args = |sub_command, args: &[&str]| {
            let args = [sub_command, "Bored Crow"]
                .into_iter()
                .chain(args.iter().copied());

            run_process_with_args("sc", args)
        };

        sc_with_args("start", &[]).unwrap();
        sc_with_args("failure", &["reset=", "1", "actions=", "restart/1000"]).unwrap();
    }

}

fn run_process_with_args<'a>(
    process: impl AsRef<OsStr>, 
    args: impl IntoIterator<Item = &'a str>
) -> io::Result<()> 
{
    Command::new(process.as_ref())
        .args(args)
        .status()?;

    Ok(())
}

