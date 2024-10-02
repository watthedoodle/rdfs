use crate::config;
use crate::config::Config;
use axum::extract::State;
use axum::http::header::HeaderMap;
use axum::middleware;
use axum::routing::{get, post};
use axum::Router;
use std::env;
use std::{thread, time::Duration};
use serde::Deserialize;

use crate::auth;

pub async fn init() {
    println!("{}", crate::LOGO);
    println!("==> launching node in [worker] mode...");

    if let Some(config) = config::get() {
        let app = Router::new()
            .route("/", get(hello))
            .route("/get-chunk", post(get_chunk))
            .route_layer(middleware::from_fn(auth::authorise))
            .with_state(config.clone());

        let listener = tokio::net::TcpListener::bind("0.0.0.0:8888").await.unwrap();
        let task = tokio::spawn(background_heartbeat(config));
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let _ = task.await;
    } else {
        println!("==> Error: unable able to load the valid cluster configuration. Please make sure the ENV 'RDFS_ENDPOINT' and 'RDFS_TOKEN' are set");
    }
}

#[derive(Deserialize)]
struct MetaChunk {
    id: String
}

#[derive(Deserialize)]
struct Chunk {
    id: String,
    chunk: String
}

async fn hello(State(state): State<Config>) -> String {
    let response = format!("configurtion token -> '{}'", state.token);
    response.to_string()
}

#[axum::debug_handler]
async fn get_chunk(State(state): State<Config>) -> String {
    let response = format!("configurtion token -> '{}'", state.token);
    response.to_string()
}

async fn store_chunk(State(state): State<Config>) -> String {
    let response = format!("configurtion token -> '{}'", state.token);
    response.to_string()
}

async fn delete_chunk(State(state): State<Config>) -> String {
    let response = format!("configurtion token -> '{}'", state.token);
    response.to_string()
}

async fn send_chunk(State(state): State<Config>) -> String {
    let response = format!("configurtion token -> '{}'", state.token);
    response.to_string()
}

async fn background_heartbeat(config: Config) {
    loop {
        println!(
            "==> simulating a heartbeat send! using token -> '{}'",
            config.token
        );
        tokio::time::sleep(Duration::from_millis(4000)).await;
    }
}
