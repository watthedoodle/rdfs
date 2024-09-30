use crate::config;
use crate::config::Config;
use axum::extract::State;
use axum::routing::get;
use axum::Router;
use std::{thread, time::Duration};

pub async fn init() {
    println!("{}", crate::LOGO);
    println!("==> launching node in [worker] mode...");

    if let Some(config) = config::get() {
        let app = Router::new().route("/", get(hello)).with_state(config);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:8888").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    } else {
        println!("==> Error: unable able to load the valid cluster configuration. Please make sure the ENV 'RDFS_ENDPOINT' and 'RDFS_TOKEN' are set");
    }
}

async fn hello(State(state): State<Config>) -> String {
    let response = format!("configurtion token -> '{}'", state.token);
    response.to_string()
}
