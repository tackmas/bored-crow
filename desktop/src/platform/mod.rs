use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use anyhow::Result;
use tokio::{
    sync::{mpsc, oneshot},
    time,
};

use crate::unwrap_variant;

mod apps;
mod blocker;

struct Request<T, R> {
    data: T,
    replier: oneshot::Sender<R>,
}

impl<T, R> Request<T, R> {
    fn new(data: T) -> (Request<T, R>, oneshot::Receiver<R>) {
        let (replier, reciever) = oneshot::channel();

        (Request { data, replier }, reciever)
    }
}

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

#[derive(Clone)]
pub struct Blocker {
    sender: mpsc::Sender<Request<BlockerMessage, BlockerReply>>,
}

impl Blocker {
    pub async fn new() -> Result<Self, &'static str> {
        if BLOCKER_NEW_CALLED.load(Ordering::Relaxed) {
            return Err("already called this function");
        }

        BLOCKER_NEW_CALLED.store(true, Ordering::Relaxed);

        let (sender, reciever) = mpsc::channel(100);

        blocker::run(reciever).await;

        Ok(Blocker { sender })
    }
    pub async fn block(&self, app: &App) {
        let msg = BlockerMessage::Block(app.clone());

        self.blocker_request(msg).await;
    }
    pub async fn block_vec(&self, apps: &Vec<&App>) {
        println!("Attmepting to block vector of apps");

        for app in apps {
            self.block(app).await
        }

        println!("Blocked vector of apps.");
    }
    pub async fn unblock(&self, app: &App) {
        let msg = BlockerMessage::Unblock(app.clone());

        self.blocker_request(msg).await;
    }
    pub async fn unblock_vec(&self, apps: &Vec<&App>) {
        for app in apps {
            self.unblock(app).await
        }
    }

    pub async fn list_blocked(&self) -> Vec<App> {
        let msg = BlockerMessage::GetInfo;

        let response = self.blocker_request(msg).await;

        unwrap_variant!(response, BlockerReply::Info => a)
    }

    async fn blocker_request(&self, req_data: BlockerMessage) -> BlockerReply {
        let (request, mut reciever) = Request::new(req_data);

        self.sender.send(request).await.unwrap();

        let response = loop {
            match reciever.try_recv() {
                Ok(response) => break response,
                Err(_) => {
                    time::sleep(time::Duration::from_millis(100)).await;

                    continue;
                }
            }
        };

        response
    }
}

enum BlockerMessage {
    Block(App),
    Unblock(App),
    GetInfo,
}

#[derive(Debug)]
enum BlockerReply {
    None,
    Info(Vec<App>),
}


/*
pub struct BlockedApps {
    apps_quantity: usize,
    apps_is_blocked: Arc<Vec<AtomicU32>>
}

impl BlockedApps {
    pub fn new() -> Self {
        let apps_quantity = App::all_apps()
            .unwrap()
            .len();

        let len = (apps_quantity / 32) + 1;
        let vec = (0..len)
            .into_iter()
            .map(|_| AtomicU32::new(0))
            .collect();

        let apps_is_blocked = Arc::new(vec);

        BlockedApps {
            apps_quantity,
            apps_is_blocked
        }
    }
}
*/
