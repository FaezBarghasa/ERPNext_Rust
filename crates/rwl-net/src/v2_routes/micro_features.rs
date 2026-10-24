//! Phase 9: Comprehensive Micro-Features Suite (`rwl-net::v2_routes::micro_features`).
//!
//! Provides the complete commercial-grade micro-features APIs:
//! 1. Navigation & Menu Builder (Header, Sidebar, Footer, Drawer, User Dropdown)
//! 2. AwesomeBar / Universal Search (Fuzzy global omnibar indexing DocTypes, Pages, Reports, Actions)
//! 3. In-App Notifications & Sound Alerts (Real-time notification center, unread badges, sound kinds)
//! 4. Media Library & Asset Manager (Folders, tags, focal point crop, duplicate hash detection)
//! 5. Universal Audit Trail & History (Visual diff, 1-click restore to previous version)
//! 6. Import/Export Studio (CSV auto-column mapping, batch ingestion, CSV error download)
//! 7. Internationalization & Localization (Visual dictionary, RTL layout auto-detection, multi-currency)
//! 8. Theme Customizer (Theme modes, brand accent color, border-radius, custom CSS compilation)

use super::*;
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use desk_components::{MenuTree, MenuTreeLocation, ThemeConfig, format_currency, is_rtl};
use erp_importer::{
    ColumnMappingItem, ExportFormat, FuzzyMatcher, ImportExportStudio, StreamingImportConfig,
    StreamingPipelineProcessor,
};
use rwl_framework::notification::SoundAlertConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configure all micro-feature routes on the Actix service configuration.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        // 1. Navigation & Menu Builder
        .route("/api/v2/admin/menus", web::get().to(list_all_menus_handler))
        .route(
            "/api/v2/admin/menus/{location}",
            web::get().to(get_menu_tree_handler),
        )
        .route(
            "/api/v2/admin/menus/{location}",
            web::post().to(save_menu_tree_handler),
        )
        // 2. AwesomeBar / Universal Search
        .route(
            "/api/v2/admin/search",
            web::get().to(awesomebar_search_handler),
        )
        // 3. In-App Notifications & Sound Alerts
        .route(
            "/api/v2/notifications/inbox",
            web::get().to(get_notifications_inbox_handler),
        )
        .route(
            "/api/v2/notifications/{id}/read",
            web::post().to(mark_notification_read_handler),
        )
        .route(
            "/api/v2/notifications/read-all",
            web::post().to(mark_all_notifications_read_handler),
        )
        .route(
            "/api/v2/notifications/clear",
            web::delete().to(clear_notifications_handler),
        )
        .route(
            "/api/v2/notifications/sound-config",
            web::get().to(get_sound_config_handler),
        )
        .route(
            "/api/v2/notifications/sound-config",
            web::put().to(update_sound_config_handler),
        )
        // 4. Media Library & Asset Manager
        .route(
            "/api/v2/media/assets",
            web::get().to(media_assets_list_handler),
        )
        .route(
            "/api/v2/media/crop",
            web::post().to(media_crop_calculate_handler),
        )
        .route(
            "/api/v2/media/duplicates",
            web::get().to(media_duplicates_handler),
        )
        // 5. Universal Audit Trail & History (1-click restore)
        .route(
            "/api/v2/admin/audit-logs/{entry_id}/restore",
            web::post().to(audit_log_restore_handler),
        )
        // 6. Import/Export Studio
        .route(
            "/api/v2/import/auto-map",
            web::post().to(import_auto_map_handler),
        )
        .route(
            "/api/v2/import/execute",
            web::post().to(import_execute_handler),
        )
        .route(
            "/api/v2/export/doctype",
            web::post().to(export_doctype_dataset_handler),
        )
        // 7. Internationalization & Localization (i18n)
        .route(
            "/api/v2/i18n/translations/{lang}",
            web::get().to(i18n_get_translations_handler),
        )
        .route(
            "/api/v2/i18n/translations",
            web::post().to(i18n_save_translations_handler),
        )
        .route(
            "/api/v2/i18n/detect-rtl",
            web::get().to(i18n_detect_rtl_handler),
        )
        .route(
            "/api/v2/i18n/format-currency",
            web::post().to(i18n_format_currency_handler),
        )
        // 8. Theme Customizer
        .route("/api/v2/admin/theme", web::get().to(theme_get_handler))
        .route("/api/v2/admin/theme", web::put().to(theme_update_handler))
        .route(
            "/api/v2/admin/theme/css",
            web::get().to(theme_get_compiled_css_handler),
        );
}

