use std::collections::HashMap;
use tokio::sync::mpsc;
use warp::filters::ws::Message;

#[derive(Clone)]
pub struct Client {
    pub user_id: usize,
    pub topics: Vec<String>,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

#[derive(Clone)]
pub struct State {
    clients: HashMap<String, Client>,
}

impl State {
    pub fn new() -> Self {
        State {
            clients: HashMap::new(),
        }
    }

    pub async fn register_client(mut self, id: String, user_id: usize) {
        self.clients.insert(
            id,
            Client {
                user_id,
                topics: vec![String::from("cats")],
                sender: None,
            },
        );
    }
}
