//! Production Hypermedia Application Server (`frappe-net::server`).
//!
//! Provides:
//! - Multi-Persona Desktop & Portal SSR Shells (`/desk`, `/portal`, `/worker`, `/factory`, `/approvals`, `/admin`)
//! - REST API v1 & v2 Document/RPC Routers with Tenant Isolation, Typestate Lifecycle and RBAC
//! - SurrealDB-backed Dynamic Storefront, Atomic Checkout, and Payment Webhook Receivers
//! - Content-Addressable Storage (CAS) Upload & Download Streams
//! - Visual CMS Builder Persistence API with SQL Injection Hardening
//! - Tracing Observability, Prometheus Metrics, Kubernetes Health Probes (`/healthz/live`, `/healthz/ready`), Token-Bucket Rate Limiter, and ACME Security Gateway

use crate::live::live_ws_handler;
use crate::middleware::auth::SecurityContext;
use crate::rate_limit::RateLimitMiddleware;
use crate::routes::{create_resource, delete_resource, get_resource, list_resource};
use crate::tenant::{
    AcmeGateway, ConnectionPoolManager, MicroTopologyConfig, TenantId, TenantResolver,
};
use crate::v2_routes::{
    DynamicRbacState, accounting_ap_aging_handler, accounting_ar_aging_handler,
    accounting_balance_sheet_handler, accounting_cash_flow_handler,
    accounting_general_ledger_handler, accounting_journal_entry_handler,
    accounting_payment_entry_handler, accounting_period_close_handler,
    accounting_profit_loss_handler, accounting_trial_balance_handler, admin_action_handler,
    admin_create_ip_rule_handler, admin_create_role_handler, admin_create_user_handler,
    admin_delete_ip_rule_handler, admin_delete_user_handler, admin_get_audit_logs_handler,
    admin_get_permissions_handler, admin_get_user_effective_permissions_handler,
    admin_list_ip_rules_handler, admin_list_lockouts_handler, admin_list_roles_handler,
    admin_list_users_handler, admin_query_handler, admin_status_handler,
    admin_unlock_target_handler, admin_update_permission_handler, admin_update_user_roles_handler,
    audio_add_comment_handler, audio_crossfade_handler, audio_eq_filter_handler,
    audio_flac_compress_handler, audio_get_comments_handler, audio_lyrics_current_handler,
    audio_lyrics_parse_handler, audio_music_info_handler, audio_normalize_handler,
    audio_recommend_handler, audio_sleep_soundboard_handler, audio_sleep_timer_handler,
    audio_waveform_handler, auth_forgot_password_handler, auth_mfa_activate_handler,
    auth_mfa_disable_handler, auth_mfa_enroll_handler, auth_refresh_token_handler,
    auth_reset_password_handler, builder_ai_prompt_handler, builder_get_page_handler,
    builder_history_list_handler, builder_list_pages_handler, builder_save_page_handler,
    buying_purchase_invoice_handler, buying_purchase_order_handler,
    buying_purchase_receipt_handler, cms_create_taxonomy_handler, cms_generate_seo_handler,
    cms_list_media_handler, cms_list_taxonomies_handler, cms_upload_media_handler,
    crm_convert_quotation_handler, download_file_handler, download_image_handler,
    email_list_queue_handler, email_send_handler, emdash_get_document_handler,
    emdash_list_documents_handler, emdash_media_crop_handler, emdash_revisions_list_handler,
    emdash_save_document_handler, emdash_transition_handler, export_dataset_handler,
    h3_stream_file_handler, h3_stream_telemetry_handler, hr_process_payroll_handler,
    inventory_stock_balance_handler, inventory_stock_entry_handler,
    inventory_variant_matrix_handler, login_handler, logout_handler,
    manufacturing_complete_work_order_handler, manufacturing_work_order_handler,
    notifications_dispatch_handler, notifications_get_inbox_handler, payment_charge_handler,
    payment_webhook_handler, ping_handler, portal_customer_dashboard_handler,
    portal_supplier_dashboard_handler, print_document_handler, quic_status_handler,
    render_invoice_handler, render_receipt_handler, report_pivot_table_handler,
    selling_delivery_note_handler, selling_sales_invoice_handler, setup_wizard_handler,
    streaming_cache_configure_handler, streaming_cache_prefetch_handler,
    streaming_cache_stats_handler, streaming_fast_epg_handler,
    streaming_fast_live_manifest_handler, streaming_fast_status_handler,
    streaming_h3_segment_handler, streaming_master_playlist_handler,
    streaming_playback_progress_handler, streaming_resilient_master_playlist_handler,
    streaming_rtmp_auth_handler, streaming_rtmp_done_handler, streaming_segment_handler,
    streaming_upload_chunk_handler, streaming_variant_playlist_handler,
    streaming_video_xray_handler, streaming_video_xray_vtt_handler, stremio_catalog_handler,
    stremio_manifest_handler, stremio_meta_handler, stremio_stream_handler,
    stremio_subtitles_handler, subsonic_get_license_handler, subsonic_get_song_handler,
    subsonic_ping_handler, subsonic_stream_handler, trade_add_wishlist_item_handler,
    trade_apply_coupon_handler, trade_calculate_shipping_handler, trade_create_coupon_handler,
    trade_get_wishlist_handler, trade_list_coupons_handler, trade_list_reviews_handler,
    trade_list_shipping_zones_handler, trade_moderate_review_handler,
    trade_order_transition_handler, trade_remove_wishlist_item_handler, trade_rma_get_handler,
    trade_rma_submit_handler, trade_submit_review_handler, upload_file_handler, v2_amend_document,
    v2_cancel_document, v2_create_document, v2_delete_document, v2_get_document,
    v2_get_meta_handler, v2_list_document, v2_list_meta_handler, v2_print_document,
    v2_submit_document, v2_update_document, webhooks_dispatch_test_handler,
    workflow_evaluate_handler, workflow_version_history_handler, workflow_version_rollback_handler,
};
use actix_web::{
    App, HttpMessage, HttpRequest, HttpResponse, HttpServer, Responder, middleware::Compress,
    middleware::Logger, middleware::NormalizePath, web,
};
use desk_components::{
    get_desk_workspaces, render_desk_shell_html, render_form_view_html, render_list_view_html,
};
use frappe_framework::{ControllerRegistry, WebhookDispatcher};
use frappe_meta::rbac::{Permission, check_permission};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tracing::{info, instrument};

static TOTAL_REQUESTS: AtomicU64 = AtomicU64::new(0);
static START_TIME: std::sync::LazyLock<Instant> = std::sync::LazyLock::new(Instant::now);

/// Sanitizes a CMS page slug to prevent SQL injection and directory traversal.
#[must_use]
pub fn sanitize_slug(slug: &str) -> Option<String> {
    let trimmed = slug.trim();
    if !trimmed.is_empty()
        && trimmed.len() <= 128
        && trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        Some(trimmed.replace('-', "_"))
    } else {
        None
    }
}

#[instrument]
async fn health_check() -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "runtime": "tokio+surrealdb",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

/// Kubernetes liveness probe: `GET /healthz/live`
#[instrument]
async fn liveness_handler() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "alive",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

/// Kubernetes readiness probe: `GET /healthz/ready`
#[instrument(skip(pool_mgr))]
async fn readiness_handler(pool_mgr: web::Data<ConnectionPoolManager>) -> impl Responder {
    match pool_mgr
        .get_or_initialize_client(&TenantId("default".into()))
        .await
    {
        Ok(client) => match client.query("RETURN 1;").await {
            Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                "status": "ready",
                "db": "connected",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            })),
            Err(e) => HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "status": "not_ready",
                "db": "unavailable",
                "error": e.to_string(),
            })),
        },
        Err(e) => HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "not_ready",
            "db": "connection_failed",
            "error": e.to_string(),
        })),
    }
}

/// Prometheus metrics exposition: `GET /metrics`
async fn metrics_handler() -> impl Responder {
    let total_reqs = TOTAL_REQUESTS.load(Ordering::Relaxed);
    let uptime_secs = START_TIME.elapsed().as_secs();

    let metrics_text = format!(
        "# HELP rustnext_http_requests_total Total number of HTTP requests processed.\n\
         # TYPE rustnext_http_requests_total counter\n\
         rustnext_http_requests_total {}\n\
         # HELP rustnext_uptime_seconds Total runtime uptime in seconds.\n\
         # TYPE rustnext_uptime_seconds gauge\n\
         rustnext_uptime_seconds {}\n\
         # HELP rustnext_active_tenant_sessions Active cached tenant sessions count.\n\
         # TYPE rustnext_active_tenant_sessions gauge\n\
         rustnext_active_tenant_sessions 1\n",
        total_reqs, uptime_secs
    );

    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(metrics_text)
}

async fn desk_handler(req: HttpRequest) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let user_name = req
        .extensions()
        .get::<SecurityContext>()
        .map(|ctx| ctx.claims.sub.clone())
        .unwrap_or_else(|| "Administrator".to_string());

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(render_desk_shell_html("Enterprise Desk", &user_name))
}

async fn desk_list_view_handler(req: HttpRequest, path: web::Path<String>) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let doctype = path.into_inner();
    let user_name = req
        .extensions()
        .get::<SecurityContext>()
        .map(|ctx| ctx.claims.sub.clone())
        .unwrap_or_else(|| "Administrator".to_string());

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(render_list_view_html(&doctype, &user_name))
}

async fn desk_form_view_handler(
    req: HttpRequest,
    path: web::Path<(String, String)>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let (doctype, docname) = path.into_inner();
    let user_name = req
        .extensions()
        .get::<SecurityContext>()
        .map(|ctx| ctx.claims.sub.clone())
        .unwrap_or_else(|| "Administrator".to_string());

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(render_form_view_html(&doctype, &docname, &user_name))
}

async fn portal_handler(req: HttpRequest) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let user_name = req
        .extensions()
        .get::<SecurityContext>()
        .map(|ctx| ctx.claims.sub.clone())
        .unwrap_or_else(|| "Customer".to_string());

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(render_desk_shell_html("Customer Portal", &user_name))
}

async fn worker_handler(req: HttpRequest) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let user_name = req
        .extensions()
        .get::<SecurityContext>()
        .map(|ctx| ctx.claims.sub.clone())
        .unwrap_or_else(|| "Operator".to_string());

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(desk_app::render_worker_kiosk_html(&user_name))
}

async fn factory_handler(req: HttpRequest) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let user_name = req
        .extensions()
        .get::<SecurityContext>()
        .map(|ctx| ctx.claims.sub.clone())
        .unwrap_or_else(|| "Factory Manager".to_string());

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(render_desk_shell_html("SCADA Floor Cockpit", &user_name))
}

async fn approvals_handler(req: HttpRequest) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let user_name = req
        .extensions()
        .get::<SecurityContext>()
        .map(|ctx| ctx.claims.sub.clone())
        .unwrap_or_else(|| "Executive".to_string());

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(render_desk_shell_html(
            "Executive Approvals Hub",
            &user_name,
        ))
}

async fn admin_handler(req: HttpRequest) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let user_name = req
        .extensions()
        .get::<SecurityContext>()
        .map(|ctx| ctx.claims.sub.clone())
        .unwrap_or_else(|| "Administrator".to_string());

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(desk_app::render_admin_cockpit_html(&user_name))
}

async fn storefront_handler() -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(erp_cms::render_luxury_storefront_html())
}

