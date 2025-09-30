/* use std::sync::Arc;
use tokio::sync::RwLock;
use axum::{body::Body, http::Request, Router};
use tower::ServiceExt; // for app.oneshot(request)

use transaction_ledger::{
    api::routes::routes,
    domain::ledger::Ledger,
    infrastructure::kafka::KafkaProducer,
    state::AppState,
};

/// Setup application router for integration tests
pub async fn setup_app() -> Router {
   // let kafka = KafkaProducer::new("localhost:9092");

    let state = AppState {
        ledger: Arc::new(RwLock::new(Ledger::new())),
        kafka,
    };

    routes(state)
}

/// Helper to make requests in tests
pub async fn send_request(app: &Router, req: Request<Body>) -> axum::response::Response {
    app.clone().oneshot(req).await.unwrap()
} */
