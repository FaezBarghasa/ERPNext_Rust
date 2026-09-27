# tenant

## Classs

- [ConnectionPoolManager](ConnectionPoolManager.md) — Dynamic Connection Pool Manager for multi-tenant database handles.
- [TenantError](TenantError.md) — Multi-tenant pool and routing errors.
- [TenantId](TenantId.md) — Unique Tenant Identifier.
- [TenantResolver](TenantResolver.md) — Actix Web Middleware for dynamic Tenant Resolution.
- [TenantResolverMiddleware](TenantResolverMiddleware.md)

## Functions

- [call](call.md)
- [call](call_1.md)
- [default](default.md)
- [default](default_1.md)
- [error_response](error_response.md)
- [error_response](error_response_1.md)
- [evict_idle_pools](evict_idle_pools.md) — Evicts idle pools exceeding the configured inactivity duration.
- [evict_idle_pools](evict_idle_pools_1.md) — Evicts idle pools exceeding the configured inactivity duration.
- [get_or_initialize_client](get_or_initialize_client.md) — Retrieves an active connection handle for the tenant, initializing if missing.
- [get_or_initialize_client](get_or_initialize_client_1.md) — Retrieves an active connection handle for the tenant, initializing if missing.
- [new](new.md) — Creates a new connection pool manager.
- [new](new_1.md) — Creates a new connection pool manager.
- [new_transform](new_transform.md)
- [new_transform](new_transform_1.md)
- [parse_tenant_id](parse_tenant_id.md) — Resolves tenant identity from request headers or host string.
- [poll_ready](poll_ready.md)
- [poll_ready](poll_ready_1.md)
- [provision_tenant](provision_tenant.md) — Cloud Multi-Tenant Provisioning Coordinator (Milestone 5.8).
- [spawn_maintenance_worker](spawn_maintenance_worker.md) — Starts a periodic background worker for evicting idle tenant connections.
- [spawn_maintenance_worker](spawn_maintenance_worker_1.md) — Starts a periodic background worker for evicting idle tenant connections.
- [status_code](status_code.md)
- [status_code](status_code_1.md)
- [validate_and_create_tenant_id](validate_and_create_tenant_id.md)
