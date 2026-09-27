# frappe-net

Multi-tenant HTTP/WS routing, real-time LiveSync event streaming, and background job queue workers.

---

## 📦 Overview

`frappe-net` provides the network and transport layer for ERPNext Rust, powered by Actix-web and asynchronous broadcast channels.

### Key Capabilities

- **Tenant Resolution (`tenant.rs`, `middleware/`)**: Subdomain and header-based tenant resolution mapping to isolated database namespaces.
- **REST & RPC Routing (`routes.rs`, `server.rs`)**: High-throughput JSON endpoints for CRUD document lifecycles and metadata reflection.
- **LiveSync Channels (`live.rs`)**: WebSocket-based event broadcaster streaming real-time document mutations to connected clients.
- **Queue Workers (`queue.rs`)**: Async background task queues for asynchronous processing and batch operations.