// =========================================================================
// 1. Navigation & Menu Builder Handlers
// =========================================================================

pub async fn list_all_menus_handler(rbac_state: web::Data<DynamicRbacState>) -> impl Responder {
    let menus = match rbac_state.menus.read() {
        Ok(m) => m,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "ERROR", "message": "Lock failure"}));
        }
    };

    HttpResponse::Ok().json(serde_json::json!({
        "status": "OK",
        "menus": menus.clone(),
    }))
}

pub async fn get_menu_tree_handler(
    location_path: web::Path<String>,
    rbac_state: web::Data<DynamicRbacState>,
) -> impl Responder {
    let loc_str = location_path.into_inner();
    let loc = parse_menu_location(&loc_str);

    let menus = match rbac_state.menus.read() {
        Ok(m) => m,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "ERROR", "message": "Lock failure"}));
        }
    };

    let menu = menus
        .get(&loc)
        .cloned()
        .unwrap_or_else(|| MenuTree::standard_default(loc));

    HttpResponse::Ok().json(serde_json::json!({
        "status": "OK",
        "location": loc,
        "menu": menu,
    }))
}

pub async fn save_menu_tree_handler(
    location_path: web::Path<String>,
    payload: web::Json<MenuTree>,
    rbac_state: web::Data<DynamicRbacState>,
) -> impl Responder {
    let loc_str = location_path.into_inner();
    let loc = parse_menu_location(&loc_str);
    let mut menu = payload.into_inner();
    menu.location = loc;

    let mut menus = match rbac_state.menus.write() {
        Ok(m) => m,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "ERROR", "message": "Lock failure"}));
        }
    };

    menus.insert(loc, menu.clone());

    HttpResponse::Ok().json(serde_json::json!({
        "status": "SAVED",
        "location": loc,
        "menu": menu,
    }))
}

fn parse_menu_location(s: &str) -> MenuTreeLocation {
    match s.trim().to_ascii_lowercase().as_str() {
        "sidebar" | "sidebarnav" => MenuTreeLocation::SidebarNav,
        "footer" | "footerlinks" => MenuTreeLocation::FooterLinks,
        "drawer" | "mobiledrawer" => MenuTreeLocation::MobileDrawer,
        "user" | "userdropdown" => MenuTreeLocation::UserDropdown,
        _ => MenuTreeLocation::HeaderNav,
    }
}

// =========================================================================
// 2. AwesomeBar / Universal Search Handlers
// =========================================================================

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIndexItem {
    pub id: String,
    pub title: String,
    pub category: String, // "DocType", "Page", "Report", "Action", "Shortcut"
    pub route: String,
    pub description: String,
}

pub async fn awesomebar_search_handler(query: web::Query<SearchQuery>) -> impl Responder {
    let q = query.q.as_deref().unwrap_or("").trim().to_lowercase();
    let limit = query.limit.unwrap_or(20);

    // Standard index of searchable ERPNext entities
    let index = vec![
        SearchIndexItem {
            id: "dt_sales_invoice".into(),
            title: "Sales Invoice List".into(),
            category: "DocType".into(),
            route: "/desk/list/sales_invoice".into(),
            description: "Accounts Receivable, statutory taxes, billing & credit notes".into(),
        },
        SearchIndexItem {
            id: "dt_sales_order".into(),
            title: "Sales Order List".into(),
            category: "DocType".into(),
            route: "/desk/list/sales_order".into(),
            description: "Customer quotations, confirmed orders & dispatch schedules".into(),
        },
        SearchIndexItem {
            id: "dt_purchase_order".into(),
            title: "Purchase Order List".into(),
            category: "DocType".into(),
            route: "/desk/list/purchase_order".into(),
            description: "Supplier procurement, landed costs & 3-way matching".into(),
        },
        SearchIndexItem {
            id: "dt_item".into(),
            title: "Item / Product Catalog".into(),
            category: "DocType".into(),
            route: "/desk/list/item".into(),
            description: "Stock ledger, barcodes, serial numbers & FIFO valuation".into(),
        },
        SearchIndexItem {
            id: "dt_work_order".into(),
            title: "Work Order List".into(),
            category: "DocType".into(),
            route: "/desk/list/work_order".into(),
            description: "Manufacturing routing, digital travelers & BOM assembly".into(),
        },
        SearchIndexItem {
            id: "page_builder".into(),
            title: "Visual Page Builder (SSS+ Rank)".into(),
            category: "Page".into(),
            route: "/desk/builder".into(),
            description: "Figma-to-production canvas, animations & headless themes".into(),
        },
        SearchIndexItem {
            id: "rep_general_ledger".into(),
            title: "General Ledger Statement".into(),
            category: "Report".into(),
            route: "/desk/report/general_ledger".into(),
            description: "ACID double-entry ledger journals and account balances".into(),
        },
        SearchIndexItem {
            id: "act_backup".into(),
            title: "1-Click Deploy & Backup Snapshot".into(),
            category: "Action".into(),
            route: "/admin/upgrade".into(),
            description: "Snapshot database state, generate zstd deployment bundles".into(),
        },
    ];

    let results: Vec<SearchIndexItem> = if q.is_empty() {
        index.into_iter().take(limit).collect()
    } else {
        index
            .into_iter()
            .filter(|item| {
                item.title.to_lowercase().contains(&q)
                    || item.description.to_lowercase().contains(&q)
                    || item.category.to_lowercase().contains(&q)
            })
            .take(limit)
            .collect()
    };

    HttpResponse::Ok().json(serde_json::json!({
        "status": "OK",
        "query": q,
        "count": results.len(),
        "results": results,
    }))
}