/// Frappe Builder Studio Visual Interface: `GET /builder` and `GET /builder/{page_id}`
async fn builder_studio_handler(page_id: Option<web::Path<String>>) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let target_page = page_id
        .map(|p| p.into_inner())
        .unwrap_or_else(|| "home".into());
    let switcher = erp_cms::builder_core::DeviceFrameSwitcher {
        breakpoint: erp_cms::builder_core::CanvasBreakpoint::Desktop,
        is_landscape: false,
    };
    let frame_style = switcher.frame_css_style();

    let html = format!(
        r###"<!DOCTYPE html>
<html lang="en" data-theme="dark">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RustNext Frappe Builder Studio — {target_page}</title>
    <style>
        :root {{
            --bg-canvas: #0b0d14;
            --bg-sidebar: #131622;
            --bg-card: rgba(255, 255, 255, 0.04);
            --border: rgba(255, 255, 255, 0.1);
            --accent: #6366f1;
            --accent-hover: #4f46e5;
            --success: #10b981;
            --text-main: #f8fafc;
            --text-muted: #94a3b8;
        }}
        * {{ box-sizing: border-box; }}
        body {{
            margin: 0;
            padding: 0;
            background: var(--bg-canvas);
            color: var(--text-main);
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            display: flex;
            height: 100vh;
            overflow: hidden;
        }}
        .builder-sidebar {{
            width: 300px;
            background: var(--bg-sidebar);
            border-right: 1px solid var(--border);
            display: flex;
            flex-direction: column;
            padding: 1.25rem;
            gap: 1rem;
            overflow-y: auto;
        }}
        .builder-inspector-sidebar {{
            width: 320px;
            background: var(--bg-sidebar);
            border-left: 1px solid var(--border);
            display: flex;
            flex-direction: column;
            padding: 1.25rem;
            gap: 1rem;
            overflow-y: auto;
        }}
        .builder-workspace {{
            flex: 1;
            display: flex;
            flex-direction: column;
            overflow: hidden;
        }}
        .builder-toolbar {{
            height: 56px;
            background: var(--bg-sidebar);
            border-bottom: 1px solid var(--border);
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 0 1.5rem;
        }}
        .builder-stage {{
            flex: 1;
            padding: 2rem;
            display: flex;
            justify-content: center;
            align-items: flex-start;
            overflow: auto;
            background: radial-gradient(circle, rgba(99,102,241,0.08) 1px, transparent 1px);
            background-size: 24px 24px;
        }}
        .device-frame {{
            {frame_style}
            box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.7);
            color: #0f172a;
            position: relative;
            background: #ffffff;
            min-height: 600px;
            border-radius: 8px;
            overflow: hidden;
            transition: width 0.3s cubic-bezier(0.4, 0, 0.2, 1);
        }}
        .palette-btn {{
            background: var(--bg-card);
            border: 1px solid var(--border);
            padding: 0.75rem 1rem;
            border-radius: 8px;
            cursor: pointer;
            font-size: 0.875rem;
            color: var(--text-main);
            display: flex;
            align-items: center;
            justify-content: space-between;
            transition: all 0.15s ease;
            text-align: left;
            width: 100%;
        }}
        .palette-btn:hover {{
            background: rgba(99, 102, 241, 0.15);
            border-color: var(--accent);
        }}
        .btn {{
            background: var(--accent);
            color: #fff;
            border: none;
            padding: 0.5rem 1rem;
            border-radius: 6px;
            cursor: pointer;
            font-weight: 600;
            font-size: 0.85rem;
            display: inline-flex;
            align-items: center;
            gap: 0.5rem;
            transition: background 0.15s ease;
        }}
        .btn:hover {{ background: var(--accent-hover); }}
        .btn-outline {{
            background: rgba(255,255,255,0.08);
            border: 1px solid var(--border);
        }}
        .btn-outline:hover {{
            background: rgba(255,255,255,0.15);
        }}
        .btn-success {{ background: var(--success); }}
        .btn-success:hover {{ background: #059669; }}
        .canvas-node-wrapper {{
            position: relative;
            margin: 0.5rem 0;
            padding: 0.5rem;
            border: 1px dashed transparent;
            border-radius: 6px;
            transition: all 0.15s ease;
            cursor: pointer;
        }}
        .canvas-node-wrapper:hover {{
            border-color: var(--accent);
        }}
        .canvas-node-wrapper.selected-node {{
            outline: 2px solid var(--accent);
            box-shadow: 0 0 0 4px rgba(99, 102, 241, 0.25);
            background: rgba(99, 102, 241, 0.03);
        }}
        .canvas-node-wrapper.dragging {{
            opacity: 0.4;
        }}
        .canvas-node-wrapper.drag-over {{
            border-top: 3px solid var(--accent) !important;
        }}
        .node-actions {{
            position: absolute;
            top: 2px;
            right: 2px;
            display: none;
            gap: 0.25rem;
            background: rgba(15, 23, 42, 0.9);
            padding: 2px 6px;
            border-radius: 4px;
            z-index: 10;
        }}
        .canvas-node-wrapper:hover > .node-actions {{
            display: flex;
        }}
        .node-act-btn {{
            background: transparent;
            border: none;
            color: #f8fafc;
            cursor: pointer;
            font-size: 0.75rem;
            padding: 2px 4px;
            border-radius: 3px;
        }}
        .node-act-btn:hover {{ background: rgba(255, 255, 255, 0.2); }}
        .drag-handle {{
            cursor: grab;
            margin-right: 4px;
            color: #94a3b8;
            font-size: 0.9rem;
            user-select: none;
        }}
        .drag-handle:active {{ cursor: grabbing; }}
        .toast-pill {{
            position: fixed;
            bottom: 1.5rem;
            right: 1.5rem;
            background: #1e293b;
            color: #f8fafc;
            border: 1px solid var(--border);
            padding: 0.75rem 1.25rem;
            border-radius: 8px;
            box-shadow: 0 10px 25px rgba(0,0,0,0.5);
            display: none;
            align-items: center;
            gap: 0.5rem;
            font-size: 0.875rem;
            z-index: 9999;
        }}
        .inspector-field {{
            display: flex;
            flex-direction: column;
            gap: 0.35rem;
            margin-bottom: 0.85rem;
        }}
        .inspector-field label {{
            font-size: 0.75rem;
            color: var(--text-muted);
            text-transform: uppercase;
            letter-spacing: 0.05em;
            font-weight: 600;
        }}
        .inspector-input {{
            background: rgba(0, 0, 0, 0.4);
            border: 1px solid var(--border);
            color: #fff;
            padding: 0.5rem;
            border-radius: 6px;
            font-size: 0.85rem;
            outline: none;
        }}
        .inspector-input:focus {{
            border-color: var(--accent);
        }}
        .tab-btn-group {{
            display: flex;
            background: rgba(0, 0, 0, 0.3);
            padding: 3px;
            border-radius: 6px;
            border: 1px solid var(--border);
            margin-bottom: 1rem;
        }}
        .tab-btn {{
            flex: 1;
            background: transparent;
            border: none;
            color: var(--text-muted);
            padding: 0.4rem;
            border-radius: 4px;
            cursor: pointer;
            font-size: 0.75rem;
            font-weight: 600;
            transition: all 0.15s ease;
        }}
        .tab-btn.active {{
            background: var(--accent);
            color: #fff;
        }}
    </style>
</head>
<body>
    <aside class="builder-sidebar">
        <div style="font-weight: 700; font-size: 1.1rem; display: flex; align-items: center; gap: 0.5rem;">
            <span>⚡</span> Frappe Builder Studio
        </div>
        <div style="font-size: 0.85rem; color: var(--text-muted); display: flex; justify-content: space-between;">
            <span>Editing Page:</span>
            <strong style="color: var(--accent);">{target_page}</strong>
        </div>
        
        <div style="font-weight: 600; font-size: 0.85rem; text-transform: uppercase; letter-spacing: 0.05em; color: var(--text-muted); margin-top: 0.5rem;">Palette</div>
        <button class="palette-btn" onclick="addBlock('Heading')">🏷️ Heading <span>+</span></button>
        <button class="palette-btn" onclick="addBlock('TextBlock')">📝 Text Paragraph <span>+</span></button>
        <button class="palette-btn" onclick="addBlock('Section')">📦 Section Container <span>+</span></button>
        <button class="palette-btn" onclick="addBlock('CardBlock')">💳 Product Card <span>+</span></button>
        <button class="palette-btn" onclick="addBlock('NumberCardBlock')">🔢 KPI Metric Card <span>+</span></button>
        <button class="palette-btn" onclick="addBlock('Navbar')">🧭 Navigation Bar <span>+</span></button>
        <button class="palette-btn" onclick="addBlock('Footer')">⚓ Page Footer <span>+</span></button>

        <div style="margin-top: auto; border-top: 1px solid var(--border); padding-top: 1rem;">
            <div style="font-weight: 600; font-size: 0.85rem; color: var(--text-muted); margin-bottom: 0.5rem; display: flex; align-items: center; gap: 0.4rem;">
                <span>✨</span> Autonomous Bob AI
            </div>
            <input type="text" id="ai-prompt" placeholder="e.g. Generate pricing grid..." style="width: 100%; background: rgba(0,0,0,0.4); border: 1px solid var(--border); color: #fff; padding: 0.6rem; border-radius: 6px; margin-bottom: 0.5rem; font-size: 0.85rem;">
            <button class="btn" style="width: 100%; justify-content: center;" onclick="callBobAi()">✨ Synthesize AST</button>
        </div>
    </aside>

    <main class="builder-workspace">
        <header class="builder-toolbar">
            <div style="display: flex; gap: 0.5rem; align-items: center;">
                <button class="btn btn-outline" id="btn-device-desktop" onclick="setDevice('desktop')">🖥️ Desktop</button>
                <button class="btn btn-outline" id="btn-device-tablet" onclick="setDevice('tablet')">📱 Tablet</button>
                <button class="btn btn-outline" id="btn-device-mobile" onclick="setDevice('mobile')">📲 Mobile</button>
                <div style="width: 1px; height: 20px; background: var(--border); margin: 0 4px;"></div>
                <button class="btn btn-outline" onclick="undo()" title="Undo (Ctrl+Z)">↶ Undo</button>
                <button class="btn btn-outline" onclick="redo()" title="Redo (Ctrl+Y)">↷ Redo</button>
            </div>
            <div style="display: flex; align-items: center; gap: 0.75rem;">
                <span id="ast-counter" style="font-size: 0.8rem; color: var(--text-muted);">0 blocks</span>
                <span id="version-badge" style="font-size: 0.75rem; background: rgba(255,255,255,0.1); padding: 2px 8px; border-radius: 12px; color: #a5b4fc;">v0</span>
                <a href="/preview/{target_page}" target="_blank" class="btn btn-outline" style="text-decoration: none;">👁️ Preview</a>
                <button class="btn btn-success" onclick="savePage()">💾 Save to SurrealDB</button>
            </div>
        </header>

        <section class="builder-stage">
            <div class="device-frame" id="canvas-frame">
                <div id="canvas-content" style="padding: 2rem;">
                    <!-- Rendered AST block hierarchy -->
                </div>
            </div>
        </section>
    </main>

    <aside class="builder-inspector-sidebar" id="inspector-sidebar">
        <div style="font-weight: 700; font-size: 1.05rem; display: flex; align-items: center; justify-content: space-between;">
            <span>⚙️ Inspector</span>
            <span id="inspector-node-type" style="font-size: 0.75rem; background: var(--accent); padding: 2px 6px; border-radius: 4px; color: #fff;">None</span>
        </div>

        <div class="tab-btn-group">
            <button class="tab-btn active" id="tab-desktop" onclick="setInspectorBreakpoint('Desktop')">Desktop</button>
            <button class="tab-btn" id="tab-tablet" onclick="setInspectorBreakpoint('Tablet')">Tablet</button>
            <button class="tab-btn" id="tab-mobile" onclick="setInspectorBreakpoint('Mobile')">Mobile</button>
        </div>

        <div id="inspector-controls">
            <div style="color: var(--text-muted); font-size: 0.85rem; text-align: center; padding: 2rem 0;">
                Click any block on the canvas to inspect its style cascade, slot, and properties.
            </div>
        </div>
    </aside>

    <div id="toast" class="toast-pill"></div>

    <script>
        const targetPage = '{target_page}';
        window.activeTree = {{
            nodes: {{}},
            root_ids: [],
            next_id: 0,
            version: 0
        }};
        window.selectedNodeId = null;
        window.activeBreakpoint = 'Desktop';
        let draggedNodeId = null;
        window.historyStack = [];
        window.redoStack = [];

        function pushHistory() {{
            try {{
                window.historyStack.push(JSON.stringify(window.activeTree));
                if (window.historyStack.length > 50) window.historyStack.shift();
                window.redoStack = [];
            }} catch (_) {{}}
        }}

        function undo() {{
            if (!window.historyStack || window.historyStack.length === 0) {{
                showToast('Nothing to undo');
                return;
            }}
            window.redoStack.push(JSON.stringify(window.activeTree));
            const prev = window.historyStack.pop();
            window.activeTree = JSON.parse(prev);
            renderCanvas();
            showToast('↶ Undo');
        }}

        function redo() {{
            if (!window.redoStack || window.redoStack.length === 0) {{
                showToast('Nothing to redo');
                return;
            }}
            window.historyStack.push(JSON.stringify(window.activeTree));
            const next = window.redoStack.pop();
            window.activeTree = JSON.parse(next);
            renderCanvas();
            showToast('↷ Redo');
        }}

        document.addEventListener('keydown', (e) => {{
            if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA' || e.target.isContentEditable) {{
                return;
            }}
            if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'z') {{
                e.preventDefault();
                if (e.shiftKey) {{
                    redo();
                }} else {{
                    undo();
                }}
            }} else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'y') {{
                e.preventDefault();
                redo();
            }} else if (e.key === 'Delete' || e.key === 'Backspace') {{
                if (window.selectedNodeId) {{
                    e.preventDefault();
                    pushHistory();
                    removeNode(window.selectedNodeId);
                }}
            }}
        }});

        function showToast(msg, isSuccess = true) {{
            const t = document.getElementById('toast');
            t.textContent = msg;
            t.style.borderColor = isSuccess ? '#10b981' : '#ef4444';
            t.style.display = 'flex';
            setTimeout(() => {{ t.style.display = 'none'; }}, 3000);
        }}

        function setDevice(device) {{
            const frame = document.getElementById('canvas-frame');
            if (device === 'mobile') {{
                frame.style.width = '375px';
                setInspectorBreakpoint('Mobile');
            }} else if (device === 'tablet') {{
                frame.style.width = '768px';
                setInspectorBreakpoint('Tablet');
            }} else {{
                frame.style.width = '1280px';
                setInspectorBreakpoint('Desktop');
            }}
        }}

        function setInspectorBreakpoint(bp) {{
            window.activeBreakpoint = bp;
            document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
            const btn = document.getElementById('tab-' + bp.toLowerCase());
            if (btn) btn.classList.add('active');
            renderInspector();
        }}

        function updateAstCounter() {{
            const count = Object.keys(window.activeTree.nodes).length;
            document.getElementById('ast-counter').textContent = `${{count}} AST blocks`;
            document.getElementById('version-badge').textContent = `v${{window.activeTree.version || 0}}`;
        }}

        function selectNode(nodeId, e) {{
            if (e) e.stopPropagation();
            window.selectedNodeId = nodeId;
            document.querySelectorAll('.canvas-node-wrapper').forEach(w => w.classList.remove('selected-node'));
            const targetEl = document.getElementById('wrap_' + nodeId);
            if (targetEl) targetEl.classList.add('selected-node');
            renderInspector();
        }}

        function renderInspector() {{
            const container = document.getElementById('inspector-controls');
            const typeBadge = document.getElementById('inspector-node-type');
            if (!window.selectedNodeId || !window.activeTree.nodes[window.selectedNodeId]) {{
                typeBadge.textContent = 'None';
                container.innerHTML = `
                    <div style="color: var(--text-muted); font-size: 0.85rem; text-align: center; padding: 2rem 0;">
                        Click any block on the canvas to inspect its style cascade, slot, and properties.
                    </div>`;
                return;
            }}

            const node = window.activeTree.nodes[window.selectedNodeId];
            const b = node.block;
            typeBadge.textContent = b.block_type || 'Block';

            // Ensure responsive_styles structure
            if (!node.responsive_styles) {{
                node.responsive_styles = {{
                    desktop: node.css_styles || {{}},
                    tablet: null,
                    mobile: null
                }};
            }}
            const activeStyles = (window.activeBreakpoint === 'Mobile') 
                ? (node.responsive_styles.mobile || {{}}) 
                : (window.activeBreakpoint === 'Tablet') 
                ? (node.responsive_styles.tablet || {{}}) 
                : (node.responsive_styles.desktop || {{}});

            let html = `
                <div class="inspector-field">
                    <label>Node ID</label>
                    <input class="inspector-input" value="${{node.id}}" readonly style="opacity: 0.6;">
                </div>
                <div class="inspector-field">
                    <label>Component Slot</label>
                    <select class="inspector-input" onchange="updateNodeSlot(this.value)">
                        <option value="" ${{!node.slot ? 'selected' : ''}}>(Default/None)</option>
                        <option value="header" ${{node.slot === 'header' ? 'selected' : ''}}>header</option>
                        <option value="main" ${{node.slot === 'main' ? 'selected' : ''}}>main</option>
                        <option value="sidebar" ${{node.slot === 'sidebar' ? 'selected' : ''}}>sidebar</option>
                        <option value="footer" ${{node.slot === 'footer' ? 'selected' : ''}}>footer</option>
                    </select>
                </div>
            `;

            if (b.text !== undefined) {{
                html += `
                    <div class="inspector-field">
                        <label>Content / Text</label>
                        <input class="inspector-input" value="${{b.text}}" oninput="updateBlockText(this.value)">
                    </div>`;
            }} else if (b.title !== undefined) {{
                html += `
                    <div class="inspector-field">
                        <label>Title</label>
                        <input class="inspector-input" value="${{b.title}}" oninput="updateBlockTitle(this.value)">
                    </div>`;
            }}

            html += `
                <div style="font-weight: 600; font-size: 0.8rem; text-transform: uppercase; color: var(--text-muted); margin: 1rem 0 0.5rem 0;">
                    CSS Style Override (${{window.activeBreakpoint}})
                </div>
                <div class="inspector-field">
                    <label>Background Color</label>
                    <input class="inspector-input" type="text" value="${{activeStyles.background_color || ''}}" placeholder="#ffffff or rgba(...)" oninput="updateStyle('background_color', this.value)">
                </div>
                <div class="inspector-field">
                    <label>Text Color</label>
                    <input class="inspector-input" type="text" value="${{activeStyles.text_color || ''}}" placeholder="#0f172a" oninput="updateStyle('text_color', this.value)">
                </div>
                <div class="inspector-field">
                    <label>Padding</label>
                    <input class="inspector-input" type="text" value="${{activeStyles.padding || ''}}" placeholder="1rem" oninput="updateStyle('padding', this.value)">
                </div>
                <div class="inspector-field">
                    <label>Margin</label>
                    <input class="inspector-input" type="text" value="${{activeStyles.margin || ''}}" placeholder="0.5rem 0" oninput="updateStyle('margin', this.value)">
                </div>
            `;

            container.innerHTML = html;
        }}

        function updateNodeSlot(slotVal) {{
            if (!window.selectedNodeId) return;
            pushHistory();
            window.activeTree.nodes[window.selectedNodeId].slot = slotVal || null;
            renderCanvas();
        }}

        function updateBlockText(textVal) {{
            if (!window.selectedNodeId) return;
            window.activeTree.nodes[window.selectedNodeId].block.text = textVal;
            const targetEl = document.getElementById('inner_' + window.selectedNodeId);
            if (targetEl) targetEl.textContent = textVal;
        }}

        function updateBlockTitle(titleVal) {{
            if (!window.selectedNodeId) return;
            window.activeTree.nodes[window.selectedNodeId].block.title = titleVal;
            renderCanvas();
        }}

        function updateStyle(prop, val) {{
            if (!window.selectedNodeId) return;
            pushHistory();
            const node = window.activeTree.nodes[window.selectedNodeId];
            if (!node.responsive_styles) {{
                node.responsive_styles = {{ desktop: {{}}, tablet: null, mobile: null }};
            }}
            const bpKey = window.activeBreakpoint.toLowerCase();
            if (!node.responsive_styles[bpKey]) {{
                node.responsive_styles[bpKey] = {{}};
            }}
            node.responsive_styles[bpKey][prop] = val || null;
            renderCanvas();
        }}

        // HTML5 Drag and Drop handlers
        function handleDragStart(e, nodeId) {{
            draggedNodeId = nodeId;
            e.dataTransfer.setData('text/plain', nodeId);
            e.dataTransfer.effectAllowed = 'move';
            e.currentTarget.classList.add('dragging');
        }}

        function handleDragOver(e, nodeId) {{
            e.preventDefault();
            e.dataTransfer.dropEffect = 'move';
            const wrap = document.getElementById('wrap_' + nodeId);
            if (wrap && draggedNodeId !== nodeId) {{
                wrap.classList.add('drag-over');
            }}
        }}

        function handleDragLeave(e, nodeId) {{
            const wrap = document.getElementById('wrap_' + nodeId);
            if (wrap) wrap.classList.remove('drag-over');
        }}

        function handleDrop(e, targetNodeId) {{
            e.preventDefault();
            const wrap = document.getElementById('wrap_' + targetNodeId);
            if (wrap) wrap.classList.remove('drag-over');

            if (!draggedNodeId || draggedNodeId === targetNodeId) return;

            const rootIds = window.activeTree.root_ids;
            const srcIdx = rootIds.indexOf(draggedNodeId);
            const dstIdx = rootIds.indexOf(targetNodeId);

            if (srcIdx !== -1 && dstIdx !== -1) {{
                pushHistory();
                rootIds.splice(srcIdx, 1);
                rootIds.splice(dstIdx, 0, draggedNodeId);
                renderCanvas();
                showToast('Reordered canvas blocks');
            }}
            draggedNodeId = null;
        }}

        function handleDragEnd(e) {{
            document.querySelectorAll('.canvas-node-wrapper').forEach(w => {{
                w.classList.remove('dragging');
                w.classList.remove('drag-over');
            }});
            draggedNodeId = null;
        }}

        function renderCanvas() {{
            const container = document.getElementById('canvas-content');
            container.innerHTML = '';

            if (window.activeTree.root_ids.length === 0) {{
                container.innerHTML = `
                    <div style="text-align: center; padding: 4rem 2rem; color: #64748b;">
                        <h2 style="margin-bottom: 0.5rem; color: #1e293b;">Empty Canvas</h2>
                        <p>Click a palette item or ask Bob AI to synthesize your page layout.</p>
                    </div>`;
                updateAstCounter();
                renderInspector();
                return;
            }}

            for (const rootId of window.activeTree.root_ids) {{
                container.appendChild(renderNode(rootId));
            }}
            updateAstCounter();
            renderInspector();
        }}

        function renderNode(nodeId) {{
            const node = window.activeTree.nodes[nodeId];
            if (!node) return document.createElement('div');

            const wrap = document.createElement('div');
            wrap.className = 'canvas-node-wrapper' + (window.selectedNodeId === nodeId ? ' selected-node' : '');
            wrap.id = 'wrap_' + nodeId;
            wrap.draggable = true;

            wrap.ondragstart = (e) => handleDragStart(e, nodeId);
            wrap.ondragover = (e) => handleDragOver(e, nodeId);
            wrap.ondragleave = (e) => handleDragLeave(e, nodeId);
            wrap.ondrop = (e) => handleDrop(e, nodeId);
            wrap.ondragend = handleDragEnd;
            wrap.onclick = (e) => selectNode(nodeId, e);

            if (node.slot) {{
                wrap.setAttribute('data-slot', node.slot);
            }}

            const actions = document.createElement('div');
            actions.className = 'node-actions';
            actions.innerHTML = `
                <span class="drag-handle" title="Drag to reorder">⠿</span>
                <span style="font-size: 0.7rem; color: #cbd5e1; margin-right: 4px;">${{node.slot ? `[slot: ${{node.slot}}]` : ''}}</span>
                <button class="node-act-btn" onclick="removeNode('${{nodeId}}'); event.stopPropagation();" title="Remove">✕</button>
            `;
            wrap.appendChild(actions);

            const b = node.block;
            if (b.block_type === 'Heading') {{
                const el = document.createElement('h' + (b.level || 2));
                el.id = 'inner_' + nodeId;
                el.textContent = b.text;
                el.contentEditable = 'true';
                el.onblur = (e) => {{ b.text = e.target.textContent; renderInspector(); }};
                wrap.appendChild(el);
            }} else if (b.block_type === 'TextBlock') {{
                const el = document.createElement('p');
                el.id = 'inner_' + nodeId;
                el.textContent = b.content || b.text || '';
                el.contentEditable = 'true';
                el.onblur = (e) => {{ b.content = e.target.textContent; b.text = e.target.textContent; renderInspector(); }};
                wrap.appendChild(el);
            }} else if (b.block_type === 'Section' || b.block_type === 'Container') {{
                const sec = document.createElement('section');
                sec.style.padding = '1.5rem';
                sec.style.background = '#f8fafc';
                sec.style.borderRadius = '8px';
                sec.style.border = '1px solid #e2e8f0';
                for (const cid of (node.child_ids || [])) {{
                    sec.appendChild(renderNode(cid));
                }}
                wrap.appendChild(sec);
            }} else if (b.block_type === 'CardBlock') {{
                const card = document.createElement('div');
                card.style.padding = '1.5rem';
                card.style.background = '#ffffff';
                card.style.border = '1px solid #e2e8f0';
                card.style.borderRadius = '8px';
                card.style.boxShadow = '0 4px 6px -1px rgba(0,0,0,0.1)';
                card.innerHTML = `<h3 style="margin-top:0;">${{b.title}}</h3><p style="color:#64748b;">${{b.body}}</p>`;
                wrap.appendChild(card);
            }} else if (b.block_type === 'NumberCardBlock') {{
                const num = document.createElement('div');
                num.style.padding = '1.25rem';
                num.style.background = '#eef2ff';
                num.style.borderRadius = '8px';
                num.style.textAlign = 'center';
                num.innerHTML = `<div style="font-size:0.8rem;font-weight:700;color:#6366f1;">${{b.label}}</div><div style="font-size:2rem;font-weight:800;color:#1e1b4b;">99.98%</div>`;
                wrap.appendChild(num);
            }} else if (b.block_type === 'Navbar') {{
                const nav = document.createElement('nav');
                nav.style.display = 'flex';
                nav.style.justifyContent = 'space-between';
                nav.style.alignItems = 'center';
                nav.style.padding = '1rem 1.5rem';
                nav.style.background = '#1e1b4b';
                nav.style.color = '#fff';
                nav.style.borderRadius = '6px';
                nav.innerHTML = `<div style="font-weight:700;">${{b.brand_title}}</div><div style="display:flex;gap:1rem;">${{(b.links || []).map(l => `<a href="${{l[1]}}" style="color:#c7d2fe;text-decoration:none;">${{l[0]}}</a>`).join('')}}</div>`;
                wrap.appendChild(nav);
            }} else if (b.block_type === 'Footer') {{
                const foot = document.createElement('footer');
                foot.style.padding = '1.5rem';
                foot.style.textAlign = 'center';
                foot.style.color = '#94a3b8';
                foot.style.borderTop = '1px solid #e2e8f0';
                foot.innerHTML = `<p style="margin:0;font-size:0.85rem;">${{b.copyright_text}}</p>`;
                wrap.appendChild(foot);
            }} else {{
                const raw = document.createElement('div');
                raw.textContent = `[${{b.block_type}}]`;
                wrap.appendChild(raw);
            }}

            return wrap;
        }}

        function addBlock(kind) {{
            window.activeTree.next_id++;
            const id = 'blk_' + window.activeTree.next_id;
            let block = null;

            if (kind === 'Heading') {{
                block = {{ block_type: 'Heading', level: 2, text: 'New Heading', data_binding: null }};
            }} else if (kind === 'TextBlock') {{
                block = {{ block_type: 'TextBlock', content: 'Crafted with Frappe Builder in pure Rust.', data_binding: null }};
            }} else if (kind === 'Section') {{
                block = {{ block_type: 'Section', id: id, children: [], css_classes: 'py-8' }};
            }} else if (kind === 'CardBlock') {{
                block = {{ block_type: 'CardBlock', title: 'Product Showcase', body: 'ACID transaction velocity on the cloud edge.', image_url: null }};
            }} else if (kind === 'NumberCardBlock') {{
                block = {{ block_type: 'NumberCardBlock', label: 'Cluster Throughput', value_data_path: 'pageData.tps', format: null }};
            }} else if (kind === 'Navbar') {{
                block = {{ block_type: 'Navbar', brand_title: 'Enterprise Studio', links: [['Home', '/'], ['Catalog', '/site/catalog']] }};
            }} else if (kind === 'Footer') {{
                block = {{ block_type: 'Footer', copyright_text: '© 2026 RustNext Enterprise. Pure Rust Engine.' }};
            }}

            if (block) {{
                pushHistory();
                window.activeTree.nodes[id] = {{
                    id: id,
                    block: block,
                    parent_id: null,
                    child_ids: [],
                    css_styles: null,
                    responsive_styles: null,
                    slot: null
                }};
                window.activeTree.root_ids.push(id);
                renderCanvas();
                selectNode(id);
                showToast(`Added ${{kind}} block`);
            }}
        }}

        function removeNode(nodeId) {{
            pushHistory();
            delete window.activeTree.nodes[nodeId];
            window.activeTree.root_ids = window.activeTree.root_ids.filter(i => i !== nodeId);
            if (window.selectedNodeId === nodeId) window.selectedNodeId = null;
            renderCanvas();
            showToast('Block removed');
        }}

        async function savePage() {{
            try {{
                const res = await fetch('/api/v2/builder/pages/' + targetPage, {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{
                        title: targetPage,
                        route_path: '/site/' + targetPage,
                        tree: window.activeTree,
                        expected_version: window.activeTree.version
                    }})
                }});
                const data = await res.json();
                if (res.status === 409) {{
                    showToast('⚠️ Conflict: Concurrent update detected! Reloading latest version...', false);
                    setTimeout(initBuilder, 1200);
                    return;
                }}
                if (res.ok) {{
                    window.activeTree.version = data.tree ? data.tree.version : (window.activeTree.version + 1);
                    updateAstCounter();
                    showToast(`✓ Page saved to SurrealDB (v${{window.activeTree.version}})`);
                }} else {{
                    showToast('Failed to save page: ' + (data.error || 'unknown error'), false);
                }}
            }} catch (e) {{
                showToast('Error saving: ' + e.message, false);
            }}
        }}

        async function callBobAi() {{
            const prompt = document.getElementById('ai-prompt').value.trim();
            if (!prompt) return;
            try {{
                const res = await fetch('/api/v2/builder/ai/prompt', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{
                        prompt: prompt,
                        mode: 'generate',
                        selected_block_id: window.selectedNodeId,
                        active_theme_id: 'default',
                        target_doctypes: ['Item'],
                        attached_image_url: null
                    }})
                }});
                const data = await res.json();
                if (data.synthesized_block) {{
                    window.activeTree.next_id++;
                    const id = 'blk_ai_' + window.activeTree.next_id;
                    window.activeTree.nodes[id] = {{
                        id: id,
                        block: data.synthesized_block,
                        parent_id: null,
                        child_ids: [],
                        css_styles: null,
                        responsive_styles: null,
                        slot: null
                    }};
                    window.activeTree.root_ids.push(id);
                    renderCanvas();
                    selectNode(id);
                    showToast('✓ Bob AI synthesized layout added to canvas');
                    document.getElementById('ai-prompt').value = '';
                }}
            }} catch (e) {{
                showToast('Bob AI Error: ' + e.message, false);
            }}
        }}

        // Initial Load
        async function initBuilder() {{
            try {{
                const res = await fetch('/api/v2/builder/pages/' + targetPage);
                if (res.ok) {{
                    const data = await res.json();
                    if (data.tree && data.tree.root_ids) {{
                        window.activeTree = data.tree;
                        if (data.tree.version !== undefined) {{
                            window.activeTree.version = data.tree.version;
                        }}
                        renderCanvas();
                        return;
                    }}
                }}
            }} catch (_) {{}}
            
            // Starter default AST
            addBlock('Navbar');
            addBlock('Heading');
            addBlock('TextBlock');
            addBlock('CardBlock');
            addBlock('Footer');
        }}

        document.addEventListener('DOMContentLoaded', initBuilder);
    </script>
