# 📐 rwl-meta

> **DocType Schema Compiler, Dynamic Naming Series, Dynamic RBAC & AI Schemas for `rust-web-land` (`rwl`)**

`rwl-meta` is the foundational metadata engine that compiles declarative DocType definitions (fields, constraints, data types, indexes) into high-performance in-memory representations and zero-allocation runtime validation rules.

---

## ⚡ Core Capabilities

- **DocType Schema Compiler (`schema_compiler.rs`, `schema.rs`)**: Compiles and validates DocType schemas, field types, child tables, links, and indexes.
- **Dynamic RBAC & Permission Enforcement (`security_rules.rs`, `role_tools.rs`)**: Evaluates role-based and document-state permissions (Read, Write, Create, Submit, Cancel, Delete, Export, Report).
- **Autoname & Naming Series (`naming.rs`)**: High-performance naming series engine supporting format expressions (e.g. `ACC-.YYYY.-.#####`).
- **Graph & AI Schema Introspection (`graph_schema.rs`, `ai_schema.rs`)**: Automated graph node/edge projections and LLM function calling schema generation.
- **Bi-temporal Schema Migrations (`migration.rs`)**: Automated non-destructive migrations and state comparison across schema versions.

---

## 🛠 Usage Example

```rust
use rwl_meta::{DocType, Field, FieldType, DynamicDoc};

let doctype = DocType::new("Customer")
    .with_field(Field::new("customer_name", FieldType::Data).mandatory(true))
    .with_field(Field::new("credit_limit", FieldType::Currency));

assert!(doctype.validate().is_ok());
```

---

Part of the **[rust-web-land](file:///home/jrad/RustroverProjects/ERPNext_workspace/README.md)** platform.
