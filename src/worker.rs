use crate::config;
use crate::config::Config;
use axum::extract;
use axum::extract::State;
use axum::http::StatusCode;
use axum::middleware;
use axum::response::{IntoResponse, Json, Response};
use axum::routing::{get, post};
use axum::Router;
use base64::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::remove_file;
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use tracing::{error, info};

use crate::auth;

pub async fn init(port: &i16) {
    println!("{}", crate::LOGO);
    info!("launching node in [worker] mode on port {}...", port);

    if let Some(config) = config::get() {
        let app = Router::new()
            .route("/", get(hello))
            .route("/get-chunk", post(get_chunk))
            .route("/store-chunk", post(store_chunk))
            .route("/delete-chunk", post(delete_chunk))
            .route("/send-chunk", post(send_chunk))
            .route_layer(middleware::from_fn(auth::authorise))
            .with_state(config.clone());

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
            .await
            .unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

        let _ = tokio::task::spawn_blocking(move || background_heartbeat(config)).await;
    } else {
        error!("unable able to load the valid cluster configuration. Please make sure the ENV 'RDFS_ENDPOINT' and 'RDFS_TOKEN' are set");
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

#[derive(Deserialize, Serialize)]
struct SendChunk {
    id: String,
    target: String,
}

async fn hello(State(state): State<Config>) -> String {
    let response = format!("configurtion token -> '{}'", state.token);
    response.to_string()
}

#[axum::debug_handler]
async fn get_chunk(extract::Json(payload): extract::Json<MetaChunk>) -> Response {
    info!("get-chunk with ID [{}]", &payload.id);
    // todo: we can use regex to make sure that the payload ID is legal e.g <INT>-<GUIDv43> format
    if !Path::new(&payload.id).exists() {
        return StatusCode::NOT_FOUND.into_response();
    }

    if let Ok(chunk) = fs::read(&payload.id) {
        return Json(Chunk {
            id: payload.id,
            chunk: BASE64_STANDARD.encode(chunk),
        })
        .into_response();
    }
    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

#[axum::debug_handler]
async fn store_chunk(extract::Json(payload): extract::Json<Chunk>) -> Response {
    info!("store-chunk with ID [{}]", &payload.id);

    if let Ok(mut file) = fs::File::create(&payload.id) {
        if let Ok(chunk) = BASE64_STANDARD.decode(&payload.chunk) {
            if let Ok(_) = file.write(&chunk) {
                return Json(MetaChunk {
                    id: payload.id.to_string(),
                })
                .into_response();
            }
        }
    }
    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

#[axum::debug_handler]
async fn delete_chunk(extract::Json(payload): extract::Json<MetaChunk>) -> Response {
    info!("delete-chunk with ID [{}]", &payload.id);

    if let Ok(_) = remove_file(&payload.id) {
        return Json(MetaChunk { id: payload.id }).into_response();
    }

    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

#[axum::debug_handler]
async fn send_chunk(
    State(state): State<Config>,
    extract::Json(payload): extract::Json<SendChunk>,
) -> Response {
    info!("send-chunk [{}] to -> {}", &payload.id, &payload.target);

    if !Path::new(&payload.id).exists() {
        return StatusCode::NOT_FOUND.into_response();
    }

    if let Ok(chunk) = fs::read(&payload.id) {
        let data = Chunk {
            id: payload.id.clone(),
            chunk: BASE64_STANDARD.encode(chunk),
        };
        // NOTE: we may need to move this i/o call into it's own thread via spawn_blocking
        match ureq::post(&format!("{}/store-chunk", payload.target))
            .set("x-rdfs-token", &state.token)
            .send_json(data)
        {
            Ok(_) => {
                return Json(MetaChunk { id: payload.id }).into_response();
            }
            _ => {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    }
    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

fn background_heartbeat(config: Config) {
    info!("initiating the background heartbeat...");
    loop {
        // TODO: later on we could sent worker node meta information e.g disk space
        // to the master node.
        let _ = ureq::post(&format!("{}/heartbeat", config.endpoint))
            .set("x-rdfs-token", &config.token)
            .call();
        std::thread::sleep(Duration::from_millis(4000));
    }
}
