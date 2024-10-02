use crate::config;
use crate::config::Config;
use axum::extract;
use axum::extract::State;
use axum::http::StatusCode;
use axum::middleware;
use axum::response::{IntoResponse, Json, Response};
use axum::routing::{get, post};
use axum::Router;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Duration;

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

#[derive(Deserialize, Serialize)]
struct MetaChunk {
    id: String,
}

#[derive(Deserialize, Serialize)]
struct Chunk {
    id: String,
    chunk: String,
}

async fn hello(State(state): State<Config>) -> String {
    let response = format!("configurtion token -> '{}'", state.token);
    response.to_string()
}

#[axum::debug_handler]
async fn get_chunk(extract::Json(payload): extract::Json<MetaChunk>) -> Response {
    println!("==> get-chunk with ID [{}]", &payload.id);

    if !Path::new(&payload.id).exists() {
        return StatusCode::NOT_FOUND.into_response();
    }

    if let Ok(chunk) = fs::read_to_string(&payload.id) {
        return Json(Chunk {
            id: payload.id,
            chunk: chunk,
        })
        .into_response();
    }
    StatusCode::BAD_REQUEST.into_response()
}

#[axum::debug_handler]
async fn store_chunk(State(state): State<Config>) -> String {
    let response = format!("configurtion token -> '{}'", state.token);
    response.to_string()
}

#[axum::debug_handler]
async fn delete_chunk(State(state): State<Config>) -> String {
    let response = format!("configurtion token -> '{}'", state.token);
    response.to_string()
}

#[axum::debug_handler]
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