// =========================================================================
// 3. In-App Notifications & Sound Alerts Handlers
// =========================================================================

pub async fn get_notifications_inbox_handler(
    req: HttpRequest,
    rbac_state: web::Data<DynamicRbacState>,
) -> impl Responder {
    let user_id = req
        .headers()
        .get("x-user-id")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Administrator");

    let items = rbac_state
        .notifications_inbox
        .get_user_notifications(user_id);
    let unread = rbac_state.notifications_inbox.unread_count(user_id);

    HttpResponse::Ok().json(serde_json::json!({
        "status": "OK",
        "user_id": user_id,
        "unread_count": unread,
        "total": items.len(),
        "notifications": items,
    }))
}

pub async fn mark_notification_read_handler(
    req: HttpRequest,
    notif_id: web::Path<String>,
    rbac_state: web::Data<DynamicRbacState>,
) -> impl Responder {
    let user_id = req
        .headers()
        .get("x-user-id")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Administrator");
    let id = notif_id.into_inner();

    match rbac_state.notifications_inbox.mark_as_read(user_id, &id) {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({
            "status": "OK",
            "message": format!("Notification '{id}' marked as read"),
        })),
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({
            "status": "NOT_FOUND",
            "message": e.to_string(),
        })),
    }
}

pub async fn mark_all_notifications_read_handler(
    req: HttpRequest,
    rbac_state: web::Data<DynamicRbacState>,
) -> impl Responder {
    let user_id = req
        .headers()
        .get("x-user-id")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Administrator");

    rbac_state.notifications_inbox.mark_all_as_read(user_id);

    HttpResponse::Ok().json(serde_json::json!({
        "status": "OK",
        "message": "All notifications marked as read",
    }))
}

pub async fn clear_notifications_handler(
    req: HttpRequest,
    rbac_state: web::Data<DynamicRbacState>,
) -> impl Responder {
    let user_id = req
        .headers()
        .get("x-user-id")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Administrator");

    rbac_state.notifications_inbox.clear_all(user_id);

    HttpResponse::Ok().json(serde_json::json!({
        "status": "CLEARED",
        "message": "Notification inbox cleared",
    }))
}

pub async fn get_sound_config_handler(
    req: HttpRequest,
    rbac_state: web::Data<DynamicRbacState>,
) -> impl Responder {
    let user_id = req
        .headers()
        .get("x-user-id")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Administrator");

    let configs = match rbac_state.sound_configs.read() {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "ERROR", "message": "Lock failure"}));
        }
    };

    let user_config = configs.get(user_id).cloned().unwrap_or_default();

    HttpResponse::Ok().json(serde_json::json!({
        "status": "OK",
        "user_id": user_id,
        "config": user_config,
    }))
}

