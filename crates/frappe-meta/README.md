# frappe-meta

DocType schema compiler, dynamic naming series, RBAC permission validator, and declarative migration runner for ERPNext Rust.

---

## 📦 Overview

`frappe-meta` is the foundational metadata engine that compiles declarative DocType definitions (fields, constraints, data types) into memory representations and runtime validation rules.

### Key Capabilities

- **DocType Schema Compiler (`schema_compiler.rs`, `schema.rs`)**: Parses and validates DocType schemas, field definitions, indexes, and relations.
- **RBAC Engine (`rbac.rs`)**: Evaluates role-based and document-state permissions (Read, Write, Create, Submit, Cancel, Delete, Report).
- **Dynamic Naming Series (`naming.rs`)**: Thread-safe autoname generator supporting expression patterns (e.g. `ACC-.YYYY.-.#####`).
- **Migration Runner (`migration.rs`)**: Compares schema states and runs non-destructive, backwards-compatible data migrations.

---

## 🛠 Usage Example

```rust
use frappe_meta::{DocType, Field, FieldType, RbacEngine, RolePermission};

// Define DocType schema
let doctype = DocType::new("Customer")
    .with_field(Field::new("customer_name", FieldType::Data).mandatory(true))
    .with_field(Field::new("credit_limit", FieldType::Currency));

// Check RBAC permissions
let rbac = RbacEngine::new();
assert!(rbac.can_read(&user_roles, &doctype));
```
