//! General Ledger, Journal Entries, Aging, and Financial Statements routes.

use super::*;
use actix_web::web;

/// Configures all Accounting, Ledger, and Financial Statement endpoints.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/api/v2/accounting/journal-entry",
        web::post().to(accounting_journal_entry_handler),
    )
    .route(
        "/api/v2/accounting/general-ledger",
        web::get().to(accounting_general_ledger_handler),
    )
    .route(
        "/api/v2/accounting/payment-entry",
        web::post().to(accounting_payment_entry_handler),
    )
    .route(
        "/api/v2/accounting/ar-aging",
        web::get().to(accounting_ar_aging_handler),
    )
    .route(
        "/api/v2/accounting/ap-aging",
        web::get().to(accounting_ap_aging_handler),
    )
    .route(
        "/api/v2/accounting/profit-loss",
        web::get().to(accounting_profit_loss_handler),
    )
    .route(
        "/api/v2/accounting/balance-sheet",
        web::get().to(accounting_balance_sheet_handler),
    )
    .route(
        "/api/v2/accounting/cash-flow",
        web::get().to(accounting_cash_flow_handler),
    )
    .route(
        "/api/v2/accounting/trial-balance",
        web::get().to(accounting_trial_balance_handler),
    )
    .route(
        "/api/v2/accounting/period-close",
        web::post().to(accounting_period_close_handler),
    );
}
