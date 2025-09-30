use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
//use hyper::Body;
use serde_json::json;
use transaction_ledger::api::dto::CreateAccountResponse;
use transaction_ledger::domain::currency::Currency;
use transaction_ledger::domain::ledger::{Ledger};
use tower::ServiceExt; // 👈 Import this for oneshot

mod common;

//#[tokio::test]
/*async fn test_create_account() {
    let app = common::setup_app().await;

    let payload = serde_json::json!({
        "owner": "Alice",
        "initial": 1000,
        "bank_name": "Test Bank",
        "bank_code": "123",
        "currency": Currency::NGN,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/accounts")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Parse response
    let body = axum::body::to_bytes(response.into_body(),usize::MAX).await.unwrap();
    let account: CreateAccountResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(account.bank_code, "123");
    assert_eq!(account.bank_name, "Test Bank");
    assert_eq!(account.currency, Currency::NGN);

    assert!(!account.account_number.is_empty());
}

#[tokio::test]
async fn test_deposit_increases_balance() {
    let app = common::setup_app().await;

    // Create account
    let payload = serde_json::json!({
        "owner": "Bob",
        "initial": 0,
        "bank_name": "Test Bank",
        "bank_code": "321",
        "currency": "USD"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/accounts")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(),usize::MAX).await.unwrap();
    let account: CreateAccountResponse = serde_json::from_slice(&body).unwrap();

    // Deposit into that account by ID
    let deposit_payload = serde_json::json!({
        "id": account.id,   // 👈 still using ID
        "amount": 500,
        "description": "Initial deposit"
    });

    let deposit_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/deposit")
                .header("content-type", "application/json")
                .body(Body::from(deposit_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(deposit_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_transfer_between_accounts() {
    let app = common::setup_app().await;
    //let app = Arc::new(common::setup_app().await);

    // Create "from" account with initial 1000
    let from_payload = serde_json::json!({
        "owner": "Charlie",
        "initial": 1000,
        "bank_name": "Bank A",
        "bank_code": "111",
        "currency": "NGN"
    });

    let from_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/accounts")
                .header("content-type", "application/json")
                .body(Body::from(from_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(from_response.into_body(),usize::MAX).await.unwrap();
    let from_account: CreateAccountResponse = serde_json::from_slice(&body).unwrap();

    // Create "to" account with initial 0
    let to_payload = serde_json::json!({
        "owner": "Daisy",
        "initial": 0,
        "bank_name": "Bank B",
        "bank_code": "222",
        "currency": "NGN"
    });

    let to_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/accounts")
                .header("content-type", "application/json")
                .body(Body::from(to_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(to_response.into_body(),usize::MAX).await.unwrap();
    let to_account: CreateAccountResponse = serde_json::from_slice(&body).unwrap();

    // Transfer 300 from Charlie -> Daisy
    let transfer_payload = serde_json::json!({
        "from": from_account.id,
        "to": to_account.id,
        "amount": 300,
        "description": "Payment"
    });

    let transfer_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/transfer")
                .header("content-type", "application/json")
                .body(Body::from(transfer_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(transfer_response.status(), StatusCode::OK);

    // ✅ Check balances after transfer
    let from_balance_resp = app
    .clone()
    .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/accounts/{}/balance", from_account.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let to_balance_resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/accounts/{}/balance", to_account.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let from_balance_body = axum::body::to_bytes(from_balance_resp.into_body(),usize::MAX).await.unwrap();
    let from_balance: serde_json::Value = serde_json::from_slice(&from_balance_body).unwrap();

    let to_balance_body = axum::body::to_bytes(to_balance_resp.into_body(),usize::MAX).await.unwrap();
    let to_balance: serde_json::Value = serde_json::from_slice(&to_balance_body).unwrap();

    assert_eq!(from_balance["balance"], 700); // 1000 - 300
    assert_eq!(to_balance["balance"], 300);   // 0 + 300
}

#[tokio::test]
async fn test_withdraw_reduces_balance() {
    let app = common::setup_app().await;

    // Step 1: Create an account
    let create_payload = json!({
        "owner": "Alice",
        "initial": 5000,
        "currency": "NGN",
        "bank_name": "MyBank",
        "bank_code": "123",
    });

    let request = Request::builder()
        .method("POST")
        .uri("/accounts")
        .header("Content-Type", "application/json")
        .body(Body::from(create_payload.to_string()))
        .unwrap();

    let response = common::send_request(&app, request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(),usize::MAX).await.unwrap();
    let account: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let account_id = account["id"].as_u64().unwrap();

    // Step 2: Withdraw from the account
    let withdraw_payload = json!({
        "id": account_id,
        "amount": 2000,
        "description": "ATM Withdrawal"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/withdraw")
        .header("Content-Type", "application/json")
        .body(Body::from(withdraw_payload.to_string()))
        .unwrap();

    let response = common::send_request(&app, request).await;
    assert_eq!(response.status(), StatusCode::OK);

    // Step 3: Check balance
    let uri = format!("/accounts/{}/balance", account_id);
    let request = Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap();

    let response = common::send_request(&app, request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(),usize::MAX).await.unwrap();
    let balance: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(balance["balance"].as_i64().unwrap(), 3000); // 5000 - 2000
}*/
