use crate::auth;
use crate::config;
use axum::extract;
use axum::extract::ConnectInfo;
use axum::http::StatusCode;
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing::{error, info};

pub async fn init(port: &i16) {
    println!("{}", crate::LOGO);
    info!("launching node in [master] mode on port {}...", port);

    if let Some(config) = config::get() {
        let app = Router::new()
            .route("/heartbeat", post(heartbeat))
            .route("/list", post(list))
            .route("/get", post(get))
            .route("/upload", post(upload))
            .route("/remove", post(remove))
            .route_layer(middleware::from_fn(auth::authorise))
            .with_state(config.clone());

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
            .await
            .unwrap();

        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap()
    } else {
        error!("Error: unable able to load the valid cluster configuration. Please make sure the ENV 'RDFS_ENDPOINT' and 'RDFS_TOKEN' are set");
    }
}

async fn heartbeat(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> String {
    info!("got a heartbeat from worker node -> ...{}", addr);
    "ok".to_string()
}

#[derive(Deserialize, Serialize)]
struct FileMeta {
    name: String,
    size: Option<u64>,
}

#[axum::debug_handler]
async fn list() -> Response {
    info!("list all files");
    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

#[axum::debug_handler]
async fn get(extract::Json(payload): extract::Json<FileMeta>) -> Response {
    info!("get file with name [{}]", &payload.name);
    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

#[axum::debug_handler]
async fn upload(extract::Json(payload): extract::Json<FileMeta>) -> Response {
    info!("upload file with name [{}]", &payload.name);
    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

#[axum::debug_handler]
async fn remove(extract::Json(payload): extract::Json<FileMeta>) -> Response {
    info!("remove file with name [{}]", &payload.name);
    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}
