use crate::fetcher::PairInfo;
use crate::processor::gossip::ValidateResponse;
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, get_service},
    Extension, Json, Router,
};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::{io, net::SocketAddr};
use tower::ServiceBuilder;
use tower_http::services::ServeDir;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ChainEvent {
    pub coin_name: String,
    pub round: u64,
    pub feed_count: u64,
    pub peers_report: Vec<PeerReport>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PeerReport {
    pub price: u128,
    pub sig: String,
    pub timestamp: u64,
    pub address: String,
}

#[derive(Default)]
pub struct SharedStateData {
    pub peers_report: BTreeMap<u64, Vec<ValidateResponse>>,
    pub exchange_pairs: Vec<PairInfo>,
    pub peers: BTreeMap<String, u64>, //peer, timestamp
    pub chain_events: Vec<ChainEvent>,
}

pub type SharedState = Arc<Mutex<SharedStateData>>;

pub async fn start(web_addr: String, s_state: SharedState) {
    // build our application with a route
    let app = Router::new()
        .route(
            "/",
            get_service(ServeDir::new("./static")).handle_error(handle_error),
        )
        .route(
            "/pairs.html",
            get_service(ServeDir::new("./static")).handle_error(handle_error),
        )
        .route(
            "/events.html",
            get_service(ServeDir::new("./static")).handle_error(handle_error),
        )
        .route("/report", get(report))
        .route("/pairs", get(pairs))
        .route("/peers", get(peers))
        .route("/events", get(events))
        .layer(ServiceBuilder::new().layer(Extension(s_state)).into_inner());
    let addr: SocketAddr = web_addr
        .parse()
        .expect("unable to parse web server address");
    info!("web listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn report(Extension(state): Extension<SharedState>) -> impl IntoResponse {
    let report = state.lock().unwrap().peers_report.clone();
    (StatusCode::ACCEPTED, Json(report))
}

async fn pairs(Extension(state): Extension<SharedState>) -> impl IntoResponse {
    let exchange_pairs = state.lock().unwrap().exchange_pairs.clone();
    (StatusCode::ACCEPTED, Json(exchange_pairs))
}

async fn peers(Extension(state): Extension<SharedState>) -> impl IntoResponse {
    let peers = state.lock().unwrap().peers.clone();
    (StatusCode::ACCEPTED, Json(peers))
}

async fn events(Extension(state): Extension<SharedState>) -> impl IntoResponse {
    let events = state.lock().unwrap().chain_events.clone();
    (StatusCode::ACCEPTED, Json(events))
}

async fn handle_error(_err: io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}
