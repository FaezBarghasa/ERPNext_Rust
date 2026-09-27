use actix_web::{
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::StatusCode,
    Error as ActixError, HttpMessage, HttpResponse, ResponseError,
};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use surrealdb::{engine::local::Mem, Surreal};
use thiserror::Error;
use tokio::sync::RwLock;

/// Unique Tenant Identifier.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(pub String);

/// Multi-tenant pool and routing errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TenantError {
    /// Remote/Local database connection failure.
    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),
    /// Tenant authentication failure.
    #[error("Tenant authentication failed: {0}")]
    AuthenticationFailed(String),
    /// Host header or X-Tenant-Id could not be resolved.
    #[error("Tenant unresolved from request")]
    TenantUnresolved,
    /// Pool acquisition timed out.
    #[error("Connection pool acquisition timeout")]
    PoolAcquisitionTimeout,
    /// Database namespace initialization failed.
    #[error("Namespace initialization failed: {0}")]
    NamespaceInitializationFailed(String),
    /// Tenant identifier is invalid.
    #[error("Invalid tenant identifier: must be alphanumeric (hyphens allowed) and <= 63 characters")]
    InvalidTenantIdentifier,
}

impl ResponseError for TenantError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::TenantUnresolved | Self::InvalidTenantIdentifier => StatusCode::BAD_REQUEST,
            Self::AuthenticationFailed(_) => StatusCode::UNAUTHORIZED,
            Self::PoolAcquisitionTimeout => StatusCode::GATEWAY_TIMEOUT,
            Self::ConnectionFailed(_) | Self::NamespaceInitializationFailed(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let status = self.status_code();
        let body = serde_json::json!({
            "type": "https://datatracker.ietf.org/doc/html/rfc7807",
            "title": status.canonical_reason().unwrap_or("Error"),
            "status": status.as_u16(),
            "detail": self.to_string(),
        });
        HttpResponse::build(status).json(body)
    }
}

/// Dynamic Connection Pool Manager for multi-tenant database handles.
#[derive(Clone)]
pub struct ConnectionPoolManager {
    pools: Arc<RwLock<HashMap<TenantId, (Surreal<surrealdb::engine::local::Db>, Instant)>>>,
    inactivity_threshold: Duration,
}

impl Default for ConnectionPoolManager {
    fn default() -> Self {
        Self::new(Duration::from_secs(300))
    }
}

impl ConnectionPoolManager {
    /// Creates a new connection pool manager.
    #[must_use]
    pub fn new(inactivity_threshold: Duration) -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            inactivity_threshold,
        }
    }

    /// Retrieves an active connection handle for the tenant, initializing if missing.
    pub async fn get_or_initialize_client(
        &self,
        tenant: &TenantId,
    ) -> Result<Surreal<surrealdb::engine::local::Db>, TenantError> {
        // 1. Fast read-lock lookup
        {
            let read_guard = self.pools.read().await;
            if let Some((client, _)) = read_guard.get(tenant) {
                return Ok(client.clone());
            }
        }

        // 2. Write-lock initialization
        let mut write_guard = self.pools.write().await;
        if let Some((client, _)) = write_guard.get(tenant) {
            return Ok(client.clone());
        }

        let db = Surreal::new::<Mem>(())
            .await
            .map_err(|e| TenantError::ConnectionFailed(e.to_string()))?;

        let ns = format!("tenant_{}", tenant.0.replace('-', "_"));
        db.use_ns(&ns)
            .use_db("erp")
            .await
            .map_err(|e| TenantError::NamespaceInitializationFailed(e.to_string()))?;

        write_guard.insert(tenant.clone(), (db.clone(), Instant::now()));
        Ok(db)
    }

    /// Evicts idle pools exceeding the configured inactivity duration.
    pub async fn evict_idle_pools(&self) -> usize {
        let mut write_guard = self.pools.write().await;
        let now = Instant::now();
        let threshold = self.inactivity_threshold;
        let initial_count = write_guard.len();

        write_guard.retain(|_, (_, last_used)| now.duration_since(*last_used) < threshold);
        initial_count - write_guard.len()
    }

    /// Starts a periodic background worker for evicting idle tenant connections.
    pub fn spawn_maintenance_worker(self: Arc<Self>, heartbeat: Duration) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(heartbeat);
            loop {
                interval.tick().await;
                self.evict_idle_pools().await;
            }
        });
    }
}

/// Resolves tenant identity from request headers or host string.
pub fn parse_tenant_id(headers: &actix_web::http::header::HeaderMap, host: &str) -> Result<TenantId, TenantError> {
    if let Some(tenant_hdr) = headers.get("X-Tenant-Id") {
        if let Ok(tenant_str) = tenant_hdr.to_str() {
            return validate_and_create_tenant_id(tenant_str);
        }
    }

    let host_regex = Regex::new(r"^(?P<tenant>[a-z0-9-]+)\.[a-z0-9.-]+$")
        .map_err(|e| TenantError::ConnectionFailed(e.to_string()))?;

    if let Some(caps) = host_regex.captures(host) {
        if let Some(m) = caps.name("tenant") {
            return validate_and_create_tenant_id(m.as_str());
        }
    }

    Err(TenantError::TenantUnresolved)
}

fn validate_and_create_tenant_id(tenant_str: &str) -> Result<TenantId, TenantError> {
    let sanitized = tenant_str.trim().to_lowercase();
    if sanitized.is_empty() || sanitized.len() > 63 {
        return Err(TenantError::InvalidTenantIdentifier);
    }
    if !sanitized.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(TenantError::InvalidTenantIdentifier);
    }
    Ok(TenantId(sanitized))
}

/// Actix Web Middleware for dynamic Tenant Resolution.
pub struct TenantResolver;

impl<S, B> Transform<S, ServiceRequest> for TenantResolver
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = ActixError;
    type InitError = ();
    type Transform = TenantResolverMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(TenantResolverMiddleware {
            service: Arc::new(service),
        })
    }
}

pub struct TenantResolverMiddleware<S> {
    service: Arc<S>,
}

impl<S, B> Service<ServiceRequest> for TenantResolverMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = ActixError;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let host = req.connection_info().host().to_string();
        let headers = req.headers().clone();

        match parse_tenant_id(&headers, &host) {
            Ok(tenant_id) => {
                req.extensions_mut().insert(tenant_id);
                let svc = self.service.clone();
                Box::pin(async move { svc.call(req).await })
            }
            Err(e) => Box::pin(async move { Err(ActixError::from(e)) }),
        }
    }
}

/// Cloud Multi-Tenant Provisioning Coordinator (Milestone 5.8).
pub async fn provision_tenant(
    pool_manager: &ConnectionPoolManager,
    tenant_id: &str,
) -> Result<Surreal<surrealdb::engine::local::Db>, TenantError> {
    let tenant = validate_and_create_tenant_id(tenant_id)?;
    let client = pool_manager.get_or_initialize_client(&tenant).await?;

    // Seed baseline core tables
    let seed_query = r#"
        DEFINE TABLE system_settings SCHEMAFULL;
        DEFINE FIELD tenant_name ON TABLE system_settings TYPE string;
        DEFINE FIELD created_at ON TABLE system_settings TYPE datetime DEFAULT time::now();
        CREATE system_settings SET tenant_name = $tenant_name;
    "#;

    client
        .query(seed_query)
        .bind(("tenant_name", tenant.0))
        .await
        .map_err(|e| TenantError::NamespaceInitializationFailed(e.to_string()))?
        .check()
        .map_err(|e| TenantError::NamespaceInitializationFailed(e.to_string()))?;

    Ok(client)
}
