//! IP Allowlist & Blocklist Access Control Middleware (`frappe-net::middleware::ip_filter`).
//!
//! Enforces per-tenant and global IP filtering rules (CIDR, subnet wildcards, exact IPs)
//! at the HTTP edge, immediately rejecting prohibited clients with 403 Forbidden.

use actix_web::body::{BoxBody, MessageBody};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::StatusCode;
use actix_web::{Error as ActixError, HttpResponse};
use chrono::Utc;
use frappe_meta::security_rules::matches_ip_rule;
use futures_util::future::{LocalBoxFuture, Ready, ok, ready};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

/// An IP access control filtering rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IpFilterRule {
    pub id: String,
    pub tenant_id: Option<String>,
    pub rule_type: String, // "allow" or "block"
    pub pattern: String,   // e.g. "192.168.1.*", "10.0.0.0/8", "203.0.113.50"
    pub description: String,
    pub created_at: String,
}

impl IpFilterRule {
    #[must_use]
    pub fn new(tenant_id: Option<&str>, rule_type: &str, pattern: &str, description: &str) -> Self {
        Self {
            id: format!("iprule_{}", Utc::now().timestamp_millis()),
            tenant_id: tenant_id.map(ToString::to_string),
            rule_type: rule_type.to_string(),
            pattern: pattern.to_string(),
            description: description.to_string(),
            created_at: Utc::now().to_rfc3339(),
        }
    }
}

/// Dynamic IP Rule Registry shared across HTTP worker threads.
#[derive(Clone, Debug, Default)]
pub struct IpRuleRegistry {
    rules: Arc<RwLock<Vec<IpFilterRule>>>,
}

impl IpRuleRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rules: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Adds a new IP filtering rule.
    pub fn add_rule(&self, rule: IpFilterRule) {
        if let Ok(mut lock) = self.rules.write() {
            lock.push(rule);
        }
    }

    /// Removes an IP rule by ID.
    pub fn remove_rule(&self, rule_id: &str) -> bool {
        if let Ok(mut lock) = self.rules.write() {
            let initial_len = lock.len();
            lock.retain(|r| r.id != rule_id);
            return lock.len() < initial_len;
        }
        false
    }

    /// Lists all active IP rules, optionally filtered by tenant.
    #[must_use]
    pub fn list_rules(&self, tenant_id: Option<&str>) -> Vec<IpFilterRule> {
        let lock = match self.rules.read() {
            Ok(guard) => guard,
            Err(_) => return Vec::new(),
        };

        lock.iter()
            .filter(|r| match (&r.tenant_id, tenant_id) {
                (Some(t1), Some(t2)) => t1 == t2,
                (None, _) => true, // Global rules always apply
                (Some(_), None) => true,
            })
            .cloned()
            .collect()
    }

    /// Tests if a client IP is allowed.
    ///
    /// Logic:
    /// 1. If any matching "block" rule exists -> Denied.
    /// 2. If any "allow" rules exist for the tenant -> Allowed ONLY if matches at least one "allow" rule.
    /// 3. Otherwise -> Allowed by default.
    #[must_use]
    pub fn is_allowed(&self, client_ip: &str, tenant_id: Option<&str>) -> bool {
        let lock = match self.rules.read() {
            Ok(guard) => guard,
            Err(_) => return true, // Fail open on lock poisoning
        };

        let applicable: Vec<&IpFilterRule> = lock
            .iter()
            .filter(|r| match (&r.tenant_id, tenant_id) {
                (Some(t1), Some(t2)) => t1 == t2,
                (None, _) => true,
                _ => false,
            })
            .collect();

        // 1. Check explicit blocklist
        for rule in &applicable {
            if rule.rule_type == "block" && matches_ip_rule(client_ip, &rule.pattern) {
                return false;
            }
        }

        // 2. Check allowlist if configured
        let allow_rules: Vec<&&IpFilterRule> = applicable
            .iter()
            .filter(|r| r.rule_type == "allow")
            .collect();
        if !allow_rules.is_empty() {
            return allow_rules
                .iter()
                .any(|r| matches_ip_rule(client_ip, &r.pattern));
        }

        true
    }
}

/// IP Access Control Actix Web Middleware.
#[derive(Clone, Debug)]
pub struct IpAccessControl {
    registry: Arc<IpRuleRegistry>,
}

impl IpAccessControl {
    #[must_use]
    pub fn new(registry: Arc<IpRuleRegistry>) -> Self {
        Self { registry }
    }
}

impl<S, B> Transform<S, ServiceRequest> for IpAccessControl
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = ActixError;
    type InitError = ();
    type Transform = IpAccessControlService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(IpAccessControlService {
            service,
            registry: self.registry.clone(),
        })
    }
}

pub struct IpAccessControlService<S> {
    service: S,
    registry: Arc<IpRuleRegistry>,
}

impl<S, B> Service<ServiceRequest> for IpAccessControlService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = ActixError;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let client_ip = req
            .headers()
            .get("X-Forwarded-For")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
            .or_else(|| {
                req.connection_info()
                    .peer_addr()
                    .map(|p| p.split(':').next().unwrap_or(p).to_string())
            })
            .unwrap_or_else(|| "127.0.0.1".to_string());

        let tenant_header = req
            .headers()
            .get("X-Frappe-Tenant")
            .and_then(|h| h.to_str().ok())
            .map(ToString::to_string);

        if !self
            .registry
            .is_allowed(&client_ip, tenant_header.as_deref())
        {
            let res = HttpResponse::build(StatusCode::FORBIDDEN).json(serde_json::json!({
                "error": "Access Denied: Client IP is prohibited by security policy",
                "ip": client_ip,
            }));
            let (http_req, _) = req.into_parts();
            return Box::pin(ready(Ok(ServiceResponse::new(
                http_req,
                res.map_into_boxed_body(),
            ))));
        }

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res.map_into_boxed_body())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_registry_block_and_allow() {
        let registry = IpRuleRegistry::new();

        // Initially open
        assert!(registry.is_allowed("203.0.113.10", None));

        // Add block rule
        registry.add_rule(IpFilterRule::new(
            None,
            "block",
            "203.0.113.*",
            "Block abusive bot subnet",
        ));
        assert!(!registry.is_allowed("203.0.113.10", None));
        assert!(registry.is_allowed("192.168.1.50", None));

        // Add allow rule for tenant
        registry.add_rule(IpFilterRule::new(
            Some("acme"),
            "allow",
            "10.0.0.*",
            "Allow only internal VPN",
        ));
        assert!(registry.is_allowed("10.0.0.5", Some("acme")));
        assert!(!registry.is_allowed("192.168.1.50", Some("acme")));
    }
}
