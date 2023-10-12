use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use warp::filters::ws::Message;

#[derive(Debug, Clone)]
pub struct Client {
    pub user_id: usize,
    pub topics: Vec<String>,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

pub type ThreadSafeState = Arc<Mutex<State>>;

#[derive(Debug, Clone)]
pub struct State {
    clients: HashMap<String, Client>,
}

impl State {
    pub fn init_thread_safe() -> ThreadSafeState {
        Arc::new(Mutex::new(State {
            clients: HashMap::new(),
        }))
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
