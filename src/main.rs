pub mod api;
pub mod domain;
pub mod state;
pub mod infrastructure;
use api::routes::routes;
use state::AppState;
use crate::infrastructure::kafka::KafkaProducer;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let kafka = KafkaProducer::new("localhost:9092"); // broker address
    let state = AppState {
        ledger: Arc::new(RwLock::new(domain::ledger::Ledger::new())),
        kafka,
    };

    let app = routes(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running at http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
