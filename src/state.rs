use std::sync::Arc;
use tokio::sync::RwLock;
use crate::domain::ledger::Ledger;
use crate::infrastructure::kafka::KafkaProducer;


#[derive(Clone)]
pub struct AppState {
    pub ledger: Arc<RwLock<Ledger>>,
    pub kafka: KafkaProducer,
}
