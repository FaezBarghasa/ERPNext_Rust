//! Stock Ledger, Balance Valuation, FIFO Batches, and Variant Matrix routes.

use super::*;
use actix_web::web;

/// Configures all Inventory and Stock management endpoints.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/api/v2/inventory/stock-entry",
        web::post().to(inventory_stock_entry_handler),
    )
    .route(
        "/api/v2/inventory/stock-balance",
        web::get().to(inventory_stock_balance_handler),
    )
    .route(
        "/api/v2/inventory/variant-matrix",
        web::post().to(inventory_variant_matrix_handler),
    );
}