</body>
</html>"###,
        target_page = target_page,
        frame_style = frame_style
    );

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

/// Emdash Editorial Studio Web Interface: `GET /emdash` and `GET /emdash/editor/{id}`
async fn emdash_studio_handler(doc_id: Option<web::Path<String>>) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let target_doc = doc_id
        .map(|p| p.into_inner())
        .unwrap_or_else(|| "new-post".into());

    let html = format!(
        r###"<!DOCTYPE html>
<html lang="en" data-theme="dark">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Emdash Editorial Studio — {target_doc}</title>
    <style>
        :root {{
            --bg-canvas: #090a0f;
            --bg-editor: #13151f;
            --border: rgba(255, 255, 255, 0.08);
            --accent: #10b981;
            --accent-hover: #059669;
            --text-main: #f8fafc;
            --text-muted: #94a3b8;
        }}
        * {{ box-sizing: border-box; }}
        body {{
            margin: 0;
            padding: 0;
            background: var(--bg-canvas);
            color: var(--text-main);
            font-family: -apple-system, BlinkMacSystemFont, "Georgia", serif;
            display: flex;
            flex-direction: column;
            height: 100vh;
        }}
        header {{
            height: 56px;
            background: var(--bg-editor);
            border-bottom: 1px solid var(--border);
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 0 2rem;
            font-family: -apple-system, BlinkMacSystemFont, sans-serif;
        }}
        .editor-container {{
            flex: 1;
            max-width: 860px;
            width: 100%;
            margin: 1.5rem auto;
            padding: 2rem;
            overflow-y: auto;
            position: relative;
        }}
        .post-title {{
            font-size: 2.25rem;
            font-weight: 700;
            border: none;
            background: transparent;
            color: var(--text-main);
            width: 100%;
            outline: none;
            margin-bottom: 1rem;
            font-family: inherit;
        }}
        .btn {{
            background: var(--accent);
            color: #fff;
            border: none;
            padding: 0.5rem 1.25rem;
            border-radius: 6px;
            cursor: pointer;
            font-weight: 600;
            font-family: -apple-system, BlinkMacSystemFont, sans-serif;
            font-size: 0.85rem;
            transition: all 0.15s ease;
        }}
        .btn:hover {{ background: var(--accent-hover); }}
        .btn-outline {{
            background: rgba(255, 255, 255, 0.08);
            border: 1px solid var(--border);
        }}
        .btn-outline:hover {{ background: rgba(255, 255, 255, 0.15); }}
        .meta-bar {{
            font-family: -apple-system, BlinkMacSystemFont, sans-serif;
            font-size: 0.85rem;
            color: var(--text-muted);
            margin-bottom: 1.5rem;
            display: flex;
            gap: 1.5rem;
            align-items: center;
            border-bottom: 1px solid var(--border);
            padding-bottom: 1rem;
        }}
        .block-toolbar {{
            display: flex;
            gap: 0.5rem;
            margin-bottom: 1.5rem;
            padding: 0.5rem;
            background: var(--bg-editor);
            border-radius: 8px;
            border: 1px solid var(--border);
            font-family: -apple-system, BlinkMacSystemFont, sans-serif;
        }}
        .tool-btn {{
            background: transparent;
            border: none;
            color: var(--text-main);
            cursor: pointer;
            padding: 0.4rem 0.75rem;
            border-radius: 4px;
            font-size: 0.85rem;
            font-weight: 500;
        }}
        .tool-btn:hover {{ background: rgba(255, 255, 255, 0.1); }}
        .emdash-block-row {{
            position: relative;
            margin-bottom: 1.25rem;
            padding: 0.75rem;
            border-radius: 6px;
            border: 1px solid transparent;
            transition: border-color 0.15s ease;
        }}
        .emdash-block-row:hover {{ border-color: var(--border); }}
        .block-del-btn {{
            position: absolute;
            top: 4px;
            right: 4px;
            background: transparent;
            border: none;
            color: #ef4444;
            cursor: pointer;
            display: none;
            font-size: 0.85rem;
        }}
        .emdash-block-row:hover .block-del-btn {{ display: block; }}
        .callout-box {{
            padding: 1rem 1.25rem;
            border-left: 4px solid var(--accent);
            background: rgba(16, 185, 129, 0.08);
            border-radius: 4px;
            font-family: -apple-system, BlinkMacSystemFont, sans-serif;
        }}
        .code-box {{
            background: #0f172a;
            color: #38bdf8;
            padding: 1rem;
            border-radius: 6px;
            font-family: monospace;
            font-size: 0.9rem;
            white-space: pre-wrap;
        }}
        .media-box {{
            border: 1px solid var(--border);
            border-radius: 8px;
            overflow: hidden;
            background: #0f172a;
            padding: 1rem;
            text-align: center;
        }}
        .media-box img {{
            max-width: 100%;
            height: auto;
            border-radius: 4px;
        }}
        .toast-pill {{
            position: fixed;
            bottom: 1.5rem;
            right: 1.5rem;
            background: #1e293b;
            color: #f8fafc;
            border: 1px solid var(--border);
            padding: 0.75rem 1.25rem;
            border-radius: 8px;
            box-shadow: 0 10px 25px rgba(0,0,0,0.5);
            display: none;
            align-items: center;
            gap: 0.5rem;
            font-size: 0.875rem;
            font-family: -apple-system, BlinkMacSystemFont, sans-serif;
            z-index: 9999;
        }}
        /* Floating Slash Command Palette */
        #slash-palette {{
            position: absolute;
            background: #181b29;
            border: 1px solid rgba(255, 255, 255, 0.15);
            border-radius: 8px;
            box-shadow: 0 20px 40px rgba(0, 0, 0, 0.7);
            width: 240px;
            display: none;
            flex-direction: column;
            z-index: 1000;
            overflow: hidden;
            font-family: -apple-system, BlinkMacSystemFont, sans-serif;
        }}
        .slash-item {{
            padding: 0.6rem 1rem;
            display: flex;
            align-items: center;
            gap: 0.75rem;
            cursor: pointer;
            color: var(--text-main);
            font-size: 0.85rem;
            transition: background 0.12s ease;
        }}
        .slash-item:hover, .slash-item.active {{
            background: rgba(16, 185, 129, 0.2);
            color: #6ee7b7;
        }}
        /* Floating Inline Format Toolbar */
        #inline-format-toolbar {{
            position: absolute;
            background: #1e2235;
            border: 1px solid rgba(255, 255, 255, 0.15);
            border-radius: 6px;
            padding: 4px;
            display: none;
            gap: 2px;
            z-index: 1000;
            box-shadow: 0 8px 24px rgba(0, 0, 0, 0.6);
            font-family: -apple-system, BlinkMacSystemFont, sans-serif;
        }}
        .fmt-btn {{
            background: transparent;
            border: none;
            color: #f8fafc;
            padding: 4px 8px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 0.85rem;
            font-weight: 600;
            line-height: 1;
        }}
        .fmt-btn:hover {{
            background: rgba(255, 255, 255, 0.15);
        }}
    </style>