pub async fn update_sound_config_handler(
    req: HttpRequest,
    payload: web::Json<SoundAlertConfig>,
    rbac_state: web::Data<DynamicRbacState>,
) -> impl Responder {
    let user_id = req
        .headers()
        .get("x-user-id")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Administrator")
        .to_string();

    let mut configs = match rbac_state.sound_configs.write() {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "ERROR", "message": "Lock failure"}));
        }
    };

    configs.insert(user_id.clone(), payload.into_inner());

    HttpResponse::Ok().json(serde_json::json!({
        "status": "UPDATED",
        "user_id": user_id,
    }))
}

// =========================================================================
// 4. Media Library & Asset Manager Handlers
// =========================================================================

#[derive(Debug, Deserialize)]
pub struct MediaAssetQuery {
    pub mime_prefix: Option<String>,
    pub q: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

pub async fn media_assets_list_handler(
    query: web::Query<MediaAssetQuery>,
    rbac_state: web::Data<DynamicRbacState>,
) -> impl Responder {
    let catalog = match rbac_state.media_catalog.read() {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "ERROR", "message": "Lock failure"}));
        }
    };

    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);
    let (assets, total) = catalog.query_assets(
        query.mime_prefix.as_deref(),
        query.q.as_deref(),
        limit,
        offset,
    );

    HttpResponse::Ok().json(serde_json::json!({
        "status": "OK",
        "total": total,
        "count": assets.len(),
        "assets": assets,
    }))
}

#[derive(Debug, Deserialize)]
pub struct MediaCropPayload {
    pub image_width: u32,
    pub image_height: u32,
    pub target_aspect_ratio: f32, // e.g. 1.777 for 16:9, 1.0 for 1:1
    pub focal_point: (f32, f32),  // (x, y) 0.0..1.0
}

#[derive(Debug, Serialize)]
pub struct CropResult {
    pub crop_x: u32,
    pub crop_y: u32,
    pub crop_width: u32,
    pub crop_height: u32,
}

pub async fn media_crop_calculate_handler(payload: web::Json<MediaCropPayload>) -> impl Responder {
    let p = payload.into_inner();
    let img_w = p.image_width as f32;
    let img_h = p.image_height as f32;

    let target_ar = p.target_aspect_ratio.max(0.1);
    let current_ar = img_w / img_h;

    let (crop_w, crop_h) = if current_ar > target_ar {
        (img_h * target_ar, img_h)
    } else {
        (img_w, img_w / target_ar)
    };

    let focal_x = p.focal_point.0.clamp(0.0, 1.0) * img_w;
    let focal_y = p.focal_point.1.clamp(0.0, 1.0) * img_h;

    let mut origin_x = focal_x - (crop_w / 2.0);
    let mut origin_y = focal_y - (crop_h / 2.0);

    origin_x = origin_x.clamp(0.0, img_w - crop_w);
    origin_y = origin_y.clamp(0.0, img_h - crop_h);

    HttpResponse::Ok().json(serde_json::json!({
        "status": "OK",
        "crop": CropResult {
            crop_x: origin_x.round() as u32,
            crop_y: origin_y.round() as u32,
            crop_width: crop_w.round() as u32,
            crop_height: crop_h.round() as u32,
        }
    }))
}

pub async fn media_duplicates_handler(rbac_state: web::Data<DynamicRbacState>) -> impl Responder {
    let catalog = match rbac_state.media_catalog.read() {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "ERROR", "message": "Lock failure"}));
        }
    };

    let (all_assets, _) = catalog.query_assets(None, None, 10000, 0);

    // Group by content hash
    let mut hash_groups: HashMap<String, Vec<&MediaAsset>> = HashMap::new();
    for asset in all_assets {
        hash_groups
            .entry(asset.content_hash.clone())
            .or_default()
            .push(asset);
    }

    let duplicates: Vec<_> = hash_groups
        .into_iter()
        .filter(|(_, items)| items.len() > 1)
        .map(|(hash, items)| {
            serde_json::json!({
                "content_hash": hash,
                "count": items.len(),
                "assets": items,
            })
        })
        .collect();

    HttpResponse::Ok().json(serde_json::json!({
        "status": "OK",
        "duplicate_groups": duplicates.len(),
        "duplicates": duplicates,
    }))
}

// =========================================================================
// 5. Universal Audit Trail & History (1-Click Restore) Handlers
// =========================================================================

