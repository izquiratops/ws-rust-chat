use warp::Filter;

use crate::{state::ThreadSafeState, handlers};

// All 'client' filters combined
pub fn clients(state: ThreadSafeState) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    register_client(state.clone())
}

pub fn register_client(state: ThreadSafeState) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("register")
        .and(warp::post())
        .and(warp::body::json())
        .and(use_state(state))
        .and_then(handlers::register_client)
}

fn use_state(state: ThreadSafeState) -> impl Filter<Extract = (ThreadSafeState,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}
