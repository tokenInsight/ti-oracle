use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, get_service},
    Router,
};
use std::{io, net::SocketAddr};
use log::{info};
use tower_http::{services::ServeDir};

pub async fn start(web_addr: String) {
    // build our application with a route
    let app = Router::new()
        .route("/", get_service(ServeDir::new("./static")).handle_error(handle_error))
        .route("/peers", get(peers));
    let addr: SocketAddr = web_addr.parse().expect("unable to parse web server address");
    info!("web listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn peers() -> &'static str {
    "Hello, World!"
}

async fn handle_error(_err: io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}