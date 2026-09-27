# frappe-storage

Content-addressable deduplicated drive storage engine and SurrealDB database abstractions.

---

## 📦 Overview

`frappe-storage` provides high-throughput persistent storage, multi-model database mappings, and deduplicated object drive capabilities.

### Key Capabilities

- **Content-Addressable Drive (`drive.rs`)**: SHA-256 chunk hashing, byte deduplication, MIME validation, and virtual directory hierarchy management.
- **SurrealDB Client Layer (`surreal.rs`, `lib.rs`)**: Multi-model database connection pooling, query builders, and document-graph mappings.

---

## 🛠 Usage Example

```rust
use frappe_storage::DeduplicatedStorage;

let storage = DeduplicatedStorage::new("./storage_root");
let file_hash = storage.store_file("report.pdf", b"%PDF-1.4...")?;
let bytes = storage.read_file(&file_hash)?;
```
