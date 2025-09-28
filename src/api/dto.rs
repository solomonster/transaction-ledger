use serde::{Deserialize, Serialize};

use crate::domain::currency::Currency;

/// Kobo type alias (₦1 = 100 Kobo)
pub type Kobo = i64;

/// --- Account DTOs ---
#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub owner: String,
    pub initial: Kobo,
    pub currency: Currency,
}

#[derive(Debug, Serialize)]
pub struct CreateAccountResponse {
    pub id: u32,
    pub currency: Currency,
}

/// --- Deposit / Withdraw DTO ---
#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    pub id: u32,
    pub amount: Kobo,
    pub description: Option<String>,
}

/// --- Transfer Between Accounts DTO ---
#[derive(Debug, Deserialize)]
pub struct TransferBetweenRequest {
    pub from: u32,
    pub to: u32,
    pub amount: Kobo,
    pub description: Option<String>,
}

/// --- Transaction response ---
#[derive(Debug, Serialize)]
pub struct TxResponse {
    pub tx_id: u64,
}

/// --- Save / Load DTO ---
#[derive(Debug, Deserialize)]
pub struct SaveLoadRequest {
    pub path: String,
}

/// --- Query DTOs ---
#[derive(Debug, Deserialize)]
pub struct ListAccountsQuery {
    pub owner: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListTxQuery {
    pub account: Option<u32>,
}
