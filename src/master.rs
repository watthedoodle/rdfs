use crate::auth;
use crate::config;
use crate::config::Config;
use axum::extract::ConnectInfo;
use axum::extract::State;
use axum::middleware;
use axum::routing::{get, post};
use axum::Router;
use std::net::SocketAddr;
use std::{thread, time::Duration};
use tracing::{error, info};

pub async fn init(port: &i16) {
    println!("{}", crate::LOGO);
    info!("launching node in [master] mode on port {}...", port);

    if let Some(config) = config::get() {
        let app = Router::new()
            .route("/heartbeat", post(heartbeat))
            .route_layer(middleware::from_fn(auth::authorise))
            .with_state(config.clone());

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
            .await
            .unwrap();
        // let task = tokio::spawn(background_heartbeat(config));
        // tokio::spawn(async move {
        //     axum::serve(
        //         listener,
        //         app.into_make_service_with_connect_info::<SocketAddr>(),
        //     )
        //     .await
        //     .unwrap()
        // });
        // let _ = task.await;
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
