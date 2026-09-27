use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web::FromRequest;
use std::future::{ready, Ready};
use std::collections::HashMap;
use frappe_storage::TenantContext;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct TenantHeader { pub tenant_id: String, pub site_name: String }

impl FromRequest for TenantHeader {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;
    /// Resolution pipeline: X-Frappe-Site-Name header -> SNI host fallback (Stage 2.1.1).
    fn from_request(req: &HttpRequest, _pl: &mut actix_web::dev::Payload) -> Self::Future {
        let mut headers = HashMap::new();
        for (k, v) in req.headers().iter() {
            if let Ok(s) = v.to_str() { headers.insert(k.to_string(), s.to_string()); }
        }
        // normalize: actix lowercases header names
        let site_hdr = headers.get("x-frappe-site-name").cloned()
            .or_else(|| headers.get("X-Frappe-Site-Name").cloned());
        let host = req.connection_info().host().to_string();
        let mut h2 = HashMap::new();
        if let Some(s) = site_hdr { h2.insert("X-Frappe-Site-Name".into(), s); }
        match TenantContext::from_headers(&h2, &host) {
            Some(ctx) => ready(Ok(TenantHeader { tenant_id: ctx.tenant_id, site_name: ctx.site_name })),
            None => ready(Err(actix_web::error::ErrorBadRequest("unknown tenant"))),
        }
    }
}

async fn health(tenant: TenantHeader) -> impl Responder {
    HttpResponse::Ok().json(&tenant)
}

async fn live_ws(req: HttpRequest, body: web::Payload) -> actix_web::Result<HttpResponse> {
    let (res, mut session, mut stream) = actix_ws::handle(&req, body)?;
    actix_web::rt::spawn(async move {
        use futures_util::StreamExt as _;
        while let Some(Ok(msg)) = stream.next().await {
            if let actix_ws::Message::Text(t) = msg {
                // echo LIVE diff frame: LIVE SELECT registration stub
                let _ = session.text(format!("LIVE-ACK:{}", t)).await;
            }
        }
    });
    Ok(res)
}

pub async fn run(addr: &str) -> std::io::Result<()> {
    HttpServer::new(|| App::new()
        .route("/health", web::get().to(health))
        .route("/ws/live", web::get().to(live_ws)))
        .bind(addr)?.run().await
}
