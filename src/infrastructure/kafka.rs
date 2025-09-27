
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer,FutureRecord};
use std::time::Duration;

#[derive(Clone)]
pub struct KafkaProducer {
    producer: FutureProducer,
}

impl KafkaProducer {
    pub fn new(brokers: &str) -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Producer creation failed");

        KafkaProducer { producer }
    }

    pub async fn send(&self, topic: &str, key: &str, payload: &str) {
        let record= FutureRecord::to(topic).key(key).payload(payload);

        match self.producer.send(record, Duration::from_secs(0)).await {
            Ok(delivery) => {
                println!("✅ Kafka delivery: {:?}", delivery);
            }
            Err((err, _msg)) => {
                eprintln!("❌ Kafka error: {:?}", err);
            }
        }
    }
}