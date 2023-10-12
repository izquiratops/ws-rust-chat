use std::convert::Infallible;
use warp::http::StatusCode;
use uuid::Uuid;

use crate::state::ThreadSafeState;

pub async fn register_client(body: RegisterRequest, state: ThreadSafeState) -> Result<impl warp::Reply, Infallible> {
    let state = state.lock().await;
    let uuid = Uuid::new_v4().simple().to_string();

    state.clone().register_client(uuid.clone(), body.user_id).await;

    let response = RegisterResponse {
        url: format!("ws://127.0.0.1:8000/ws/{}", uuid),
    };

    Ok(warp::reply::json(&response))
}

pub async fn unregister_client(params: UnregisterRequest, state: ThreadSafeState) -> Result<impl warp::Reply, Infallible> {
    let state = state.lock().await;

    state.clone().unregister_client(params.uuid).await;

    Ok(StatusCode::OK)
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
pub struct UnregisterRequest {
    uuid: String,
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