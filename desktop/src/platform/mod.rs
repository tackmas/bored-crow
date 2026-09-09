use std::ffi::{OsStr, OsString};
use std::ops::{Deref, DerefMut};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    OnceLock,
};

use anyhow::Result;

use serde::{Deserialize, Serialize};

use tokio::{
    sync::{mpsc, oneshot},
    time,
};

use crate::{impl_deref_mut_for_newtype, unwrap_variant};

mod apps;
mod blocker;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct App {
    name: String,
}

impl App {
    pub fn all_apps() -> Result<Vec<App>> {
        apps::apps()
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn map_into_names(apps: &[App]) -> Vec<String> {
        let mut names = Vec::with_capacity(apps.len());

        for app in apps {
            names.push(app.name.clone());
        }

        names
    }
}


impl From<String> for App {
    fn from(value: String) -> Self {
        App { name: value }
    }
}

static BLOCKER_NEW_CALLED: AtomicBool = AtomicBool::new(false);
static BLOCKER: OnceLock<Blocker> = OnceLock::new();

#[derive(Clone)]
pub struct Blocker {
    sender: mpsc::Sender<BlockerMessage>,
}

impl Blocker {
    pub async fn init() {
        let (sender, reciever) = mpsc::channel(100);

        blocker::run(reciever).await;

        let blocker = Blocker { sender };

        BLOCKER.set(blocker);       
    }
    pub async fn new() -> Result<Self, &'static str> {
        if BLOCKER_NEW_CALLED.load(Ordering::Relaxed) {
            return Err("already called this function");
        }

        BLOCKER_NEW_CALLED.store(true, Ordering::Relaxed);

        let (sender, reciever) = mpsc::channel(100);

        blocker::run(reciever).await;

        Ok(Blocker { sender })
    }
    pub async fn block_process(&self, process_name: ProcessName) {
        let msg = BlockerMessage::Block(process_name);

        self.sender
            .send(msg)
            .await
            .expect("Reciever should never be dropped since it recieves in a indefinte loop until Blocker goes out of scope");
    }
    pub async fn block_processes(&self, process_names: impl IntoIterator<Item = ProcessName>) {
        println!("Attmepting to block vector of apps");

        for process_name in process_names {
            self.block_process(process_name).await
        }

        println!("Blocked vector of apps.");
    }
    pub async fn unblock_process(&self, process_name: ProcessName) {
        let msg = BlockerMessage::Unblock(process_name);

        self.sender
            .send(msg)
            .await
            .expect("Reciever should never be dropped since it recieves in a indefinte loop until Blocker goes out of scope");
    }
    pub async fn unblock_processes(&self, process_names: impl IntoIterator<Item = ProcessName>) {
        for process_name in process_names {
            self.unblock_process(process_name).await
        }
    }

}

enum BlockerMessage {
    Block(ProcessName),
    Unblock(ProcessName),
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ProcessName(pub Arc<str>);



impl ProcessName {
    pub fn from(from: impl Into<Arc<str>>) -> Self {
        ProcessName(from.into())
    }
    // Clones
    pub fn from_os_str(os_str: &OsStr) -> Self {
        ProcessName(os_str.to_string_lossy().into())
    }
}

impl_deref_mut_for_newtype!(ProcessName, Arc<str>);