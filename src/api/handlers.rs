use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use axum_macros::debug_handler;
use std::{collections::HashMap, path::PathBuf};

use crate::{
    api::dto::*,
    domain::{account::Account, ledger::Ledger, transaction::Transaction},
    state::AppState,
};

/// --- Account Handlers ---

#[debug_handler]
pub async fn create_account_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateAccountRequest>,
)-> Result<Json<CreateAccountResponse>,StatusCode> {
    let mut ledger = state.ledger.write().await;
    match ledger.create_account(payload.owner, payload.initial) {
        Ok(id) => Ok(Json(CreateAccountResponse { id })),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    } 
}


pub async fn get_balance_handler(
    State(state): State<AppState>,
    Path(account_id): Path<u32>,
) -> Result<Json<HashMap<&'static str, i64>>, (StatusCode, String)> {
    let ledger = state.ledger.read().await;
    match ledger.get_balance(account_id) {
        Some(bal) => {
            let mut map = HashMap::new();
            map.insert("balance", bal);
            Ok(Json(map))
        }
        None => Err((StatusCode::NOT_FOUND, "Account not found".into())),
    }
}


pub async fn find_account_by_owner_handler(
    State(state): State<AppState>,
    Query(query): Query<ListAccountsQuery>,
) -> Json<Vec<Account>> {
    let ledger = state.ledger.read().await;

    let accounts: Vec<Account> = match query.owner {
        Some(ref owner) => ledger
            .accounts
            .values()
            .filter(|a| a.owner == *owner)
            .cloned()
            .collect(),
        None => ledger.accounts.values().cloned().collect(),
    };

    Json(accounts)
}

/// --- Transaction Handlers ---

#[debug_handler]
pub async fn deposit_handler(
    State(state): State<AppState>,
    Json(req): Json<TransferRequest>,
) -> Result<Json<TxResponse>, (StatusCode, String)> {
    let mut ledger = state.ledger.write().await;
    match ledger.deposit(req.id, req.amount, req.description.clone()) {
        Ok(txid) => {
            let event = serde_json::json!({
                "type": "deposit",
                "account_id": req.id,
                "amount": req.amount,
                "description": req.description,
                "tx_id": txid
            }
            );

            state.kafka.send("transactions",&req.id.to_string(),&event.to_string()).await;
            Ok(Json(TxResponse { tx_id: txid }))
        }
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}


pub async fn withdraw_handler(
    State(state): State<AppState>,
    Json(req): Json<TransferRequest>,
) -> Result<Json<TxResponse>, (StatusCode, String)> {
    let mut ledger = state.ledger.write().await;
    match ledger.withdraw(req.id, req.amount, req.description.clone()) {
        Ok(txid) => {
            // ✅ Send Kafka event
            let event = serde_json::json!({
                "type": "withdrawal",
                "account_id": req.id,
                "amount": req.amount,
                "description": req.description,
                "tx_id": txid
            });
            state.kafka.send("transactions", &req.id.to_string(), &event.to_string()).await;

            Ok(Json(TxResponse { tx_id: txid }))
        },
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}


pub async fn transfer_handler(
    State(state): State<AppState>,
    Json(req): Json<TransferBetweenRequest>,
) -> Result<Json<TxResponse>, (StatusCode, String)> {
    let mut ledger = state.ledger.write().await;
    match ledger.transfer(req.from, req.to, req.amount, req.description.clone()) {
        Ok(txid) =>{ 
            // ✅ Send Kafka event
            let event = serde_json::json!({
                "type": "deposit",
                "from_id": req.from,
                "to_id": req.to,
                "amount": req.amount,
                "description": req.description,
                "tx_id": txid
            });
            let key = format!("{}->{}", req.from, req.to);
            state.kafka.send("transactions", &key, &event.to_string()).await;
            Ok(Json(TxResponse { tx_id: txid }))
        },
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

#[debug_handler]
pub async fn list_transactions_handler(
    State(state): State<AppState>,
    Query(q): Query<ListTxQuery>,
) -> Result<Json<Vec<Transaction>>, (StatusCode, String)> {
    let ledger = state.ledger.read().await;
    if let Some(acc_id) = q.account {
        let txs = ledger
            .transactions_for_account(acc_id)
            .into_iter()
            .cloned()
            .collect();
        Ok(Json(txs))
    } else {
        Ok(Json(ledger.transactions.clone()))
    }
}


/// --- Persistence Handlers (Save / Load) ---

pub async fn save_handler(
    State(state): State<AppState>,
    Json(req): Json<SaveLoadRequest>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    // Serialize while holding a read lock to get a consistent snapshot, then write to disk in blocking thread.
    let ledger_snapshot = {
        let ledger = state.ledger.read().await;
        serde_json::to_string_pretty(&*ledger).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };

    let path = PathBuf::from(req.path);
    let path_clone = path.clone();
    let write_result = tokio::task::spawn_blocking(move || std::fs::write(&path, ledger_snapshot))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::OK, format!("Saved ledger to {:?}", path_clone)))
}


pub async fn load_handler(
    State(state): State<AppState>,
    Json(req): Json<SaveLoadRequest>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let path = PathBuf::from(req.path);
    let path_clone = path.clone();
    // read + parse on blocking thread
    let loaded: Ledger = tokio::task::spawn_blocking(move || {
        let s = std::fs::read_to_string(&path)?;
        let ledger: Ledger = serde_json::from_str(&s)?;
        Ok::<Ledger, Box<dyn std::error::Error + Send + Sync>>(ledger)
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    // replace in-memory ledger
    let mut ledger_guard = state.ledger.write().await;
    *ledger_guard = loaded;

    Ok((StatusCode::OK, format!("Loaded ledger from {:?}", path_clone)))
}


/// --- Report Handler ---
pub async fn report_handler(State(state): State<AppState>) -> Json<HashMap<&'static str, String>> {
    let ledger = state.ledger.read().await;
    let mut map = HashMap::new();
    let total = ledger.total_assets();
    map.insert("total_assets", format!("{}", total));
    if let Some(acc) = ledger.richest_account() {
        map.insert("richest_account", format!("{} ({})", acc.id, acc.owner));
        map.insert("richest_balance", format!("{}", acc.balance));
    }
    Json(map)
}
