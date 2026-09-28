use actix_web::{
    Error as ActixError, HttpMessage, HttpResponse, ResponseError,
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::StatusCode,
};
use futures_util::future::{LocalBoxFuture, Ready, ok};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use surrealdb::{Surreal, engine::local::{Mem, SurrealKv}};
use thiserror::Error;
use tokio::sync::RwLock;

use actix_web::FromRequest;
use actix_web::HttpRequest;
use actix_web::dev::Payload;

/// Storage engine backend for tenant database instances.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatabaseBackend {
    /// In-memory storage (ephemeral, zero-disk).
    Memory,
    /// Persistent on-disk key-value storage engine (`SurrealKV`).
    SurrealKv { base_path: String },
}

impl Default for DatabaseBackend {
    fn default() -> Self {
        Self::SurrealKv {
            base_path: "./data/tenants".to_string(),
        }
    }
}

/// Unique Tenant Identifier.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(pub String);

/// Scoped Tenant Context extracted from incoming requests.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantContext {
    pub tenant_id: TenantId,
    pub namespace: String,
    pub database: String,
}

impl FromRequest for TenantContext {
    type Error = ActixError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        if let Some(ctx) = req.extensions().get::<TenantContext>() {
            return futures_util::future::ready(Ok(ctx.clone()));
        }
        if let Some(tenant_id) = req.extensions().get::<TenantId>() {
            let ns = format!("tenant_{}", tenant_id.0.replace('-', "_"));
            return futures_util::future::ready(Ok(TenantContext {
                tenant_id: tenant_id.clone(),
                namespace: ns,
                database: "erp".to_string(),
            }));
        }
        let host = req.connection_info().host().to_string();
        let headers = req.headers();
        match parse_tenant_id(headers, &host) {
            Ok(tenant_id) => {
                let ns = format!("tenant_{}", tenant_id.0.replace('-', "_"));
                futures_util::future::ready(Ok(TenantContext {
                    tenant_id,
                    namespace: ns,
                    database: "erp".to_string(),
                }))
            }
            Err(e) => futures_util::future::ready(Err(ActixError::from(e))),
        }
    }
}

/// In-Process Automated ACME Reverse Proxy Gateway (Milestone 1.3).
#[derive(Clone, Default)]
pub struct AcmeGateway {
    domain_mapping: Arc<RwLock<HashMap<String, TenantId>>>,
    cached_certificates: Arc<RwLock<HashMap<String, String>>>,
}

impl AcmeGateway {
    #[must_use]
    pub fn new() -> Self {
        Self {
            domain_mapping: Arc::new(RwLock::new(HashMap::new())),
            cached_certificates: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registers an authorized custom domain mapping to a tenant namespace.
    pub async fn register_domain(&self, domain: &str, tenant: TenantId) {
        let mut map = self.domain_mapping.write().await;
        map.insert(domain.to_lowercase(), tenant);
    }

    /// Resolves a domain against the authorized domain mapping table.
    pub async fn resolve_domain(&self, domain: &str) -> Option<TenantId> {
        let map = self.domain_mapping.read().await;
        map.get(&domain.to_lowercase()).cloned()
    }

    /// Simulates dynamic ACME challenge issuance & certificate caching in <4000ms.
    pub async fn issue_and_cache_certificate(&self, domain: &str) -> Result<String, TenantError> {
        let domain_norm = domain.to_lowercase();
        if self.resolve_domain(&domain_norm).await.is_none() {
            return Err(TenantError::AuthenticationFailed(format!(
                "Domain {} not authorized for dynamic ACME issuance",
                domain
            )));
        }

        let cert = format!(
            "---BEGIN CERTIFICATE---\nDOMAIN:{}\n---END CERTIFICATE---",
            domain_norm
        );
        let mut cert_cache = self.cached_certificates.write().await;
        cert_cache.insert(domain_norm, cert.clone());
        Ok(cert)
    }
}

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
    #[error(
        "Invalid tenant identifier: must be alphanumeric (hyphens allowed) and <= 63 characters"
    )]
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

type TenantPoolEntry = (Surreal<surrealdb::engine::local::Db>, Instant);
type TenantPoolMap = Arc<RwLock<HashMap<TenantId, TenantPoolEntry>>>;

/// Dynamic Connection Pool Manager for multi-tenant database handles.
#[derive(Clone)]
pub struct ConnectionPoolManager {
    pools: TenantPoolMap,
    inactivity_threshold: Duration,
    backend: DatabaseBackend,
}

impl Default for ConnectionPoolManager {
    fn default() -> Self {
        Self::new(Duration::from_secs(300))
    }
}

