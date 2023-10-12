use std::convert::Infallible;
use uuid::Uuid;

use crate::state::ThreadSafeState;

pub async fn register_client(body: RegisterRequest, state: ThreadSafeState) -> Result<impl warp::Reply, Infallible> {
    let state = state.lock().await;
    let user_id = body.user_id;
    let uuid = Uuid::new_v4().simple().to_string();

    state.clone().register_client(uuid.clone(), user_id).await;

    let response = RegisterResponse {
        url: format!("ws://127.0.0.1:8000/ws/{}", uuid),
    };

    Ok(warp::reply::json(&response))
}

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