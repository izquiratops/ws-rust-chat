use warp::Filter;

use crate::{state::ThreadSafeState, handlers::{self, UnregisterRequest}};


// All 'client' filters combined
pub fn clients(state: ThreadSafeState) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    register_client(state.clone()).or(unregister_client(state))
}

pub fn register_client(state: ThreadSafeState) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("register")
        .and(warp::post())
        .and(warp::body::json())
        .and(use_state(state))
        .and_then(handlers::register_client)
}

pub fn unregister_client(state: ThreadSafeState) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("delete")
        .and(warp::delete())
        .and(warp::query::<UnregisterRequest>())
        .and(use_state(state))
        .and_then(handlers::unregister_client)
}

fn use_state(state: ThreadSafeState) -> impl Filter<Extract = (ThreadSafeState,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}
