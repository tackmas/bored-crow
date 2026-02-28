use anyhow::Result;
use tokio::{
    sync::{mpsc, oneshot}
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
    pub fn map_into_names(apps: &Vec<App>) -> Vec<String> {
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

pub struct Blocker {
    sender: mpsc::Sender<Request<BlockerMessage, BlockerReply>>
}

impl Blocker {
    pub fn new() -> Blocker {
        let (sender, reciever) = mpsc::channel(100); 

        blocker::run(reciever);

        Blocker {sender}
    }
    pub fn block(&self, app: &App) -> Result<(), String>  {
        let msg = BlockerMessage::Block(app.clone());

        let response = self.blocker_request(msg);

        response.outcome_inner()
    }
    pub fn unblock(&self, app: &App) -> Result<(), String> {
        let msg = BlockerMessage::Unblock(app.clone());

        let response = self.blocker_request(msg);

        response.outcome_inner()
    }

    pub fn list_blocked(&self) -> Vec<App> {
        let msg = BlockerMessage::GetInfo;

        let response = self.blocker_request(msg);

        response.info_inner()
    }

    fn blocker_request(&self, req_data: BlockerMessage) -> BlockerReply {   
        let (request, reciever) = Request::new(
            req_data
        );
        self.sender.blocking_send(request).unwrap();

        let response = reciever.blocking_recv().unwrap();

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


