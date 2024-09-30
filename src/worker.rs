use std::{thread, time::Duration};
use axum::routing::get;
use axum::Router;

pub async fn init() {
    println!("{}", crate::LOGO);
    println!("==> launching node in [worker] mode...");

    let app = Router::new()
        .route("/", get(hello));
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8888").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn hello() -> String {
    "hello world!".to_string()
}