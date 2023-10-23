use std::collections::HashMap;
use std::sync::Arc;

use futures::{SinkExt, StreamExt, TryFutureExt};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::ws::{Message, WebSocket};
use warp::Filter;

type Users<'a> = Arc<RwLock<HashMap<&'a str, mpsc::UnboundedSender<Message>>>>;
type MessageHistory<'a> = Arc<RwLock<Vec<&'a str>>>;

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

async fn user_connected<'a>(ws: WebSocket, users: Users<'a>, chat_history: MessageHistory<'a>) {
    let uuid = Uuid::new_v4().simple().to_string();
    eprintln!("new chat user: {}", uuid);

    let (mut user_ws_tx, mut user_ws_rx) = ws.split();
    let (tx, rx) = mpsc::unbounded_channel();

    // Send the history messages, so a new user can understand what's currently going on
    for msg in chat_history.read().await.iter() {
        eprintln!("testing history: {:?}", msg);
        // tx.send(msg);
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
    users.write().await.insert(&uuid.clone(), tx);

    // Listen ws connection rx to broadcast incoming messages
    while let Some(result) = user_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("websocket error(uid={}): {}", uuid, e);
                break;
            }
        };

        user_message(msg, users.clone(), chat_history.clone()).await;
    }

    user_disconnected(&uuid, users).await;
}

async fn user_message<'a>(msg: Message, users: Users<'a>, chat_history: MessageHistory<'a>) {
    let msg = if let Ok(s) = msg.to_str() {
        s.to_string()
    } else {
        return;
    };

    chat_history.write().await.push(&msg);

    for (_, tx) in users.read().await.iter() {
        if let Err(_disconnected) = tx.send(Message::text(&msg)) {
            // The tx is disconnected, our `user_disconnected` code
            // should be happening in another task, nothing more to
            // do here.
        }
    }
}

async fn user_disconnected<'a>(my_id: &str, users: Users<'a>) {
    eprintln!("good bye user: {}", my_id);
    users.write().await.remove(my_id);
}