</head>
<body>
    <header>
        <div style="font-weight: 700; display: flex; align-items: center; gap: 0.5rem;">
            <span>✍️</span> Emdash Editorial Studio
        </div>
        <div style="display: flex; gap: 0.5rem; align-items: center;">
            <span id="version-badge" style="font-size: 0.75rem; background: rgba(255,255,255,0.1); padding: 2px 8px; border-radius: 12px; color: #6ee7b7; font-family: -apple-system, sans-serif;">v0</span>
            <a href="/site/{target_doc}" id="view-live-link" target="_blank" style="color: #6ee7b7; font-size: 0.85rem; text-decoration: none; margin-right: 0.5rem; display: none;">Live Link ↗</a>
            <button class="btn btn-outline" onclick="saveDraft()">Save Draft</button>
            <button class="btn" onclick="publishPost()">Publish Now</button>
        </div>
    </header>

    <main class="editor-container" id="editor-container">
        <div class="meta-bar">
            <span>Status: <strong id="doc-status" style="color: #f59e0b;">Draft</strong></span>
            <span>Words: <strong id="word-count">0</strong></span>
            <span>Reading Time: <strong id="reading-time">1 min</strong></span>
            <span>Slug: <strong style="color: #6ee7b7;">{target_doc}</strong></span>
        </div>

        <input class="post-title" id="post-title" value="Architecting Next-Generation Enterprise Systems with Pure Rust" placeholder="Article Title...">

        <div class="block-toolbar">
            <span style="font-size: 0.85rem; color: var(--text-muted); display: flex; align-items: center; margin-right: 0.5rem;">Insert:</span>
            <button class="tool-btn" onclick="insertBlock('Paragraph')">+ Paragraph</button>
            <button class="tool-btn" onclick="insertBlock('Heading')">+ Heading</button>
            <button class="tool-btn" onclick="insertBlock('Quote')">+ Quote</button>
            <button class="tool-btn" onclick="insertBlock('Callout')">+ Callout</button>
            <button class="tool-btn" onclick="insertBlock('Code')">+ Code</button>
            <button class="tool-btn" onclick="insertBlock('Table')">+ Table</button>
            <button class="tool-btn" onclick="insertBlock('Image')">+ Image</button>
        </div>

        <div id="blocks-container">
            <!-- Dynamic Emdash Blocks -->
        </div>

        <!-- Floating Slash Command Palette -->
        <div id="slash-palette">
            <div class="slash-item" onclick="applySlash('Heading1')"><span>🏷️</span> Heading 1 (/h1)</div>
            <div class="slash-item" onclick="applySlash('Heading2')"><span>🏷️</span> Heading 2 (/h2)</div>
            <div class="slash-item" onclick="applySlash('Heading3')"><span>🏷️</span> Heading 3 (/h3)</div>
            <div class="slash-item" onclick="applySlash('Paragraph')"><span>📝</span> Text Paragraph (/p)</div>
            <div class="slash-item" onclick="applySlash('Quote')"><span>💬</span> Blockquote (/quote)</div>
            <div class="slash-item" onclick="applySlash('Callout')"><span>💡</span> Callout Box (/callout)</div>
            <div class="slash-item" onclick="applySlash('Code')"><span>💻</span> Code Snippet (/code)</div>
            <div class="slash-item" onclick="applySlash('Table')"><span>📊</span> Data Table (/table)</div>
            <div class="slash-item" onclick="applySlash('Image')"><span>🖼️</span> Responsive Image (/image)</div>
        </div>

        <!-- Floating Inline Format Toolbar -->
        <div id="inline-format-toolbar">
            <button class="fmt-btn" onclick="formatText('bold')" title="Bold"><b>B</b></button>
            <button class="fmt-btn" onclick="formatText('italic')" title="Italic"><i>I</i></button>
            <button class="fmt-btn" onclick="formatText('underline')" title="Underline"><u>U</u></button>
            <button class="fmt-btn" onclick="formatText('strikeThrough')" title="Strikethrough"><s>S</s></button>
            <button class="fmt-btn" onclick="promptLink()" title="Hyperlink">🔗</button>
        </div>
    </main>

    <div id="toast" class="toast-pill"></div>

    <script>
        const docId = '{target_doc}';
        window.emdashBlocks = [];
        window.documentVersion = 0;
        let slashTargetIdx = null;

        function showToast(msg, isSuccess = true) {{
            const t = document.getElementById('toast');
            t.textContent = msg;
            t.style.borderColor = isSuccess ? '#10b981' : '#ef4444';
            t.style.display = 'flex';
            setTimeout(() => {{ t.style.display = 'none'; }}, 3000);
        }}

        function updateStats() {{
            let words = 0;
            for (const b of window.emdashBlocks) {{
                if (b.text) words += b.text.split(/\s+/).filter(Boolean).length;
                if (b.quote) words += b.quote.split(/\s+/).filter(Boolean).length;
                if (b.code) words += b.code.split(/\s+/).filter(Boolean).length;
            }}
            document.getElementById('word-count').textContent = words;
            document.getElementById('reading-time').textContent = Math.max(1, Math.ceil(words / 200)) + ' min';
            document.getElementById('version-badge').textContent = `v${{window.documentVersion || 0}}`;
        }}

        function renderBlocks() {{
            const c = document.getElementById('blocks-container');
            c.innerHTML = '';

            window.emdashBlocks.forEach((b, idx) => {{
                const row = document.createElement('div');
                row.className = 'emdash-block-row';
                row.id = 'block_row_' + idx;
                row.innerHTML = `<button class="block-del-btn" onclick="removeBlock(${{idx}})" title="Delete Block">✕</button>`;

                if (b.block_type === 'Heading') {{
                    const h = document.createElement('h' + (b.level || 2));
                    h.textContent = b.text;
                    h.contentEditable = 'true';
                    h.style.outline = 'none';
                    attachInputListeners(h, idx, (val) => {{ b.text = val; }});
                    row.appendChild(h);
                }} else if (b.block_type === 'Paragraph') {{
                    const p = document.createElement('p');
                    p.textContent = b.text;
                    p.contentEditable = 'true';
                    p.style.outline = 'none';
                    p.style.lineHeight = '1.8';
                    p.style.fontSize = '1.15rem';
                    p.style.color = '#cbd5e1';
                    attachInputListeners(p, idx, (val) => {{ b.text = val; }});
                    row.appendChild(p);
                }} else if (b.block_type === 'Quote') {{
                    const q = document.createElement('blockquote');
                    q.style.borderLeft = '4px solid var(--accent)';
                    q.style.paddingLeft = '1rem';
                    q.style.color = '#94a3b8';
                    q.style.fontStyle = 'italic';
                    q.contentEditable = 'true';
                    q.style.outline = 'none';
                    q.textContent = b.quote || b.text || '';
                    attachInputListeners(q, idx, (val) => {{ b.quote = val; b.text = val; }});
                    row.appendChild(q);
                }} else if (b.block_type === 'Callout') {{
                    const box = document.createElement('div');
                    box.className = 'callout-box';
                    box.contentEditable = 'true';
                    box.style.outline = 'none';
                    box.textContent = b.text;
                    attachInputListeners(box, idx, (val) => {{ b.text = val; }});
                    row.appendChild(box);
                }} else if (b.block_type === 'Code') {{
                    const pre = document.createElement('pre');
                    pre.className = 'code-box';
                    pre.contentEditable = 'true';
                    pre.style.outline = 'none';
                    pre.textContent = b.code;
                    attachInputListeners(pre, idx, (val) => {{ b.code = val; }});
                    row.appendChild(pre);
                }} else if (b.block_type === 'Table') {{
                    const tbl = document.createElement('table');
                    tbl.style.width = '100%';
                    tbl.style.borderCollapse = 'collapse';
                    tbl.style.margin = '1rem 0';
                    const thead = document.createElement('thead');
                    const hr = document.createElement('tr');
                    (b.headers || ['Column 1', 'Column 2']).forEach((h, hidx) => {{
                        const th = document.createElement('th');
                        th.style.border = '1px solid var(--border)';
                        th.style.padding = '0.5rem 0.75rem';
                        th.style.textAlign = 'left';
                        th.style.background = 'rgba(255,255,255,0.05)';
                        th.contentEditable = 'true';
                        th.textContent = h;
                        th.onblur = () => {{ b.headers[hidx] = th.textContent; updateStats(); }};
                        hr.appendChild(th);
                    }});
                    thead.appendChild(hr);
                    tbl.appendChild(thead);
                    const tbody = document.createElement('tbody');
                    (b.rows || [['Value 1', 'Value 2']]).forEach((rowVals, ridx) => {{
                        const tr = document.createElement('tr');
                        rowVals.forEach((cval, cidx) => {{
                            const td = document.createElement('td');
                            td.style.border = '1px solid var(--border)';
                            td.style.padding = '0.5rem 0.75rem';
                            td.contentEditable = 'true';
                            td.textContent = cval;
                            td.onblur = () => {{ b.rows[ridx][cidx] = td.textContent; updateStats(); }};
                            tr.appendChild(td);
                        }});
                        tbody.appendChild(tr);
                    }});
                    tbl.appendChild(tbody);
                    row.appendChild(tbl);
                }} else if (b.block_type === 'Image') {{
                    const imgBox = document.createElement('div');
                    imgBox.className = 'media-box';
                    const src = b.url || '/files/image/placeholder?w=800&h=400&fmt=webp';
                    imgBox.innerHTML = `
                        <img src="${{src}}" alt="${{b.caption || 'Editorial Media'}}">
                        <div style="font-size:0.8rem;color:var(--text-muted);margin-top:0.5rem;" contenteditable="true" onblur="window.emdashBlocks[${{idx}}].caption = this.textContent;">${{b.caption || 'Click to edit caption...'}}</div>
                    `;
                    row.appendChild(imgBox);
                }}

                c.appendChild(row);
            }});
            updateStats();
        }}

        function attachInputListeners(el, idx, onUpdate) {{
            el.onblur = (e) => {{
                onUpdate(e.target.textContent);
                updateStats();
            }};
            el.onkeyup = (e) => {{
                const text = el.textContent;
                if (text === '/' || text.endsWith(' /')) {{
                    showSlashPalette(el, idx);
                }} else if (e.key === 'Escape') {{
                    hideSlashPalette();
                }}
            }};
        }}

        function showSlashPalette(targetEl, idx) {{
            slashTargetIdx = idx;
            const p = document.getElementById('slash-palette');
            const rect = targetEl.getBoundingClientRect();
            const containerRect = document.getElementById('editor-container').getBoundingClientRect();
            
            p.style.top = (rect.bottom - containerRect.top + 8) + 'px';
            p.style.left = (rect.left - containerRect.left) + 'px';
            p.style.display = 'flex';
        }}

        function hideSlashPalette() {{
            const p = document.getElementById('slash-palette');
            p.style.display = 'none';
            slashTargetIdx = null;
        }}

        function applySlash(kind) {{
            if (slashTargetIdx === null) return;
            const idx = slashTargetIdx;
            hideSlashPalette();

            if (kind === 'Heading1') {{
                window.emdashBlocks[idx] = {{ block_type: 'Heading', level: 1, text: 'Main Heading' }};
            }} else if (kind === 'Heading2') {{
                window.emdashBlocks[idx] = {{ block_type: 'Heading', level: 2, text: 'Sub Heading' }};
            }} else if (kind === 'Heading3') {{
                window.emdashBlocks[idx] = {{ block_type: 'Heading', level: 3, text: 'Section Heading' }};
            }} else if (kind === 'Paragraph') {{
                window.emdashBlocks[idx] = {{ block_type: 'Paragraph', text: 'Enter text here...' }};
            }} else if (kind === 'Quote') {{
                window.emdashBlocks[idx] = {{ block_type: 'Quote', quote: 'A memorable insight...', citation: 'Author' }};
            }} else if (kind === 'Callout') {{
                window.emdashBlocks[idx] = {{ block_type: 'Callout', style: 'Info', title: 'Note', text: 'Key takeaway message here.' }};
            }} else if (kind === 'Code') {{
                window.emdashBlocks[idx] = {{ block_type: 'Code', language: 'rust', code: '// Rust snippet\\nfn main() {{}}' }};
            }} else if (kind === 'Table') {{
                window.emdashBlocks[idx] = {{ block_type: 'Table', headers: ['Metric', 'Benchmark'], rows: [['Latency', '< 1ms'], ['Throughput', '25,000 req/s']] }};
            }} else if (kind === 'Image') {{
                window.emdashBlocks[idx] = {{ block_type: 'Image', url: '/files/image/sample?w=800&h=450&fmt=webp', caption: 'High-resolution diagram' }};
            }}
            renderBlocks();
        }}

        // Inline Formatting Toolbar
        document.addEventListener('selectionchange', () => {{
            const sel = window.getSelection();
            const tb = document.getElementById('inline-format-toolbar');
            if (!sel || sel.isCollapsed || !sel.rangeCount) {{
                tb.style.display = 'none';
                return;
            }}
            const range = sel.getRangeAt(0);
            const rect = range.getBoundingClientRect();
            const containerRect = document.getElementById('editor-container').getBoundingClientRect();
            
            if (rect.width > 0) {{
                tb.style.display = 'flex';
                tb.style.top = (rect.top - containerRect.top - 40) + 'px';
                tb.style.left = (rect.left - containerRect.left + (rect.width / 2) - 80) + 'px';
            }} else {{
                tb.style.display = 'none';
            }}
        }});

        function formatText(cmd) {{
            document.execCommand(cmd, false, null);
        }}

        function promptLink() {{
            const url = prompt('Enter hyperlink URL:');
            if (url) {{
                document.execCommand('createLink', false, url);
            }}
        }}

        function insertBlock(kind) {{
            if (kind === 'Heading') {{
                window.emdashBlocks.push({{ block_type: 'Heading', level: 2, text: 'New Section Header' }});
            }} else if (kind === 'Paragraph') {{
                window.emdashBlocks.push({{ block_type: 'Paragraph', text: 'Enter paragraph copy here...' }});
            }} else if (kind === 'Quote') {{
                window.emdashBlocks.push({{ block_type: 'Quote', quote: 'Zero-allocation telemetry delivers unbreakable resilience.', citation: 'RustNext Architect' }});
            }} else if (kind === 'Callout') {{
                window.emdashBlocks.push({{ block_type: 'Callout', style: 'Info', title: 'Note', text: 'Ensure SurrealDB schema migrations have concluded.' }});
            }} else if (kind === 'Code') {{
                window.emdashBlocks.push({{ block_type: 'Code', language: 'rust', code: '// Pure-Rust concurrent actor\\npub async fn run() {{ ... }}' }});
            }} else if (kind === 'Table') {{
                window.emdashBlocks.push({{ block_type: 'Table', headers: ['Metric', 'Benchmark'], rows: [['Latency', '< 1ms'], ['Throughput', '25,000 req/s']] }});
            }} else if (kind === 'Image') {{
                window.emdashBlocks.push({{ block_type: 'Image', url: '/files/image/sample?w=800&h=450&fmt=webp', caption: 'Dynamic responsive media' }});
            }}
            renderBlocks();
        }}

        function removeBlock(idx) {{
            window.emdashBlocks.splice(idx, 1);
            renderBlocks();
        }}

        async function saveDraft() {{
            const title = document.getElementById('post-title').value;
            try {{
                const res = await fetch('/api/v2/emdash/documents/' + docId, {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{
                        title: title,
                        slug: docId,
                        author: 'Lead Editor',
                        blocks: window.emdashBlocks,
                        tags: ['enterprise', 'rust', 'cms'],
                        expected_version: window.documentVersion
                    }})
                }});
                const data = await res.json();
                if (res.status === 409) {{
                    showToast('⚠️ Conflict: Concurrent update detected! Reloading latest draft...', false);
                    setTimeout(initEmdash, 1200);
                    return;
                }}
                if (res.ok) {{
                    window.documentVersion = data.document ? data.document.version : (window.documentVersion + 1);
                    updateStats();
                    showToast(`✓ Draft saved to SurrealDB (v${{window.documentVersion}})`);
                }} else {{
                    showToast('Failed to save draft: ' + (data.error || 'error'), false);
                }}
            }} catch (e) {{
                showToast('Save error: ' + e.message, false);
            }}
        }}

        async function publishPost() {{
            await saveDraft();
            try {{
                const res = await fetch('/api/v2/emdash/documents/' + docId + '/transition', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{
                        target_state: 'Published',
                        timestamp: Math.floor(Date.now() / 1000)
                    }})
                }});
                const data = await res.json();
                if (res.ok) {{
                    document.getElementById('doc-status').textContent = 'Published';
                    document.getElementById('doc-status').style.color = '#10b981';
                    document.getElementById('view-live-link').style.display = 'inline-block';
                    showToast('✓ Post Published! Live at /site/' + docId);
                }} else {{
                    showToast('Publishing failed: ' + (data.error || 'error'), false);
                }}
            }} catch (e) {{
                showToast('Publish error: ' + e.message, false);
            }}
        }}

        async function initEmdash() {{
            try {{
                const res = await fetch('/api/v2/emdash/documents/' + docId);
                if (res.ok) {{
                    const data = await res.json();
                    if (data.document) {{
                        document.getElementById('post-title').value = data.document.title || 'Untitled';
                        if (data.document.version !== undefined) {{
                            window.documentVersion = data.document.version;
                        }}
                        if (data.document.blocks && data.document.blocks.length > 0) {{
                            window.emdashBlocks = data.document.blocks;
                            renderBlocks();
                            return;
                        }}
                    }}
                }}
            }} catch (_) {{}}

            // Defaults
            window.emdashBlocks = [
                {{ block_type: 'Paragraph', text: 'Modern enterprise resource planning demands microsecond query execution, zero garbage collection pauses, and absolute memory safety.' }},
                {{ block_type: 'Callout', style: 'Info', title: 'Architecture Note', text: 'Zero-allocation parsers on the warehouse floor coupled with multi-model database graphs in the cloud yield an unbreakable supply chain fabric.' }},
                {{ block_type: 'Code', language: 'rust', code: 'async fn execute_workflow() -> Result<(), SystemError> {{\n    println!("Real-time transaction executed.");\n    Ok(())\n}}' }}
            ];
            renderBlocks();
        }}

        document.addEventListener('DOMContentLoaded', initEmdash);
    </script>
