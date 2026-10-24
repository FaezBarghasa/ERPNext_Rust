# 💾 rwl-storage

> **SurrealKV Storage Substrate, Content-Addressable Deduplicated Drive & CRDT Multi-Node Sync for `rust-web-land` (`rwl`)**

`rwl-storage` provides high-throughput persistent storage, multi-model SurrealKV database mappings, CRDT mesh synchronization, and deduplicated object drive capabilities for `rust-web-land` (`rwl`).

---

## ⚡ Core Capabilities

- **SurrealDB & Embedded SurrealKV (`surreal.rs`)**:
  - Connection pool management (`ConnectionPoolManager`) with configurable write buffer and cache limits.
  - Multi-tenant namespace and database partitioning with tenant isolation guarantees.
- **Content-Addressable Deduplicated Storage (`drive.rs`)**:
  - SHA-256 chunk hashing, byte deduplication, MIME validation, and virtual directory hierarchy management.
- **CRDT & Conflict-Free Replication (`crdt.rs`, `merkle.rs`)**:
  - Merkle-tree state reconciliation for offline-first edge nodes and multi-datacenter clusters.
- **Field-Level Encryption (`encryption.rs`)**:
  - Hardware-accelerated ChaCha20-Poly1305 encryption for sensitive enterprise credentials and secrets.

---

## 🛠 Usage Example

```rust
use rwl_storage::ConnectionPoolManager;

let pool = ConnectionPoolManager::new_in_memory().await.unwrap();
let count = pool.count_documents("Customer").await.unwrap();
assert_eq!(count, 0);
```

---

Part of the **[rust-web-land](file:///home/jrad/RustroverProjects/ERPNext_workspace/README.md)** platform.
