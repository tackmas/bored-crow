use std::process::ExitCode;

use interprocess::local_socket::Stream;

use desktop::ipc::{IPCClientExt, Request, Response};


fn main() -> ExitCode {
    let stream = Stream::create_client().unwrap();

    let response = stream.send_request(Request::CanUninstall);

    match response {
        Response::CanUninstall => {
            println!("Can uninstall!");

            ExitCode::SUCCESS
        },
        Response::CanNotUninstall => {
            println!("Can not uninstall");
            ExitCode::FAILURE
        }
    }
}