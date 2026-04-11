use std::{
    sync::{
        Arc,
        atomic::{
            AtomicU32,
        },
    }
};

use anyhow::Result;
use tokio::{
    sync::{mpsc, oneshot},
    time,
};

mod blocker;
mod apps;

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
    pub fn all_apps() -> Result<Vec<App>>  {
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
        App {
            name: value
        }
    }
}

#[derive(Clone)]
pub struct Blocker {
    sender: mpsc::Sender<Request<BlockerMessage, BlockerReply>>
}

impl Blocker {
    pub fn new() -> Blocker {
        let (sender, reciever) = mpsc::channel(100); 

        blocker::run(reciever);

        Blocker {sender}
    }
    pub async fn block(&self, app: &App) -> Result<(), String>  {
        let msg = BlockerMessage::Block(app.clone());

        let response = self.blocker_request(msg).await;

        response.outcome_inner()
    }
    pub async fn block_vec(&self, apps: &Vec<&App>) -> Result<(), String> {
        for app in apps {
            self.block(app).await?
        }

        Ok(())
    }
    pub async fn unblock(&self, app: &App) -> Result<(), String> {
        let msg = BlockerMessage::Unblock(app.clone());

        let response = self.blocker_request(msg).await;

        response.outcome_inner()
    }
    pub async fn unblock_vec(&self, apps: &Vec<&App>) -> Result<(), String> {
        for app in apps {
            self.unblock(app).await?
        }
        
        Ok(())
    }

    pub async fn list_blocked(&self) -> Vec<App> {
        let msg = BlockerMessage::GetInfo;

        let response = self.blocker_request(msg).await;

        response.info_inner()
    }

    async fn blocker_request(&self, req_data: BlockerMessage) -> BlockerReply {   
        let (request, mut reciever) = Request::new(
            req_data
        );
        
        self.sender.send(request).await.unwrap();

        let response = {
            loop {
                match reciever.try_recv() {
                    Ok(response) => break response,
                    Err(_) => {
                        time::sleep(time::Duration::from_millis(100)).await;

                        continue;
                    }
                }
            }

        };

        response
    }
}

enum BlockerMessage {
    Block(App),
    Unblock(App),
    GetInfo
}

#[derive(Debug)]
enum BlockerReply {
    Outcome(Result<(), String>),
    Info(Vec<App>)
}

impl BlockerReply {
    fn outcome_inner(self) -> Result<(), String> {
        if let BlockerReply::Outcome(outcome) = self {
            outcome
        } else {
            panic!(r#"wrong enum variant; must be BlockerReply::Outcome(...)"#);
        }
    }
    fn info_inner(self) -> Vec<App> {
        if let BlockerReply::Info(apps) = self {
            apps
        } else {
            panic!(r#"wrong enum variant; must be BlockerReply::Info(...)"#);
        }
    }
}


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