</body>
</html>"###,
        target_doc = target_doc
    );

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

/// Live preview renderer for Builder pages: `GET /preview/{page_id}`
async fn builder_preview_handler(
    page_id: web::Path<String>,
    state: web::Data<DynamicRbacState>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let id = page_id.into_inner();
    let lock = match state.builder_pages.read() {
        Ok(l) => l,
        Err(_) => return HttpResponse::InternalServerError().body("Lock error"),
    };

    let body_html = if let Some(tree) = lock.get(&id) {
        let resolver = erp_cms::builder_core::BlockValueResolver::new();
        let rendered = tree.render_html(&resolver);
        format!(
            "<div class=\"rendered-builder-page\" style=\"min-height: 100vh; background: #ffffff;\">{}</div>",
            rendered
        )
    } else {
        format!(
            "<div style=\"padding: 3rem; text-align: center; font-family: sans-serif;\"><h1>Page '{}' Preview</h1><p>Default responsive canvas layout initialized.</p></div>",
            id
        )
    };

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(format!(
            "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"UTF-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\"><title>Preview — {id}</title><style>body {{ margin: 0; font-family: -apple-system, BlinkMacSystemFont, sans-serif; }}</style></head><body>{body_html}</body></html>"
        ))
}

/// Fallback Dynamic Route Dispatcher for published Builder pages and Emdash articles: `GET /site/{slug}`
#[allow(clippy::collapsible_if)]
async fn dynamic_site_page_handler(
    slug: web::Path<String>,
    state: web::Data<DynamicRbacState>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let target_slug = slug.into_inner();

    // 1. Check if matching Emdash published document exists
    if let Ok(lock) = state.emdash_documents.read() {
        for doc in lock.values() {
            if doc.slug.as_str() == target_slug.as_str() {
                let seo_tags = doc.seo.render_head_tags();
                let html = format!(
                    "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"UTF-8\">{seo_tags}</head><body style=\"font-family: Georgia, serif; max-width: 800px; margin: 3rem auto; padding: 0 1rem; line-height: 1.8;\">{}</body></html>",
                    doc.render_html()
                );
                return HttpResponse::Ok()
                    .content_type("text/html; charset=utf-8")
                    .body(html);
            }
        }
    }

    // 2. Check if matching Builder page exists
    if let Ok(lock) = state.builder_pages.read() {
        if let Some(tree) = lock.get(&target_slug) {
            let resolver = erp_cms::builder_core::BlockValueResolver::new();
            let rendered = tree.render_html(&resolver);
            let canonical = format!("https://rustnext.enterprise.io/site/{target_slug}");
            let seo = erp_cms::seo_engine::SeoMetadata::new_website(
                &format!("RustNext — {target_slug}"),
                "Published via RustNext pure-Rust Visual Builder.",
                &canonical,
            );
            let tags = seo.render_head_tags();
            let html = format!(
                "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"UTF-8\">{tags}<style>body {{ margin: 0; font-family: -apple-system, BlinkMacSystemFont, sans-serif; }}</style></head><body>{rendered}</body></html>"
            );
            return HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(html);
        }
    }

    // 3. Fallback to default rendered site page with Schema.org SEO tags
    let canonical = format!("https://rustnext.enterprise.io/site/{target_slug}");
    let seo = erp_cms::seo_engine::SeoMetadata::new_website(
        &format!("RustNext — {target_slug}"),
        "High-performance pure-Rust published web page.",
        &canonical,
    );
    let tags = seo.render_head_tags();

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(format!(
            "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"UTF-8\">{tags}</head><body style=\"font-family: -apple-system, sans-serif; max-width: 900px; margin: 4rem auto; padding: 0 1.5rem; text-align: center;\"><h1 style=\"font-size: 2.5rem;\">{target_slug}</h1><p style=\"color: #64748b; font-size: 1.1rem;\">Published via RustNext pure-Rust Visual Builder & Emdash CMS.</p></body></html>"
        ))
}

async fn list_desk_workspaces_handler() -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    HttpResponse::Ok().json(serde_json::json!({
        "workspaces": get_desk_workspaces()
    }))
}

async fn storefront_products_handler() -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    HttpResponse::Ok().json(erp_cms::get_luxury_catalog())
}

#[instrument(skip(req, pool_mgr, payload))]
async fn storefront_checkout_handler(
    req: HttpRequest,
    pool_mgr: web::Data<ConnectionPoolManager>,
    payload: web::Json<erp_cms::CustomerCheckoutRequest>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let tenant_id = req
        .extensions()
        .get::<TenantId>()
        .cloned()
        .unwrap_or_else(|| TenantId("default".into()));

    let checkout_req = payload.into_inner();
    match erp_cms::AtomicCheckoutEngine::process_checkout(&checkout_req) {
        Ok(result) => {
            if let Ok(client) = pool_mgr.get_or_initialize_client(&tenant_id).await
                && let Ok(invoice_val) = serde_json::to_value(&result)
            {
                let _ = client
                    .query(format!(
                        "CREATE sales_invoice:{} CONTENT $invoice;",
                        result.invoice_id.replace('-', "_")
                    ))
                    .bind(("invoice", invoice_val))
                    .await;
            }
            HttpResponse::Ok().json(result)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err.to_string(),
        })),
    }
}

