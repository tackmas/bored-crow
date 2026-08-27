mod state;

// Std
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::{self, Command};
use std::sync::atomic::{AtomicBool, Ordering};

// External
use tokio::runtime::Builder;
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};

use qsu::argp::{ArgParser, ArgsProc};
use qsu::async_trait;
use qsu::log;
use qsu::rt::{Demise, InitCtx, RunCtx, RunEnv, SrvAppRt, SvcEvt, TermCtx, TokioServiceHandler};

// Local

use desktop::APP_NAME;

use desktop::ipc::*;

use self::state::State;

enum _Error {
    InvalidMessage,
    UnknownId,
    AlreadyBlocked,
    IsLocked,
}

enum Shutdown {
    Restart,
    Uninstall
}


fn main() {
    println!("Hello, world!");

    ArgParser::new(APP_NAME, &mut Args)
        .regsvc_proc(|regsvc| regsvc.autostart())
        .proc()
        .unwrap();
}

struct Args;

impl ArgsProc for Args {
    type AppErr = AppError;

    fn build_apprt(&mut self, runctx: &mut RunCtx) -> Result<SrvAppRt<Self::AppErr>, Self::AppErr> {
        let mut builder = Builder::new_multi_thread();

        builder.enable_io();
        builder.enable_time();

        let rtbldr = Some(builder);

        let (restart_tx, restart_rx) = mpsc::channel(1);
        let uninstall_tx = restart_tx.clone();
        runctx.init_passthrough_r(uninstall_tx);

        let svcevt_handler = Box::new(move |event| {
            if let SvcEvt::Shutdown(Demise::Terminated) = event {
                let _ = restart_tx.blocking_send(Shutdown::Restart);
            }
        });

        let rt_handler = Box::new(
            Daemon::new_with_shutdown_rx(restart_rx)
        );

        Ok(SrvAppRt::Tokio {
            rtbldr,
            svcevt_handler,
            rt_handler
        })
    }
}

#[derive(Debug)]
pub struct AppError;

struct Daemon {
    state: Option<State>,
    shutdown_rx: mpsc::Receiver<Shutdown>
}

impl Daemon {
    fn new_with_shutdown_rx(shutdown_rx: mpsc::Receiver<Shutdown>) -> Self {
        Self {
            state: None,
            shutdown_rx
        }
    }

    async fn load_state(&mut self, uninstall_tx: mpsc::Sender<Shutdown>) {
        self.state = Some(State::load(uninstall_tx).await);
    }
}

#[async_trait]
impl TokioServiceHandler for Daemon {
    type AppErr = AppError;

    async fn init(&mut self, ictx: &mut InitCtx) -> Result<(), AppError> {
        log_to_file("init() called");

        let uninstall_tx = ictx
            .take()
            .expect("`RunCtx` in `ArgsProc::build_apprt` should always pass a `mpsc::Sender<Shutdown>`");

        self.load_state(uninstall_tx).await;

        log_to_file("init() done");

        Ok(())
    }

    async fn run(&mut self, _re: &RunEnv) -> Result<(), AppError> {
        println!("a");
        log_to_file("run() started");
        let state = self.state.as_mut().unwrap();

        tokio::select! {
            _ = state.run() => { log_to_file("state.run() finished"); },
            shutdown = self.shutdown_rx.recv() => { 
                let shutdown = shutdown
                    .expect("`Sender` should always outlive `Reciever`");

                log_to_file("shutdown signal received");

                on_shutdown(shutdown)
            }
        }

        log_to_file("run() returning");
        Ok(())
    }

    async fn shutdown(&mut self, _tctx: &mut TermCtx) -> Result<(), AppError> {
        // Useless function

        Ok(())
    }
}

fn on_shutdown(shutdown: Shutdown) {
    match shutdown {
        Shutdown::Restart => {
            let restarter_path = {
                let mut current_exe_path = env::current_exe().unwrap();

                current_exe_path.pop();
                current_exe_path.push("restarter.exe");
                current_exe_path
             };
            
            // Relevant to UNIX platforms only:
            // Parent (this binary) will exit after this call, and the child's parent will become init,
            // which periodically calls wait on the child. Therefore no zombie process will be left behind
            #[allow(clippy::zombie_processes)]
            Command::new(restarter_path)
                .spawn()
                .unwrap();
        },
        Shutdown::Uninstall => ()
    }
}

fn log_to_file(msg: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("C:\\tackmas_debug.log")
        .unwrap();
    writeln!(file, "{}", msg).unwrap();
}