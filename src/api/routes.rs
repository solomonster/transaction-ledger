use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::state::AppState;

use super::handlers::{
    create_account_handler, get_balance_handler, find_account_by_owner_handler,
    deposit_handler, withdraw_handler, transfer_handler,
    list_transactions_handler, save_handler, load_handler, report_handler,
};

/// Build the full application router.
pub fn routes(state:AppState) -> Router {
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any);

    Router::new()
        // Accounts
        .route("/accounts", post(create_account_handler).get(find_account_by_owner_handler))
        .route("/accounts/:id/balance", get(get_balance_handler))

        // Transactions
        .route("/deposit", post(deposit_handler))
        .route("/withdraw", post(withdraw_handler))
        .route("/transfer", post(transfer_handler))
        .route("/transactions", get(list_transactions_handler))

        // Persistence
        .route("/save", post(save_handler))
        .route("/load", post(load_handler))

        // Reports
        .route("/report", get(report_handler))

        // Add state here
        .with_state(state)
        // Add CORS
        .layer(cors)
}