#[instrument(skip(req, pool_mgr, body))]
async fn stripe_webhook_handler(
    req: HttpRequest,
    pool_mgr: web::Data<ConnectionPoolManager>,
    body: web::Bytes,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let sig_header = req
        .headers()
        .get("Stripe-Signature")
        .or_else(|| req.headers().get("X-RustNext-Signature-256"))
        .or_else(|| req.headers().get("X-Signature"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let stripe_secret = std::env::var("STRIPE_WEBHOOK_SECRET").unwrap_or_default();
    if !stripe_secret.is_empty()
        && !WebhookDispatcher::verify_signature(&stripe_secret, &body, sig_header)
    {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid webhook signature"
        }));
    }

    let tenant_id = req
        .extensions()
        .get::<TenantId>()
        .cloned()
        .unwrap_or_else(|| TenantId("default".into()));

    let event: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Invalid JSON payload: {e}")
            }));
        }
    };

    let event_type = event
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or_default();

    if (event_type == "payment_intent.succeeded" || event_type == "checkout.session.completed")
        && let Some(data) = event.get("data").and_then(|d| d.get("object"))
        && let Some(invoice_id) = data
            .get("invoice_id")
            .or_else(|| data.get("id"))
            .and_then(|i| i.as_str())
        && let Ok(client) = pool_mgr.get_or_initialize_client(&tenant_id).await
    {
        let sanitized_id = invoice_id.replace('-', "_");
        let now = chrono::Utc::now().to_rfc3339();
        let _ = client
            .query(format!(
                "UPDATE sales_invoice:{sanitized_id} SET status = 'Paid', paid_at = '{now}';"
            ))
            .await;
    }

    HttpResponse::Ok().json(serde_json::json!({
        "received": true,
        "event_type": event_type
    }))
}

async fn cms_get_page_handler(
    path: web::Path<String>,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let raw_slug = path.into_inner();
    let sanitized = match sanitize_slug(&raw_slug) {
        Some(s) => s,
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": "Invalid slug identifier"}));
        }
    };

    let client = match pool_mgr
        .get_or_initialize_client(&TenantId("default".into()))
        .await
    {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let sql = format!("SELECT * FROM cms_page_draft:{sanitized};");
    match client.query(&sql).await {
        Ok(mut res) => {
            let record: Option<serde_json::Value> = res.take(0).unwrap_or(None);
            match record {
                Some(r) => HttpResponse::Ok().json(r),
                None => HttpResponse::NotFound()
                    .json(serde_json::json!({"error": "Page draft not found"})),
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn cms_save_page_handler(
    path: web::Path<String>,
    payload: web::Json<serde_json::Value>,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let raw_slug = path.into_inner();
    let sanitized = match sanitize_slug(&raw_slug) {
        Some(s) => s,
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": "Invalid slug identifier"}));
        }
    };

    let client = match pool_mgr
        .get_or_initialize_client(&TenantId("default".into()))
        .await
    {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let doc = payload.into_inner();
    let sql = format!("CREATE cms_page_draft:{sanitized} CONTENT $doc;");
    match client.query(&sql).bind(("doc", doc.clone())).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Page draft saved",
            "slug": raw_slug
        })),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn cms_publish_page_handler(
    path: web::Path<String>,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let raw_slug = path.into_inner();
    let sanitized = match sanitize_slug(&raw_slug) {
        Some(s) => s,
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": "Invalid slug identifier"}));
        }
    };

    let client = match pool_mgr
        .get_or_initialize_client(&TenantId("default".into()))
        .await
    {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let fetch_sql = format!("SELECT * FROM cms_page_draft:{sanitized};");
    let mut res = match client.query(&fetch_sql).await {
        Ok(r) => r,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let record: Option<serde_json::Value> = res.take(0).unwrap_or(None);
    let Some(draft) = record else {
        return HttpResponse::NotFound()
            .json(serde_json::json!({"error": "Draft not found to publish"}));
    };

    let pub_sql = format!("CREATE cms_page_published:{sanitized} CONTENT $doc;");
    match client.query(&pub_sql).bind(("doc", draft)).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Page published successfully",
            "slug": raw_slug
        })),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn templates_portal_handler() -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(erp_cms::render_template_index_html())
}

#[derive(serde::Deserialize)]
struct TemplateQuery {
    variant: Option<String>,
}

async fn template_detail_handler(
    path: web::Path<String>,
    query: web::Query<TemplateQuery>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let slug = path.into_inner();
    let variant = query.variant.as_deref();
    match erp_cms::render_template_html_with_variant(&slug, variant) {
        Some(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Template '{slug}' not found"),
            "available_templates": erp_cms::list_template_suites().into_iter().map(|s| s.slug).collect::<Vec<_>>(),
        })),
    }
}

pub async fn sitemap_xml_handler(state: Option<web::Data<DynamicRbacState>>) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let mut entries = Vec::new();
    if let Some(st) = state {
        if let Ok(docs) = st.emdash_documents.read() {
            for (id, doc) in docs.iter() {
                if matches!(
                    doc.workflow_state,
                    erp_cms::EditorialWorkflowState::Published { .. }
                ) {
                    let loc = if !doc.slug.is_empty() {
                        format!("https://rustnext.enterprise.io/site/{}", doc.slug)
                    } else {
                        format!("https://rustnext.enterprise.io/site/{}", id)
                    };
                    entries.push(erp_cms::SitemapEntry {
                        loc,
                        lastmod: Some("2026-10-17".into()),
                        changefreq: Some("weekly".into()),
                        priority: Some(0.8),
                    });
                }
            }
        }
        if let Ok(pages) = st.builder_pages.read() {
            for id in pages.keys() {
                entries.push(erp_cms::SitemapEntry {
                    loc: format!("https://rustnext.enterprise.io/site/{}", id),
                    lastmod: Some("2026-10-17".into()),
                    changefreq: Some("daily".into()),
                    priority: Some(0.9),
                });
            }
        }
    }
    HttpResponse::Ok()
        .content_type("application/xml; charset=utf-8")
        .body(erp_cms::generate_dynamic_sitemap_xml(&entries))
}

#[allow(clippy::collapsible_if)]
pub async fn rss_feed_handler(state: Option<web::Data<DynamicRbacState>>) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let mut items = Vec::new();
    if let Some(st) = state {
        if let Ok(docs) = st.emdash_documents.read() {
            for (id, doc) in docs.iter() {
                if matches!(
                    doc.workflow_state,
                    erp_cms::EditorialWorkflowState::Published { .. }
                ) {
                    let link = if !doc.slug.is_empty() {
                        format!("https://rustnext.enterprise.io/site/{}", doc.slug)
                    } else {
                        format!("https://rustnext.enterprise.io/site/{}", id)
                    };
                    let summary = doc
                        .blocks
                        .iter()
                        .find_map(|b| match b {
                            erp_cms::EmdashBlock::Paragraph { text } => Some(text.to_string()),
                            _ => None,
                        })
                        .unwrap_or_else(|| "Planetary ERP Editorial Publication".to_string());

                    items.push(erp_cms::RssFeedItem {
                        title: doc.title.to_string(),
                        link,
                        description: summary,
                        pub_date: "Sat, 17 Oct 2026 12:00:00 GMT".into(),
                        author: Some(doc.author.to_string()),
                        guid: format!("emdash-{}", id),
                    });
                }
            }
        }
    }
    HttpResponse::Ok()
        .content_type("application/rss+xml; charset=utf-8")
        .body(erp_cms::generate_rss_2_xml(
            "rustnext Enterprise Journal",
            "https://rustnext.enterprise.io",
            "Continuous technical publications and operational blueprints.",
            &items,
        ))
}

/// HTML5 Video Streaming Player Studio & Telemetry HUD: `GET /studio/player`
pub async fn streaming_player_studio_handler(
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let video_id = query
        .get("v")
        .cloned()
        .unwrap_or_else(|| "vid_demo_master".into());
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RustNext OTT Video Player Studio</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;600&family=Outfit:wght@400;600;700&display=swap" rel="stylesheet">
    <script src="https://cdn.jsdelivr.net/npm/hls.js@latest"></script>
    <style>
        :root {{
            --bg: #090d16;
            --surface: #121927;
            --border: #1e293b;
            --accent: #06b6d4;
            --accent-glow: rgba(6, 182, 212, 0.35);
            --text-main: #f8fafc;
            --text-muted: #94a3b8;
            --success: #10b981;
        }}
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            background: var(--bg);
            color: var(--text-main);
            font-family: 'Outfit', sans-serif;
            min-height: 100vh;
            padding: 2rem;
            display: flex;
            flex-direction: column;
            gap: 1.5rem;
        }}
        .header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            border-bottom: 1px solid var(--border);
            padding-bottom: 1rem;
        }}
        .header h1 {{
            font-size: 1.5rem;
            display: flex;
            align-items: center;
            gap: 0.5rem;
            color: var(--accent);
        }}
        .studio-grid {{
            display: grid;
            grid-template-columns: 2fr 1fr;
            gap: 1.5rem;
        }}
        .player-card, .hud-card {{
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 12px;
            padding: 1.25rem;
            display: flex;
            flex-direction: column;
            gap: 1rem;
        }}
        .video-wrapper {{
            position: relative;
            background: #000;
            border-radius: 8px;
            overflow: hidden;
            aspect-ratio: 16/9;
        }}
        video {{
            width: 100%;
            height: 100%;
            object-fit: contain;
        }}
        .controls-bar {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            font-size: 0.85rem;
        }}
        .badge {{
            padding: 0.25rem 0.5rem;
            border-radius: 4px;
            background: rgba(16, 185, 129, 0.15);
            color: var(--success);
            font-weight: 600;
        }}
        .mono {{
            font-family: 'JetBrains Mono', monospace;
            font-size: 0.8rem;
            background: #050811;
            padding: 0.75rem;
            border-radius: 6px;
            color: #38bdf8;
            overflow-x: auto;
        }}
        select, button {{
            background: #1e293b;
            color: #f8fafc;
            border: 1px solid #334155;
            padding: 0.4rem 0.8rem;
            border-radius: 6px;
            font-family: inherit;
            cursor: pointer;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1><span>⚡</span> RustNext OTT Player Studio</h1>
        <div class="badge">RFC 8216 Compliant</div>
    </div>
    <div class="studio-grid">
        <div class="player-card">
            <div class="video-wrapper">
                <video id="video-elem" controls autoplay playsinline></video>
            </div>
            <div class="controls-bar">
                <div>Source: <span class="mono">/api/v2/streaming/videos/{video_id}/master.m3u8</span></div>
                <div>
                    Ladder:
                    <select id="quality-select">
                        <option value="auto">Auto ABR</option>
                        <option value="1080p">1080p (4.5M)</option>
                        <option value="720p">720p (2.4M)</option>
                        <option value="480p">480p (1.2M)</option>
                        <option value="360p">360p (600k)</option>
                    </select>
                </div>
            </div>
        </div>
        <div class="hud-card">
            <h3>Edge Telemetry HUD (no_std wire format)</h3>
            <div class="mono" id="telemetry-box">
                [VODP Packet]<br>
                Playhead: 00:00:00<br>
                Buffer Health: 0.0s<br>
                Bandwidth: Evaluating...<br>
                Status: Standby
            </div>
            <button id="ping-progress-btn">Send Progress Heartbeat</button>
        </div>
    </div>
    <script>
        const video = document.getElementById('video-elem');
        const src = '/api/v2/streaming/videos/{video_id}/master.m3u8';
        if (Hls.isSupported()) {{
            const hls = new Hls();
            hls.loadSource(src);
            hls.attachMedia(video);
            hls.on(Hls.Events.MANIFEST_PARSED, function() {{
                console.log('HLS manifest loaded with ' + hls.levels.length + ' quality levels');
            }});
        }} else if (video.canPlayType('application/vnd.apple.mpegurl')) {{
            video.src = src;
        }}

        document.getElementById('ping-progress-btn').addEventListener('click', async () => {{
            const res = await fetch('/api/v2/streaming/playback/progress', {{
                method: 'POST',
                headers: {{ 'Content-Type': 'application/json' }},
                body: JSON.stringify({{
                    user_id: 'usr_viewer_1',
                    media_id: '{video_id}',
                    session_id: 'sess_' + Math.random().toString(36).substring(7),
                    playhead_sec: Math.floor(video.currentTime || 0),
                    duration_sec: Math.floor(video.duration || 7200),
                    buffered_sec: 25,
                    is_paused: video.paused
                }})
            }});
            const data = await res.json();
            document.getElementById('telemetry-box').innerHTML =
                '[VODP Packet Telemetry Verified]<br>' +
                'Status: ' + data.status + '<br>' +
                'Playhead: ' + data.playhead_sec + 's<br>' +
                'Duration: ' + data.duration_sec + 's<br>' +
                'Wire Bytes: ' + data.wire_bytes + ' (no_std fixed)';
        }});
    </script>
</body>
</html>"#,
        video_id = video_id
    );

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

async fn robots_txt_handler() -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(erp_cms::generate_robots_txt())
}

async fn list_templates_api_handler() -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    HttpResponse::Ok().json(erp_cms::list_template_suites())
}

async fn template_manifest_api_handler(path: web::Path<String>) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let slug = path.into_inner();
    match erp_cms::get_template_suite(&slug) {
        Some(suite) => HttpResponse::Ok().json(suite.to_theme_manifest()),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Template '{slug}' not found"),
        })),
    }
}

async fn acme_challenge_handler(
    path: web::Path<String>,
    gateway: web::Data<AcmeGateway>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let token = path.into_inner();
    match gateway.resolve_domain(&token).await {
        Some(_) => HttpResponse::Ok().body(format!("{token}.simulated_acme_key_authorization")),
        None => HttpResponse::NotFound().body("ACME Challenge Not Found"),
    }
}

// ---------------------------------------------------------------------------
// Edge Analytics Handlers
// ---------------------------------------------------------------------------

async fn record_analytics_event_handler(
    analytics: web::Data<erp_cms::EdgeAnalyticsEngine>,
    payload: web::Json<erp_cms::PageViewEvent>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    analytics.record_page_view(payload.into_inner());
    HttpResponse::Ok().json(serde_json::json!({
        "status": "recorded"
    }))
}

async fn get_analytics_summary_handler(
    analytics: web::Data<erp_cms::EdgeAnalyticsEngine>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    HttpResponse::Ok().json(analytics.compute_summary())
}

// ---------------------------------------------------------------------------
// Webhook Subscription Management Handlers (RBAC Hardened)
// ---------------------------------------------------------------------------