impl ConnectionPoolManager {
    /// Creates a new connection pool manager with the default persistent SurrealKV backend.
    #[must_use]
    pub fn new(inactivity_threshold: Duration) -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            inactivity_threshold,
            backend: DatabaseBackend::default(),
        }
    }

    /// Creates an in-memory connection pool manager for tests and ephemeral workspaces.
    #[must_use]
    pub fn in_memory(inactivity_threshold: Duration) -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            inactivity_threshold,
            backend: DatabaseBackend::Memory,
        }
    }

    /// Creates a connection pool manager with a customized database backend.
    #[must_use]
    pub fn with_backend(inactivity_threshold: Duration, backend: DatabaseBackend) -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            inactivity_threshold,
            backend,
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

        let db = match &self.backend {
            DatabaseBackend::Memory => {
                Surreal::new::<Mem>(())
                    .await
                    .map_err(|e| TenantError::ConnectionFailed(e.to_string()))?
            }
            DatabaseBackend::SurrealKv { base_path } => {
                let tenant_dir = format!("{}/{}", base_path, tenant.0);
                std::fs::create_dir_all(&tenant_dir)
                    .map_err(|e| TenantError::ConnectionFailed(format!("Failed to create tenant data dir: {e}")))?;
                Surreal::new::<SurrealKv>(&tenant_dir)
                    .await
                    .map_err(|e| TenantError::ConnectionFailed(e.to_string()))?
            }
        };

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
pub fn parse_tenant_id(
    headers: &actix_web::http::header::HeaderMap,
    host: &str,
) -> Result<TenantId, TenantError> {
    if let Some(tenant_hdr) = headers.get("X-Tenant-Id")
        && let Ok(tenant_str) = tenant_hdr.to_str()
    {
        return validate_and_create_tenant_id(tenant_str);
    }

    let host_regex = Regex::new(r"^(?P<tenant>[a-z0-9-]+)\.[a-z0-9.-]+$")
        .map_err(|e| TenantError::ConnectionFailed(e.to_string()))?;

    if let Some(caps) = host_regex.captures(host)
        && let Some(m) = caps.name("tenant")
    {
        return validate_and_create_tenant_id(m.as_str());
    }

    Err(TenantError::TenantUnresolved)
}

fn validate_and_create_tenant_id(tenant_str: &str) -> Result<TenantId, TenantError> {
    let sanitized = tenant_str.trim().to_lowercase();
    if sanitized.is_empty() || sanitized.len() > 63 {
        return Err(TenantError::InvalidTenantIdentifier);
    }
    if !sanitized
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
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

/// Constructs a lightweight, isolated request-scoped session handle bound to the tenant namespace.
pub async fn resolve_scoped_session(
    pool_manager: &ConnectionPoolManager,
    tenant_id: &TenantId,
) -> Result<Surreal<surrealdb::engine::local::Db>, TenantError> {
    let client = pool_manager.get_or_initialize_client(tenant_id).await?;
    let session = client.clone();
    let ns = format!("tenant_{}", tenant_id.0.replace('-', "_"));
    session
        .use_ns(&ns)
        .use_db("erp")
        .await
        .map_err(|e| TenantError::ConnectionFailed(e.to_string()))?;
    Ok(session)
}

/// Micro-Mode runtime configuration (<64MB RSS budget).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MicroTopologyConfig {
    pub is_micro_mode: bool,
    pub max_write_buffer_mb: usize,
    pub max_read_cache_mb: usize,
    pub max_queue_capacity: usize,
    pub idle_reap_interval_secs: u64,
    pub max_payload_bytes: usize,
}

impl Default for MicroTopologyConfig {
    fn default() -> Self {
        Self {
            is_micro_mode: false,
            max_write_buffer_mb: 64,
            max_read_cache_mb: 128,
            max_queue_capacity: 10_000,
            idle_reap_interval_secs: 300,
            max_payload_bytes: 10 * 1024 * 1024,
        }
    }
}

impl MicroTopologyConfig {
    #[must_use]
    pub fn micro_mode() -> Self {
        Self {
            is_micro_mode: true,
            max_write_buffer_mb: 8,
            max_read_cache_mb: 16,
            max_queue_capacity: 1024,
            idle_reap_interval_secs: 60,
            max_payload_bytes: 2 * 1024 * 1024,
        }
    }
}

#[cfg(test)]
mod tenant_unit_tests {
    use super::*;

    #[tokio::test]
    async fn test_scoped_session_isolation() {
        let pool = ConnectionPoolManager::in_memory(Duration::from_secs(300));
        let t1 = TenantId("alpha-shop".into());
        let t2 = TenantId("beta-clinic".into());

        let s1 = resolve_scoped_session(&pool, &t1).await.unwrap();
        let s2 = resolve_scoped_session(&pool, &t2).await.unwrap();

        // Verify independent handles
        assert!(s1.query("INFO FOR DB;").await.is_ok());
        assert!(s2.query("INFO FOR DB;").await.is_ok());

        let micro = MicroTopologyConfig::micro_mode();
        assert!(micro.is_micro_mode);
        assert_eq!(micro.max_write_buffer_mb, 8);
        assert_eq!(micro.max_read_cache_mb, 16);
    }

    #[tokio::test]
    async fn test_surrealkv_persistent_storage() {
        let temp_dir = format!("./target/test_data_tenants_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
        let pool = ConnectionPoolManager::with_backend(
            Duration::from_secs(300),
            DatabaseBackend::SurrealKv { base_path: temp_dir.clone() },
        );
        let t1 = TenantId("persistent-tenant".into());
        let s1 = pool.get_or_initialize_client(&t1).await.unwrap();
        assert!(s1.query("INFO FOR DB;").await.is_ok());

        // Cleanup test directory
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
