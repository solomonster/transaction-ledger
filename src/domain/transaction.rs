use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::api::dto::Kobo;

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct TransactionEntry {
    pub account_id: u32,
    pub debit: Kobo,
    pub credit: Kobo,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Transaction {
    pub id: u64,
    pub description: Option<String>,
    pub entries: Vec<TransactionEntry>,
    pub timestamp: DateTime<Utc>, // ISO-8601 string for skeleton
}