#[instrument(skip(req, pool_mgr, payload))]
async fn create_webhook_subscription_handler(
    req: HttpRequest,
    pool_mgr: web::Data<ConnectionPoolManager>,
    payload: web::Json<frappe_framework::WebhookSubscription>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    if let Some(ctx) = req.extensions().get::<SecurityContext>()
        && !check_permission(&ctx.claims.roles, &[], Permission::Write, 0)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Permission Denied: insufficient write privileges"
        }));
    }

    let tenant_id = req
        .extensions()
        .get::<TenantId>()
        .cloned()
        .unwrap_or_else(|| TenantId("default".into()));

    let client = match pool_mgr.get_or_initialize_client(&tenant_id).await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let sub = payload.into_inner();
    let sanitized_id = sub.id.replace('-', "_");
    let sql = format!("CREATE webhook_subscription:{sanitized_id} CONTENT $sub;");
    let sub_val = match serde_json::to_value(&sub) {
        Ok(v) => v,
        Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
    };
    match client.query(&sql).bind(("sub", sub_val)).await {
        Ok(_) => HttpResponse::Created().json(sub),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[instrument(skip(req, pool_mgr))]
async fn list_webhook_subscriptions_handler(
    req: HttpRequest,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    if let Some(ctx) = req.extensions().get::<SecurityContext>()
        && !check_permission(&ctx.claims.roles, &[], Permission::Read, 0)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Permission Denied: insufficient read privileges"
        }));
    }

    let tenant_id = req
        .extensions()
        .get::<TenantId>()
        .cloned()
        .unwrap_or_else(|| TenantId("default".into()));

    let client = match pool_mgr.get_or_initialize_client(&tenant_id).await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    match client.query("SELECT * FROM webhook_subscription;").await {
        Ok(mut res) => {
            let subs: Vec<serde_json::Value> = res.take(0).unwrap_or_default();
            HttpResponse::Ok().json(subs)
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[instrument(skip(req, pool_mgr))]
async fn delete_webhook_subscription_handler(
    path: web::Path<String>,
    req: HttpRequest,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    if let Some(ctx) = req.extensions().get::<SecurityContext>()
        && !check_permission(&ctx.claims.roles, &[], Permission::Write, 0)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Permission Denied: insufficient write privileges"
        }));
    }

    let id = path.into_inner();
    let tenant_id = req
        .extensions()
        .get::<TenantId>()
        .cloned()
        .unwrap_or_else(|| TenantId("default".into()));

    let client = match pool_mgr.get_or_initialize_client(&tenant_id).await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let sanitized_id = id.replace('-', "_");
    match client
        .query(format!("DELETE webhook_subscription:{sanitized_id};"))
        .await
    {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Subscription deleted",
            "id": id
        })),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// WASI Plugin Execution Handler (RBAC Hardened)
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct RunPluginPayload {
    wat_code: String,
    fuel_limit: Option<u64>,
}

async fn run_plugin_handler(
    req: HttpRequest,
    payload: web::Json<RunPluginPayload>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    if let Some(ctx) = req.extensions().get::<SecurityContext>()
        && !check_permission(&ctx.claims.roles, &[], Permission::Execute, 0)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Permission Denied: insufficient execute privileges for plugins"
        }));
    }

    let inner = payload.into_inner();
    let fuel = inner.fuel_limit.unwrap_or(1_000_000);
    let sandbox = match frappe_framework::RealSandbox::new(fuel, 32 * 1024 * 1024) {
        Ok(s) => s,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to create sandbox: {e}")
            }));
        }
    };

    match sandbox.run_wat(&inner.wat_code) {
        Ok(result) => HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "exit_code": result
        })),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "error": e.to_string()
        })),
    }
}

async fn pwa_manifest_handler() -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let manifest = desk_app::PwaWebManifest::new("RustNext Enterprise", "#6366f1", "#0f172a");
    HttpResponse::Ok()
        .content_type("application/manifest+json")
        .json(manifest)
}

async fn service_worker_handler() -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let sw_script =
        desk_app::ServiceWorkerGenerator::generate_service_worker_js(env!("CARGO_PKG_VERSION"));
    HttpResponse::Ok()
        .content_type("application/javascript")
        .insert_header(("Cache-Control", "public, max-age=0, must-revalidate"))
        .body(sw_script)
}

