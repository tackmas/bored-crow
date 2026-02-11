
use std::io;

use tokio::{
    sync::{mpsc, oneshot}
};

mod blocker;
mod linux;

struct Request<T, R> {
    data: T,
    reply: oneshot::Sender<R>,
}

impl<T, R> Request<T, R> {
    fn new(data: T) -> (Request<T, R>, oneshot::Receiver<R>) {
        let (reply, response) = oneshot::channel();
        
        (Request { data, reply }, response)
    }   
}

#[derive(Clone, Debug)]
pub struct App {
    name: String,
}

impl App {
    pub fn all_apps() -> io::Result<Vec<App>>  {
        linux::apps()
    }
}

pub struct Blocker {
    sender: mpsc::Sender<Request<BlockAction, Result<(), String>>>
}

impl Blocker {
    pub fn new() -> Blocker {
        let (sender, reciever) = mpsc::channel(100); 

        blocker::run(reciever);

        Blocker {sender}
    }
    pub fn block(&self, app: &mut App) -> Result<(), String>  {
        let (request, response) = Request::new(
            BlockAction::Block(app.clone())
        );

        self.sender.blocking_send(request).unwrap();

        let response: Result<(), String> = response.blocking_recv().unwrap();

        response
    }
    pub fn unblock(&self, app: &mut App) -> Result<(), String> {
        let (request, response) = Request::new(
            BlockAction::Unblock(app.clone())
        );

        self.sender.blocking_send(request).unwrap();

        let response = response.blocking_recv().unwrap();

        response
    }
}

enum BlockAction {
    Block(App),
    Unblock(App)
}
