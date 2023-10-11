use std::{sync::{Arc, Mutex}, convert::Infallible};
use uuid::Uuid;

use crate::state::State;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct RegisterRequest {
    user_id: usize,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct RegisterResponse {
    url: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Event {
    topic: String,
    user_id: Option<usize>,
    message: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct TopicsRequest {
    topics: Vec<String>,
}

type Result<T> = std::result::Result<T, Infallible>;

pub async fn register_handler(body: RegisterRequest, state: Arc<Mutex<State>>) -> Result<impl warp::Reply> {
    let state = state.lock().unwrap();
    let user_id = body.user_id;
    let uuid = Uuid::new_v4().simple().to_string();

    state.register_client(uuid.clone(), user_id).await;

    let response = RegisterResponse {
        url: format!("ws://127.0.0.1:8000/ws/{}", uuid),
    };

    Ok(warp::reply::json(&response))
}
