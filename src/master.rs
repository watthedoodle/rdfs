use crate::auth;
use crate::config;
use crate::config::Config;
use axum::extract::State;
use axum::middleware;
use axum::routing::get;
use axum::Router;
use std::{thread, time::Duration};

pub async fn init(port: &i16) {
    println!("{}", crate::LOGO);
    println!("==> launching node in [master] mode on port {}...", port);

    if let Some(config) = config::get() {
        let app = Router::new()
            .route("/", get(heartbeat))
            .route_layer(middleware::from_fn(auth::authorise))
            .with_state(config.clone());

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
            .await
            .unwrap();
        // let task = tokio::spawn(background_heartbeat(config));
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        // let _ = task.await;
    } else {
        println!("==> Error: unable able to load the valid cluster configuration. Please make sure the ENV 'RDFS_ENDPOINT' and 'RDFS_TOKEN' are set");
    }
}

async fn heartbeat() -> String {
    println!("==> got a heartbeat from worker node -> ...");
    "ok".to_string()
}
