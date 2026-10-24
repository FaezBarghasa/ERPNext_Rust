//! Commerce, Order Lifecycle, Shipping, Coupons, Reviews, Wishlists, and RMA routes.

use super::*;
use actix_web::web;

/// Configures all Commerce and Trade endpoints.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/api/v2/trade/coupons/apply",
        web::post().to(trade_apply_coupon_handler),
    )
    .route(
        "/api/v2/trade/reviews",
        web::post().to(trade_submit_review_handler),
    )
    .route(
        "/api/v2/trade/reviews/{item_code}",
        web::get().to(trade_list_reviews_handler),
    )
    .route(
        "/api/v2/trade/wishlist/{user_id}",
        web::get().to(trade_get_wishlist_handler),
    )
    .route(
        "/api/v2/trade/wishlist",
        web::post().to(trade_add_wishlist_item_handler),
    )
    .route(
        "/api/v2/trade/wishlist/remove",
        web::post().to(trade_remove_wishlist_item_handler),
    )
    .route(
        "/api/v2/trade/shipping/calculate",
        web::post().to(trade_calculate_shipping_handler),
    )
    .route(
        "/api/v2/trade/orders/{order_id}/transition",
        web::post().to(trade_order_transition_handler),
    )
    .route(
        "/api/v2/trade/rma",
        web::post().to(trade_rma_submit_handler),
    )
    .route(
        "/api/v2/trade/rma/{rma_id}",
        web::get().to(trade_rma_get_handler),
    );
}
