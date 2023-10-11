use std::sync::{Mutex, Arc};
use warp::Filter;
use state::State;

mod handlers;
mod services;
mod state;

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(State::new()));

    let register_post_route = warp::path!("register")
        .and(warp::post())
        .and(warp::body::json())
        .and_then({
            let state = state.clone();
            move |body| handlers::register_handler(body, state.clone())
        });

    let routes = register_post_route.with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}
