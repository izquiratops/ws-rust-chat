use warp::Filter;
use state::State;

mod handlers;
mod filters;
mod state;

#[tokio::main]
async fn main() {
    let server_state = State::init_thread_safe();

    let api = filters::clients(server_state);

    let routes = api
        .with(warp::log("todos"))
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

#[cfg(test)]
mod api_test {
    use std::collections::HashMap;

    use warp::test::request;
    use warp::http::StatusCode;

    use super::{state::State, filters};

    #[tokio::test]
    async fn post_register_client() {
        let server_state = State::init_thread_safe();
        let api = filters::clients(server_state);

        let response = request()
            .method("POST")
            .path("/register")
            .json(&HashMap::from([("user_id", 123)]))
            .reply(&api)
            .await;

        assert_eq!(response.status(), StatusCode::OK);
    }
}