use axum::{
    extract::Request,
    http::{StatusCode, HeaderMap},
    middleware::{self, Next},
    response::Response
};

crate::config;


async pub fn authorise(headers: HeaderMap, request: Request, next: Next) -> Result<Response, StatusCode> {
    if let Some(config) = config::get() {
        if let Some(token) = headers.get("x-rdfs-token") {
            if token == config.token {
                let response = next.run(request).await;
                return Ok(response)
            }
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}