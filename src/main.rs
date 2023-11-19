use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
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

    warp::serve(index.or(chat))
        .run(([127, 0, 0, 1], 3030))
        .await;
}

async fn user_connected(ws: WebSocket, users: Users, chat_history: MessageHistory) {
    let uuid = Uuid::new_v4().simple().to_string();

    let (user_ws_tx, mut user_ws_rx) = ws.split();
    let (user_channel_tx, user_channel_rx) = mpsc::unbounded_channel();

    // Send the user the chat history when a user is connected.
    send_chat_history(&chat_history, &user_channel_tx).await;

    // Use an unbounded channel to set up a message queue for the user.
    tokio::task::spawn(forward_messages(user_channel_rx, user_ws_tx));

    // Save the sender in our list of connected users.
    users.write().await.insert(uuid.clone(), user_channel_tx);

    // Loop that listens for incoming messages from a user WebSocket.
    // It processes each message by calling the `user_message` function with the received message,
    // the `users` collection, and the `chat_history` collection as arguments.
    // If an error occurs during the WebSocket communication, it prints an error message and breaks out of the loop.
    while let Some(result) = user_ws_rx.next().await {
        match result {
            Ok(msg) => user_message(msg, &users, &chat_history).await,
            Err(e) => {
                eprintln!("websocket error(uid={}): {}", &uuid, e);
                break;
            }
        }
    }

    user_disconnected(uuid, users).await;
}

async fn send_chat_history(chat_history: &MessageHistory, tx: &mpsc::UnboundedSender<Message>) {
    for chat_history_message in chat_history.read().await.iter() {
        if let Err(_disconnected) = tx.send(Message::text(chat_history_message)) {
            eprintln!("Error sending a history message: {}", _disconnected);
        }
    }
}

async fn forward_messages(rx: mpsc::UnboundedReceiver<Message>, mut user_ws_tx: SplitSink<WebSocket, Message>) {
    let mut rx = UnboundedReceiverStream::new(rx);
    while let Some(message) = rx.next().await {
        if let Err(_disconnected) = user_ws_tx.send(message).await {
            eprintln!("Error forwarding a message: {}", _disconnected);
        }
    }
}

async fn user_message(msg: Message, users: &Users, chat_history: &MessageHistory) {
    if let Ok(msg_str) = msg.to_str() {
        let msg_text = msg_str.to_string();

        broadcast_message(&users, &msg_text).await;

        save_message_to_history(&chat_history, msg_text).await;
    }
}

async fn save_message_to_history(chat_history: &MessageHistory, msg_text: String) {
    let mut chat_write = chat_history.write().await;
    chat_write.push_back(msg_text);

    if chat_write.len() > 60 {
        chat_write.pop_front();
    }
}

async fn broadcast_message(users: &Users, msg_text: &str) {
    for (_, tx) in users.read().await.iter() {
        if let Err(_disconnected) = tx.send(Message::text(msg_text)) {
            // The tx is disconnected, our `user_disconnected` code
            // should be happening in another task, nothing more to
            // do here.
            eprintln!("Couldn't broadcast a message, user may be disconnected: {}", _disconnected);
        }
    }
}

async fn user_disconnected(my_id: String, users: Users) {
    users.write().await.remove(&my_id);
}