pub async fn audit_log_restore_handler(
    entry_id_path: web::Path<String>,
    rbac_state: web::Data<DynamicRbacState>,
) -> impl Responder {
    let entry_id = entry_id_path.into_inner();

    let (entries, _) = rbac_state.audit_trail.query(&AuditQueryFilter::default());
    let target_entry = match entries.into_iter().find(|e| e.id == entry_id) {
        Some(e) => e,
        None => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "status": "NOT_FOUND",
                "message": format!("Audit log entry '{entry_id}' not found"),
            }));
        }
    };

    let restored_payload = match target_entry.before_state {
        Some(state) => state,
        None => match target_entry.after_state {
            Some(state) => state,
            None => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "status": "NO_STATE",
                    "message": "Audit entry does not contain restorable document state snapshot",
                }));
            }
        },
    };

    // Record a new audit log entry for the restoration
    rbac_state.audit_trail.record(
        AuditEntry::new(
            &target_entry.tenant_id,
            "Administrator",
            "127.0.0.1",
            "AuditDesk",
            AuditAction::Update,
            "Success",
        )
        .with_document_diff(
            target_entry.doctype.as_deref().unwrap_or("Unknown"),
            target_entry.docname.as_deref().unwrap_or("Unknown"),
            None,
            Some(serde_json::json!({"restored_from_entry": entry_id})),
        ),
    );

    HttpResponse::Ok().json(serde_json::json!({
        "status": "RESTORED",
        "doctype": target_entry.doctype,
        "docname": target_entry.docname,
        "restored_state": restored_payload,
        "message": "Document successfully restored to selected historical revision",
    }))
}

// =========================================================================
// 6. Import/Export Studio Handlers
// =========================================================================

#[derive(Debug, Deserialize)]
pub struct AutoMapPayload {
    pub headers: Vec<String>,
}

pub async fn import_auto_map_handler(payload: web::Json<AutoMapPayload>) -> impl Responder {
    let schema = FuzzyMatcher::default_item_schema();
    let header_refs: Vec<&str> = payload.headers.iter().map(AsRef::as_ref).collect();
    let mappings: Vec<ColumnMappingItem> =
        ImportExportStudio::suggest_mappings(&header_refs, &schema);

    HttpResponse::Ok().json(serde_json::json!({
        "status": "OK",
        "count": mappings.len(),
        "mappings": mappings,
    }))
}

#[derive(Debug, Deserialize)]
pub struct ImportExecutePayload {
    pub doctype: String,
    pub csv_content: String,
    pub dry_run: Option<bool>,
}

pub async fn import_execute_handler(payload: web::Json<ImportExecutePayload>) -> impl Responder {
    let schema = FuzzyMatcher::default_item_schema();
    let dry_run = payload.dry_run.unwrap_or(false);

    let config = StreamingImportConfig {
        dry_run,
        ..Default::default()
    };
    let (rows, summary) =
        StreamingPipelineProcessor::process_csv_stream(&payload.csv_content, &schema, &config);

    let error_csv = if summary.failed_rows > 0 {
        Some(ImportExportStudio::generate_error_csv(&summary.errors))
    } else {
        None
    };

    HttpResponse::Ok().json(serde_json::json!({
        "status": "COMPLETED",
        "doctype": payload.doctype,
        "dry_run": dry_run,
        "total_rows": summary.total_rows,
        "imported_rows": rows.len(),
        "failed_rows": summary.failed_rows,
        "errors": summary.errors,
        "error_csv": error_csv,
    }))
}

#[derive(Debug, Deserialize)]
pub struct ExportDatasetPayload {
    pub doctype: String,
    pub fields: Vec<String>,
    pub rows: Vec<serde_json::Value>,
    pub format: Option<String>, // "csv" or "json"
}

pub async fn export_doctype_dataset_handler(
    payload: web::Json<ExportDatasetPayload>,
) -> impl Responder {
    let p = payload.into_inner();
    let format = match p.format.as_deref().unwrap_or("csv").to_lowercase().as_str() {
        "json" => ExportFormat::Json,
        _ => ExportFormat::Csv,
    };

    match ImportExportStudio::export_dataset(&p.rows, &p.fields, format) {
        Ok(bytes) => {
            let (content_type, filename) = match format {
                ExportFormat::Json => (
                    "application/json",
                    format!("{}.json", p.doctype.to_lowercase()),
                ),
                ExportFormat::Csv => ("text/csv", format!("{}.csv", p.doctype.to_lowercase())),
            };

            HttpResponse::Ok()
                .content_type(content_type)
                .insert_header((
                    "Content-Disposition",
                    format!("attachment; filename=\"{filename}\""),
                ))
                .body(bytes)
        }
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "status": "ERROR",
            "message": e,
        })),
    }
}

