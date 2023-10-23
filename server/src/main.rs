use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use futures::{SinkExt, StreamExt, TryFutureExt};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::ws::{Message, WebSocket};
use warp::Filter;

type Users = Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Message>>>>;
type MessageHistory = Arc<RwLock<VecDeque<String>>>;

#[tokio::main]
async fn main() {
    let users = Users::default();
    let with_users = warp::any().map(move || users.clone());
    let chat_history = MessageHistory::default();
    let with_chat_history = warp::any().map(move || chat_history.clone());

    let index = warp::fs::dir("client");

    let chat = warp::path("chat")
        .and(warp::ws())
        .and(with_users)
        .and(with_chat_history)
        .map(|ws: warp::ws::Ws, users, chat_history| {
            ws.on_upgrade(move |socket| user_connected(socket, users, chat_history))
        });

    warp::serve(index.or(chat)).run(([127, 0, 0, 1], 3030)).await;
}

async fn user_connected(ws: WebSocket, users: Users, chat_history: MessageHistory) {
    let uuid = Uuid::new_v4().simple().to_string();

    let (mut user_ws_tx, mut user_ws_rx) = ws.split();
    let (tx, rx) = mpsc::unbounded_channel();

    // Send the history messages, so a new user can understand what's currently going on
    for msg in chat_history.read().await.iter() {
        if let Err(_disconnected) = tx.send(Message::text(msg)) {
            // Nothing here
        }
    }

    // Listen unbound receiver stream to forward message to ws connection
    let mut rx = UnboundedReceiverStream::new(rx);
    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            user_ws_tx
                .send(message)
                .unwrap_or_else(|e| {
                    eprintln!("websocket send error: {}", e);
                })
                .await;
        }
    });

    // Save user in a "current online" list
    users.write().await.insert(uuid.clone(), tx);

    // Listen ws connection rx to broadcast incoming messages
    while let Some(result) = user_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("websocket error(uid={}): {}", &uuid, e);
                break;
            }
        };

        user_message(msg, users.clone(), chat_history.clone()).await;
    }

    user_disconnected(uuid, users).await;
}

async fn user_message(msg: Message, users: Users, chat_history: MessageHistory) {
    let msg = if let Ok(s) = msg.to_str() {
        s.to_string()
    } else {
        return;
    };

    // Save message into history, remove older one if max threshold is exceeded
    let mut chat_write = chat_history.write().await;
    chat_write.push_back(msg.clone());
    if chat_write.len() > 60 {
        chat_write.pop_front();
    }

    for (_, tx) in users.read().await.iter() {
        if let Err(_disconnected) = tx.send(Message::text(&msg)) {
            // The tx is disconnected, our `user_disconnected` code
            // should be happening in another task, nothing more to
            // do here.
        }
    }
}

async fn user_disconnected(my_id: String, users: Users) {
    users.write().await.remove(&my_id);
}