/// Configures the Actix Web ERP API application.
pub fn configure_app(
    cfg: &mut web::ServiceConfig,
    pool_mgr: ConnectionPoolManager,
    topology: MicroTopologyConfig,
    acme_gateway: Option<AcmeGateway>,
) {
    cfg.app_data(web::Data::new(pool_mgr))
        .app_data(web::Data::new(
            ControllerRegistry::with_standard_submittables(),
        ))
        .app_data(web::Data::new(DynamicRbacState::default()))
        .app_data(web::Data::new(erp_cms::EdgeAnalyticsEngine::new()))
        .app_data(web::PayloadConfig::new(topology.max_payload_bytes))
        // Root storefront & Multi-Persona Shells
        .route("/", web::get().to(storefront_handler))
        .route("/desk", web::get().to(desk_handler))
        .route(
            "/desk/list/{doctype}",
            web::get().to(desk_list_view_handler),
        )
        .route(
            "/desk/form/{doctype}/{id}",
            web::get().to(desk_form_view_handler),
        )
        .route("/portal", web::get().to(portal_handler))
        .route("/worker", web::get().to(worker_handler))
        .route("/factory", web::get().to(factory_handler))
        .route("/approvals", web::get().to(approvals_handler))
        .route("/admin", web::get().to(admin_handler))
        .route("/storefront", web::get().to(storefront_handler))
        .route("/builder", web::get().to(builder_studio_handler))
        .route("/builder/{page_id}", web::get().to(builder_studio_handler))
        .route("/emdash", web::get().to(emdash_studio_handler))
        .route("/emdash/editor/{id}", web::get().to(emdash_studio_handler))
        .route("/preview/{page_id}", web::get().to(builder_preview_handler))
        .route("/site/{slug}", web::get().to(dynamic_site_page_handler))
        .route("/templates", web::get().to(templates_portal_handler))
        .route("/templates/{slug}", web::get().to(template_detail_handler))
        .route("/sitemap.xml", web::get().to(sitemap_xml_handler))
        .route("/feed.xml", web::get().to(rss_feed_handler))
        .route("/rss.xml", web::get().to(rss_feed_handler))
        .route("/robots.txt", web::get().to(robots_txt_handler))
        .route("/manifest.json", web::get().to(pwa_manifest_handler))
        .route("/manifest.webmanifest", web::get().to(pwa_manifest_handler))
        .route("/sw.js", web::get().to(service_worker_handler))
        .route("/health", web::get().to(health_check))
        .route("/healthz/live", web::get().to(liveness_handler))
        .route("/healthz/ready", web::get().to(readiness_handler))
        .route("/metrics", web::get().to(metrics_handler))
        .route("/files/{hash}", web::get().to(download_file_handler))
        .route("/files/image/{hash}", web::get().to(download_image_handler))
        .route("/api/v1/live", web::get().to(live_ws_handler))
        .route(
            "/api/v1/desk/workspaces",
            web::get().to(list_desk_workspaces_handler),
        )
        .route(
            "/api/v1/storefront/products",
            web::get().to(storefront_products_handler),
        )
        .route(
            "/api/v1/storefront/checkout",
            web::post().to(storefront_checkout_handler),
        )
        .route(
            "/api/v1/storefront/webhooks/stripe",
            web::post().to(stripe_webhook_handler),
        )
        .route(
            "/api/v1/templates",
            web::get().to(list_templates_api_handler),
        )
        .route(
            "/api/v1/templates/{slug}/manifest",
            web::get().to(template_manifest_api_handler),
        )
        // Edge Analytics API
        .route(
            "/api/v1/analytics/event",
            web::post().to(record_analytics_event_handler),
        )
        .route(
            "/api/v1/analytics/summary",
            web::get().to(get_analytics_summary_handler),
        )
        // Universal AI Agent OAuth 2.0 Token Server
        .route(
            "/api/v1/oauth/token",
            web::post().to(crate::ai_oauth::oauth_token_handler),
        )
        .route(
            "/oauth/token",
            web::post().to(crate::ai_oauth::oauth_token_handler),
        )
        // V2 Authentication, RPC System Methods, Webhooks & Storage
        .route("/api/v2/quic/status", web::get().to(quic_status_handler))
        .route(
            "/api/v2/stream/h3/file/{hash}",
            web::get().to(h3_stream_file_handler),
        )
        .route(
            "/api/v2/stream/h3/telemetry",
            web::post().to(h3_stream_telemetry_handler),
        )
        .route("/api/v2/admin/status", web::get().to(admin_status_handler))
        .route(
            "/api/v2/admin/action/{action_id}",
            web::post().to(admin_action_handler),
        )
        .route("/api/v2/admin/query", web::post().to(admin_query_handler))
        // Dynamic User & Role Permission Management
        .route(
            "/api/v2/admin/users",
            web::get().to(admin_list_users_handler),
        )
        .route(
            "/api/v2/admin/users",
            web::post().to(admin_create_user_handler),
        )
        .route(
            "/api/v2/admin/users/{user_id}/roles",
            web::put().to(admin_update_user_roles_handler),
        )
        .route(
            "/api/v2/admin/users/{user_id}",
            web::delete().to(admin_delete_user_handler),
        )
        .route(
            "/api/v2/admin/users/{user_id}/effective-permissions",
            web::get().to(admin_get_user_effective_permissions_handler),
        )
        .route(
            "/api/v2/admin/roles",
            web::get().to(admin_list_roles_handler),
        )
        .route(
            "/api/v2/admin/roles",
            web::post().to(admin_create_role_handler),
        )
        .route(
            "/api/v2/admin/permissions",
            web::get().to(admin_get_permissions_handler),
        )
        .route(
            "/api/v2/admin/permissions",
            web::post().to(admin_update_permission_handler),
        )
        .route("/api/v2/method/login", web::post().to(login_handler))
        .route("/api/v2/method/logout", web::post().to(logout_handler))
        .route("/api/v2/method/ping", web::get().to(ping_handler))
        .route(
            "/api/v2/auth/refresh",
            web::post().to(auth_refresh_token_handler),
        )
        .route(
            "/api/v2/method/render_invoice",
            web::post().to(render_invoice_handler),
        )
        .route(
            "/api/v2/method/render_receipt",
            web::post().to(render_receipt_handler),
        )
        // Enterprise Domain Modules: Accounting, Inventory, CRM, HR
        .route(
            "/api/v2/accounting/journal_entry",
            web::post().to(accounting_journal_entry_handler),
        )
        .route(
            "/api/v2/accounting/trial_balance",
            web::get().to(accounting_trial_balance_handler),
        )
        .route(
            "/api/v2/inventory/stock_entry",
            web::post().to(inventory_stock_entry_handler),
        )
        .route(
            "/api/v2/inventory/balance/{warehouse}/{item_code}",
            web::get().to(inventory_stock_balance_handler),
        )
        .route(
            "/api/v2/crm/quotations/convert",
            web::post().to(crm_convert_quotation_handler),
        )
        .route(
            "/api/v2/hr/payroll/process",
            web::post().to(hr_process_payroll_handler),
        )
        // Wave 1 MFA & Password Security Endpoints
        .route(
            "/api/v2/auth/mfa/enroll",
            web::post().to(auth_mfa_enroll_handler),
        )
        .route(
            "/api/v2/auth/mfa/activate",
            web::post().to(auth_mfa_activate_handler),
        )
        .route(
            "/api/v2/auth/mfa/disable",
            web::post().to(auth_mfa_disable_handler),
        )
        .route(
            "/api/v2/auth/forgot-password",
            web::post().to(auth_forgot_password_handler),
        )
        .route(
            "/api/v2/auth/reset-password",
            web::post().to(auth_reset_password_handler),
        )
        // Wave 1 Enterprise Audit Logging & IP Security Governance
        .route(
            "/api/v2/admin/audit-logs",
            web::get().to(admin_get_audit_logs_handler),
        )
        .route(
            "/api/v2/admin/security/ip-rules",
            web::get().to(admin_list_ip_rules_handler),
        )
        .route(
            "/api/v2/admin/security/ip-rules",
            web::post().to(admin_create_ip_rule_handler),
        )
        .route(
            "/api/v2/admin/security/ip-rules/{rule_id}",
            web::delete().to(admin_delete_ip_rule_handler),
        )
        .route(
            "/api/v2/admin/security/lockouts",
            web::get().to(admin_list_lockouts_handler),
        )
        .route(
            "/api/v2/admin/security/unlock",
            web::post().to(admin_unlock_target_handler),
        )
        .route(
            "/api/v2/method/upload_file",
            web::post().to(upload_file_handler),
        )
        .route(
            "/api/v2/method/run_plugin",
            web::post().to(run_plugin_handler),
        )
        .route(
            "/api/v2/webhooks/subscribe",
            web::post().to(create_webhook_subscription_handler),
        )
        .route(
            "/api/v2/webhooks/subscriptions",
            web::get().to(list_webhook_subscriptions_handler),
        )
        .route(
            "/api/v2/webhooks/subscribe/{id}",
            web::delete().to(delete_webhook_subscription_handler),
        )
        // V2 Visual CMS Persistence Endpoints
        .route(
            "/api/v2/cms/page/{slug}",
            web::get().to(cms_get_page_handler),
        )
        .route(
            "/api/v2/cms/page/{slug}/save",
            web::post().to(cms_save_page_handler),
        )
        .route(
            "/api/v2/cms/page/{slug}/publish",
            web::post().to(cms_publish_page_handler),
        )
        // V2 Commerce & Promotion Endpoints
        .route(
            "/api/v2/trade/coupons",
            web::get().to(trade_list_coupons_handler),
        )
        .route(
            "/api/v2/trade/coupons",
            web::post().to(trade_create_coupon_handler),
        )
        .route(
            "/api/v2/trade/coupons/apply",
            web::post().to(trade_apply_coupon_handler),
        )
        .route(
            "/api/v2/trade/reviews",
            web::get().to(trade_list_reviews_handler),
        )
        .route(
            "/api/v2/trade/reviews",
            web::post().to(trade_submit_review_handler),
        )
        .route(
            "/api/v2/trade/reviews/moderate",
            web::post().to(trade_moderate_review_handler),
        )
        .route(
            "/api/v2/trade/wishlists/{user_id}",
            web::get().to(trade_get_wishlist_handler),
        )
        .route(
            "/api/v2/trade/wishlists/{user_id}/items",
            web::post().to(trade_add_wishlist_item_handler),
        )
        .route(
            "/api/v2/trade/wishlists/{user_id}/items/{item_code}",
            web::delete().to(trade_remove_wishlist_item_handler),
        )
        .route(
            "/api/v2/trade/shipping/zones",
            web::get().to(trade_list_shipping_zones_handler),
        )
        .route(
            "/api/v2/trade/shipping/calculate",
            web::post().to(trade_calculate_shipping_handler),
        )
        .route(
            "/api/v2/trade/orders/transition",
            web::post().to(trade_order_transition_handler),
        )
        .route(
            "/api/v2/trade/rma",
            web::post().to(trade_rma_submit_handler),
        )
        .route(
            "/api/v2/trade/rma/{rma_id}",
            web::get().to(trade_rma_get_handler),
        )
        // V2 CMS Taxonomy, Media & SEO Endpoints
        .route(
            "/api/v2/cms/taxonomy",
            web::get().to(cms_list_taxonomies_handler),
        )
        .route(
            "/api/v2/cms/taxonomy",
            web::post().to(cms_create_taxonomy_handler),
        )
        .route("/api/v2/cms/media", web::get().to(cms_list_media_handler))
        .route(
            "/api/v2/cms/media",
            web::post().to(cms_upload_media_handler),
        )
        .route(
            "/api/v2/cms/seo/generate",
            web::post().to(cms_generate_seo_handler),
        )
        // V2 Frappe Builder Visual Canvas APIs
        .route(
            "/api/v2/builder/pages",
            web::get().to(builder_list_pages_handler),
        )
        .route(
            "/api/v2/builder/pages/{id}",
            web::get().to(builder_get_page_handler),
        )
        .route(
            "/api/v2/builder/pages/{id}",
            web::post().to(builder_save_page_handler),
        )
        .route(
            "/api/v2/builder/pages/{id}/history",
            web::get().to(builder_history_list_handler),
        )
        .route(
            "/api/v2/builder/ai/prompt",
            web::post().to(builder_ai_prompt_handler),
        )
        // V2 Emdash Editorial Studio APIs
        .route(
            "/api/v2/emdash/documents",
            web::get().to(emdash_list_documents_handler),
        )
        .route(
            "/api/v2/emdash/documents/{id}",
            web::get().to(emdash_get_document_handler),
        )
        .route(
            "/api/v2/emdash/documents/{id}",
            web::post().to(emdash_save_document_handler),
        )
        .route(
            "/api/v2/emdash/documents/{id}/revisions",
            web::get().to(emdash_revisions_list_handler),
        )
        .route(
            "/api/v2/emdash/documents/{id}/transition",
            web::post().to(emdash_transition_handler),
        )
        .route(
            "/api/v2/emdash/media/crop",
            web::post().to(emdash_media_crop_handler),
        )
        // V2 OTT Video Streaming, RTMP Ingest & ABR Playback Endpoints (Phase 6)
        .route(
            "/api/v2/streaming/videos/{id}/master.m3u8",
            web::get().to(streaming_master_playlist_handler),
        )
        .route(
            "/api/v2/streaming/videos/{id}/{variant}/index.m3u8",
            web::get().to(streaming_variant_playlist_handler),
        )
        .route(
            "/api/v2/streaming/videos/{id}/{variant}/{segment}",
            web::get().to(streaming_segment_handler),
        )
        .route(
            "/api/v2/streaming/rtmp/auth",
            web::post().to(streaming_rtmp_auth_handler),
        )
        .route(
            "/api/v2/streaming/rtmp/done",
            web::post().to(streaming_rtmp_done_handler),
        )
        .route(
            "/api/v2/streaming/upload/chunk",
            web::post().to(streaming_upload_chunk_handler),
        )
        .route(
            "/api/v2/streaming/playback/progress",
            web::post().to(streaming_playback_progress_handler),
        )
        .route(
            "/api/v2/streaming/videos/{id}/xray",
            web::get().to(streaming_video_xray_handler),
        )
        .route(
            "/api/v2/streaming/videos/{id}/xray.vtt",
            web::get().to(streaming_video_xray_vtt_handler),
        )
        .route(
            "/api/v2/streaming/fast/{channel_id}/live.m3u8",
            web::get().to(streaming_fast_live_manifest_handler),
        )
        .route(
            "/api/v2/streaming/fast/{channel_id}/status",
            web::get().to(streaming_fast_status_handler),
        )
        .route(
            "/api/v2/streaming/fast/{channel_id}/epg.json",
            web::get().to(streaming_fast_epg_handler),
        )
        // QUIC Resilient Streaming & User Cache Endpoints
        .route(
            "/api/v2/streaming/videos/{id}/resilient.m3u8",
            web::get().to(streaming_resilient_master_playlist_handler),
        )
        .route(
            "/api/v2/streaming/cache/configure",
            web::post().to(streaming_cache_configure_handler),
        )
        .route(
            "/api/v2/streaming/cache/stats",
            web::get().to(streaming_cache_stats_handler),
        )
        .route(
            "/api/v2/streaming/cache/prefetch",
            web::post().to(streaming_cache_prefetch_handler),
        )
        .route(
            "/api/v2/streaming/h3/segment/{id}/{variant}/{segment}",
            web::get().to(streaming_h3_segment_handler),
        )
        // Stremio Addon Protocol v3 Endpoints (Phase 10)
        .route(
            "/api/v2/streaming/addon/manifest.json",
            web::get().to(stremio_manifest_handler),
        )
        .route(
            "/api/v2/streaming/addon/catalog/{type}/{id}.json",
            web::get().to(stremio_catalog_handler),
        )
        .route(
            "/api/v2/streaming/addon/meta/{type}/{id}.json",
            web::get().to(stremio_meta_handler),
        )
        .route(
            "/api/v2/streaming/addon/stream/{type}/{id}.json",
            web::get().to(stremio_stream_handler),
        )
        .route(
            "/api/v2/streaming/addon/subtitles/{type}/{id}.json",
            web::get().to(stremio_subtitles_handler),
        )
        .route(
            "/studio/player",
            web::get().to(streaming_player_studio_handler),
        )
        // High-Level Audio Streaming & Subsonic Protocol v1.16 Endpoints
        .route(
            "/api/v2/audio/normalize",
            web::post().to(audio_normalize_handler),
        )
        .route(
            "/api/v2/audio/crossfade",
            web::post().to(audio_crossfade_handler),
        )
        .route(
            "/api/v2/audio/waveform",
            web::post().to(audio_waveform_handler),
        )
        .route(
            "/api/v2/audio/comments",
            web::post().to(audio_add_comment_handler),
        )
        .route(
            "/api/v2/audio/comments/{track_id}",
            web::get().to(audio_get_comments_handler),
        )
        .route(
            "/api/v2/audio/sleep/soundboard",
            web::post().to(audio_sleep_soundboard_handler),
        )
        .route(
            "/api/v2/audio/sleep/timer",
            web::post().to(audio_sleep_timer_handler),
        )
        .route("/rest/ping.view", web::get().to(subsonic_ping_handler))
        .route(
            "/rest/getLicense.view",
            web::get().to(subsonic_get_license_handler),
        )
        .route(
            "/rest/getSong.view",
            web::get().to(subsonic_get_song_handler),
        )
        .route("/rest/stream.view", web::get().to(subsonic_stream_handler))
        .route(
            "/api/v2/audio/flac/compress",
            web::post().to(audio_flac_compress_handler),
        )
        .route(
            "/api/v2/audio/eq/filter",
            web::post().to(audio_eq_filter_handler),
        )
        .route(
            "/api/v2/audio/lyrics/parse",
            web::post().to(audio_lyrics_parse_handler),
        )
        .route(
            "/api/v2/audio/lyrics/current",
            web::post().to(audio_lyrics_current_handler),
        )
        .route(
            "/api/v2/audio/music/info",
            web::post().to(audio_music_info_handler),
        )
        .route(
            "/api/v2/audio/recommend",
            web::post().to(audio_recommend_handler),
        )
        // V2 Approval Workflow & Versioning Endpoints
        .route(
            "/api/v2/workflow/evaluate",
            web::post().to(workflow_evaluate_handler),
        )
        .route(
            "/api/v2/workflow/versions/{doctype}/{docname}",
            web::get().to(workflow_version_history_handler),
        )
        .route(
            "/api/v2/workflow/versions/rollback",
            web::post().to(workflow_version_rollback_handler),
        )
        // V2 Universal Export & Pivot Reporting Endpoints
        .route(
            "/api/v2/export/data",
            web::post().to(export_dataset_handler),
        )
        .route(
            "/api/v2/reports/pivot",
            web::post().to(report_pivot_table_handler),
        )
        // V2 Omni-Channel Notifications & Webhooks Endpoints
        .route(
            "/api/v2/notifications/inbox/{user_id}",
            web::get().to(notifications_get_inbox_handler),
        )
        .route(
            "/api/v2/notifications/dispatch",
            web::post().to(notifications_dispatch_handler),
        )
        .route(
            "/api/v2/webhooks/test",
            web::post().to(webhooks_dispatch_test_handler),
        )
        // V2 Buying (Procure-to-Pay) Endpoints
        .route(
            "/api/v2/buying/purchase_order",
            web::post().to(buying_purchase_order_handler),
        )
        .route(
            "/api/v2/buying/purchase_receipt",
            web::post().to(buying_purchase_receipt_handler),
        )
        .route(
            "/api/v2/buying/purchase_invoice",
            web::post().to(buying_purchase_invoice_handler),
        )
        // V2 Selling (Order-to-Cash) Endpoints
        .route(
            "/api/v2/selling/delivery_note",
            web::post().to(selling_delivery_note_handler),
        )
        .route(
            "/api/v2/selling/sales_invoice",
            web::post().to(selling_sales_invoice_handler),
        )
        // V2 Accounting Settlement & Financial Reporting Endpoints
        .route(
            "/api/v2/accounting/payment_entry",
            web::post().to(accounting_payment_entry_handler),
        )
        .route(
            "/api/v2/accounting/profit_loss",
            web::get().to(accounting_profit_loss_handler),
        )
        .route(
            "/api/v2/accounting/balance_sheet",
            web::get().to(accounting_balance_sheet_handler),
        )
        .route(
            "/api/v2/accounting/period_close",
            web::post().to(accounting_period_close_handler),
        )
        .route(
            "/api/v2/accounting/ar_aging",
            web::get().to(accounting_ar_aging_handler),
        )
        .route(
            "/api/v2/accounting/ap_aging",
            web::get().to(accounting_ap_aging_handler),
        )
        .route(
            "/api/v2/accounting/general_ledger",
            web::get().to(accounting_general_ledger_handler),
        )
        .route(
            "/api/v2/accounting/cash_flow",
            web::get().to(accounting_cash_flow_handler),
        )
        // V2 Customer & Supplier Self-Service Portal Endpoints
        .route(
            "/api/v2/portal/customer_dashboard",
            web::get().to(portal_customer_dashboard_handler),
        )
        .route(
            "/api/v2/portal/supplier_dashboard",
            web::get().to(portal_supplier_dashboard_handler),
        )
        // V2 First-Time Setup Wizard & Seeding Endpoint
        .route("/api/v2/setup/wizard", web::post().to(setup_wizard_handler))
        // V2 Item Variant Matrix Generation Endpoint
        .route(
            "/api/v2/inventory/variant_matrix",
            web::post().to(inventory_variant_matrix_handler),
        )
        // V2 Manufacturing Work Order Endpoints
        .route(
            "/api/v2/manufacturing/work_order",
            web::post().to(manufacturing_work_order_handler),
        )
        .route(
            "/api/v2/manufacturing/work_order/complete",
            web::post().to(manufacturing_complete_work_order_handler),
        )
        // V2 Universal Print Format & Email Engine Endpoints
        .route(
            "/api/v2/print/document",
            web::post().to(print_document_handler),
        )
        .route("/api/v2/email/send", web::post().to(email_send_handler))
        .route(
            "/api/v2/email/queue",
            web::get().to(email_list_queue_handler),
        )
        // V2 Payment Gateway Processing & Settlement Webhooks
        .route(
            "/api/v2/payments/charge",
            web::post().to(payment_charge_handler),
        )
        .route(
            "/api/v2/payments/webhook",
            web::post().to(payment_webhook_handler),
        )
        // Protected V1 REST Resource API Scope
        .service(
            web::scope("/api/v1/resource")
                .wrap(TenantResolver)
                .route("/{doctype}", web::get().to(list_resource))
                .route("/{doctype}", web::post().to(create_resource))
                .route("/{doctype}/{id}", web::get().to(get_resource))
                .route("/{doctype}/{id}", web::delete().to(delete_resource)),
        )
        // Protected V2 Hypermedia Document API Scope
        .service(
            web::scope("/api/v2/document")
                .wrap(TenantResolver)
                .route("/{doctype}", web::get().to(v2_list_document))
                .route("/{doctype}", web::post().to(v2_create_document))
                .route("/{doctype}/{name}", web::get().to(v2_get_document))
                .route("/{doctype}/{name}/print", web::get().to(v2_print_document))
                .route("/{doctype}/{name}", web::put().to(v2_update_document))
                .route("/{doctype}/{name}", web::delete().to(v2_delete_document))
                .route(
                    "/{doctype}/{name}/submit",
                    web::post().to(v2_submit_document),
                )
                .route(
                    "/{doctype}/{name}/cancel",
                    web::post().to(v2_cancel_document),
                )
                .route("/{doctype}/{name}/amend", web::post().to(v2_amend_document)),
        )
        // DocType Metadata API Scope
        .service(
            web::scope("/api/v2/meta")
                .wrap(TenantResolver)
                .route("", web::get().to(v2_list_meta_handler))
                .route("/", web::get().to(v2_list_meta_handler))
                .route("/{doctype}", web::get().to(v2_get_meta_handler)),
        );

    if let Some(gateway) = acme_gateway {
        cfg.app_data(web::Data::new(gateway)).route(
            "/.well-known/acme-challenge/{token}",
            web::get().to(acme_challenge_handler),
        );
    }
}

/// Runs the Actix Web Server on the specified address with custom topology and ACME gateway options.
pub async fn run_server_with_config(
    addr: &str,
    topology: MicroTopologyConfig,
    acme_gateway: Option<AcmeGateway>,
    workers: Option<usize>,
) -> std::io::Result<()> {
    let pool_mgr = ConnectionPoolManager::new(Duration::from_secs(300));
    let pool_mgr_data = pool_mgr.clone();
    let topology_clone = topology.clone();
    let gateway_clone = acme_gateway.clone();

    info!(
        "Starting RustNext server on {addr} with workers: {:?}",
        workers
    );

    let mut server = HttpServer::new(move || {
        let topo = topology_clone.clone();
        let gw = gateway_clone.clone();
        App::new()
            .wrap(crate::middleware::RequestIdMiddleware::new())
            .wrap(crate::middleware::CorsMiddleware::new())
            .wrap(Logger::default())
            .wrap(Compress::default())
            .wrap(NormalizePath::trim())
            .wrap(crate::middleware::SecurityHeaders::new())
            .wrap(RateLimitMiddleware::new(300.0, 50.0))
            .configure(|cfg| configure_app(cfg, pool_mgr_data.clone(), topo, gw))
    });

    let effective_workers = workers.unwrap_or(if topology.is_micro_mode {
        1
    } else {
        num_cpus()
    });
    server = server.workers(effective_workers);

    server.bind(addr)?.run().await
}

/// Runs the Actix Web Server on the specified address with default configuration.
pub async fn run_server(addr: &str) -> std::io::Result<()> {
    run_server_with_config(addr, MicroTopologyConfig::default(), None, None).await
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(std::num::NonZeroUsize::get)
        .unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_slug_valid() {
        assert_eq!(sanitize_slug("home-page"), Some("home_page".into()));
        assert_eq!(sanitize_slug("about_us_2026"), Some("about_us_2026".into()));
        assert_eq!(sanitize_slug("pricing"), Some("pricing".into()));
    }

    #[test]
    fn test_sanitize_slug_rejects_malicious() {
        assert_eq!(sanitize_slug(""), None);
        assert_eq!(sanitize_slug("home; DROP TABLE users;"), None);
        assert_eq!(sanitize_slug("../etc/passwd"), None);
        assert_eq!(sanitize_slug("foo$bar"), None);
    }
}