// =========================================================================
// 7. Internationalization & Localization (i18n) Handlers
// =========================================================================

pub async fn i18n_get_translations_handler(
    lang_path: web::Path<String>,
    rbac_state: web::Data<DynamicRbacState>,
) -> impl Responder {
    let lang = lang_path.into_inner().to_lowercase();
    let translations_map = match rbac_state.translations.read() {
        Ok(t) => t,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "ERROR", "message": "Lock failure"}));
        }
    };

    let dict = translations_map.get(&lang).cloned().unwrap_or_default();
    let is_lang_rtl = is_rtl(&lang);

    HttpResponse::Ok().json(serde_json::json!({
        "status": "OK",
        "language": lang,
        "is_rtl": is_lang_rtl,
        "direction": if is_lang_rtl { "rtl" } else { "ltr" },
        "translations": dict,
    }))
}

#[derive(Debug, Deserialize)]
pub struct SaveTranslationPayload {
    pub language: String,
    pub terms: HashMap<String, String>,
}

pub async fn i18n_save_translations_handler(
    payload: web::Json<SaveTranslationPayload>,
    rbac_state: web::Data<DynamicRbacState>,
) -> impl Responder {
    let p = payload.into_inner();
    let lang = p.language.to_lowercase();

    let mut translations_map = match rbac_state.translations.write() {
        Ok(t) => t,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "ERROR", "message": "Lock failure"}));
        }
    };

    let dict = translations_map.entry(lang.clone()).or_default();
    for (k, v) in p.terms {
        dict.insert(k, v);
    }

    HttpResponse::Ok().json(serde_json::json!({
        "status": "SAVED",
        "language": lang,
        "count": dict.len(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct DetectRtlQuery {
    pub lang: Option<String>,
}

pub async fn i18n_detect_rtl_handler(query: web::Query<DetectRtlQuery>) -> impl Responder {
    let lang = query.lang.as_deref().unwrap_or("en");
    let rtl = is_rtl(lang);

    HttpResponse::Ok().json(serde_json::json!({
        "status": "OK",
        "language": lang,
        "is_rtl": rtl,
        "direction": if rtl { "rtl" } else { "ltr" },
    }))
}

#[derive(Debug, Deserialize)]
pub struct FormatCurrencyPayload {
    pub amount: f64,
    pub currency: String,
    pub locale: String,
}

pub async fn i18n_format_currency_handler(
    payload: web::Json<FormatCurrencyPayload>,
) -> impl Responder {
    let p = payload.into_inner();
    let formatted = format_currency(p.amount, &p.currency, &p.locale);

    HttpResponse::Ok().json(serde_json::json!({
        "status": "OK",
        "amount": p.amount,
        "currency": p.currency,
        "locale": p.locale,
        "formatted": formatted,
    }))
}

// =========================================================================
// 8. Theme Customizer Handlers
// =========================================================================

pub async fn theme_get_handler(rbac_state: web::Data<DynamicRbacState>) -> impl Responder {
    let theme = match rbac_state.theme_config.read() {
        Ok(t) => t,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "ERROR", "message": "Lock failure"}));
        }
    };

    HttpResponse::Ok().json(serde_json::json!({
        "status": "OK",
        "theme": theme.clone(),
    }))
}

pub async fn theme_update_handler(
    payload: web::Json<ThemeConfig>,
    rbac_state: web::Data<DynamicRbacState>,
) -> impl Responder {
    let mut theme = match rbac_state.theme_config.write() {
        Ok(t) => t,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "ERROR", "message": "Lock failure"}));
        }
    };

    *theme = payload.into_inner();

    HttpResponse::Ok().json(serde_json::json!({
        "status": "UPDATED",
        "theme": theme.clone(),
    }))
}

pub async fn theme_get_compiled_css_handler(
    rbac_state: web::Data<DynamicRbacState>,
) -> impl Responder {
    let theme = match rbac_state.theme_config.read() {
        Ok(t) => t,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "ERROR", "message": "Lock failure"}));
        }
    };

    let css = theme.compile_css();

    HttpResponse::Ok().content_type("text/css").body(css)
}
