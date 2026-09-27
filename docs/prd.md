# Product Requirements Document (PRD)

## 1. Executive Summary

ERPNext Rust is an enterprise-grade ERP platform re-engineered from the ground up in Rust. It aims to deliver microsecond-level transaction speeds, strict memory safety, seamless multi-tenancy, and modular business domains capable of scaling from local single-node edge devices to distributed cloud clusters.

---

## 2. Key Objectives & Goals

- **Performance**: Sub-10ms p99 latency on core transaction APIs (Sales Invoices, Journal Entries, Stock Movements).
- **Safety & Precision**: Absolute elimination of data inconsistency in financial and inventory records using immutable ledgers and exact-precision numerical arithmetic.
- **Modularity**: Domain crates remain cleanly decoupled and independently testable without circular dependencies.
- **Extensibility**: Sandboxed plugin architecture allowing custom user scripting without compromising system stability.
- **Multi-Tenant Native**: Native support for thousands of tenants per single binary instance with isolated databases and storage.

---

## 3. Core Functional Requirements

### 3.1 Metadata & Schema System (`frappe-meta`)
- **FR-META-01**: Must dynamically compile JSON/declarative DocType schemas into memory structures.
- **FR-META-02**: Must enforce RBAC matrix permissions for roles, document states, and individual field permissions.
- **FR-META-03**: Must support custom sequential and formatted naming series.
- **FR-META-04**: Must provide automated migration validation between schema versions.

### 3.2 Framework & Runtime (`frappe-framework`)
- **FR-FRAME-01**: Must enforce rigid document lifecycle state machines (`Draft` → `Submitted` → `Cancelled`).
- **FR-FRAME-02**: Must execute safe, sandboxed custom scripts via Rhai and WASI Wasmtime plugins.
- **FR-FRAME-03**: Must maintain comprehensive audit trails and change diffs on every record mutation.

### 3.3 Financial Accounting (`erp-accounting`)
- **FR-ACC-01**: Must guarantee zero-sum debit/credit balance across all General Ledger entries.
- **FR-ACC-02**: Must support multi-currency accounting with historical exchange rate lookup.
- **FR-ACC-03**: Must provide automated straight-line and reducing-balance asset depreciation schedules.
- **FR-ACC-04**: Must track Accounts Receivable / Accounts Payable aging schedules accurately.

### 3.4 Inventory & Stock (`erp-inventory`)
- **FR-INV-01**: Must track real-time bin quantities across multi-level hierarchical warehouses.
- **FR-INV-02**: Must support exact FIFO queue valuation and moving-average valuation methods.
- **FR-INV-03**: Must enforce batch and serial number tracking with expiration validation.

### 3.5 Manufacturing & Production (`erp-manufacturing`)
- **FR-MFG-01**: Must support multi-level Bill of Materials (BOM) explosion with scrap calculations.
- **FR-MFG-02**: Must generate and track Work Orders and operational workstation routing.
- **FR-MFG-03**: Must calculate Material Requirement Planning (MRP) demand forecasts.

### 3.6 Trade & Logistics (`erp-trade`)
- **FR-TRD-01**: Must support multi-tier discount rules, pricing rules, and item price matrices.
- **FR-TRD-02**: Must allocate landed costs (shipping, customs) proportionally across received items.
- **FR-TRD-03**: Must calculate cascading and inclusive tax templates.

### 3.7 Human Resources & Payroll (`erp-hr`)
- **FR-HR-01**: Must maintain employee records, departmental hierarchies, and attendance logs.
- **FR-HR-02**: Must compute salary structures, taxable earnings, statutory deductions, and generate payroll batches.

### 3.8 Customer Relationship Management (`erp-crm`)
- **FR-CRM-01**: Must provide lead intake pipelines, stages, and weighted conversion scoring.
- **FR-CRM-02**: Must track opportunities, customer communications, and deal value metrics.

### 3.9 Specialized Industry Modules
- **Lending (`erp-lending`)**: Fixed and floating loan disbursements, interest compounding, and amortization table generation.
- **Support (`erp-support`)**: Ticket SLA timers, resolution targets, priority matrices, and support gameplans.
- **Education (`erp-learning`)**: Course curriculums, lesson tracking, enrollment, and automated quiz scoring.
- **CMS & Media (`erp-cms`)**: Secure media transcode queues, WebVTT/SRT subtitle processing, and HMAC-signed media delivery URLs.

---

## 4. Non-Functional Requirements

- **Reliability**: 99.99% uptime with zero data loss guarantee.
- **Security**: Strict token-based authentication (PASETO/JWT), HMAC URL signing for media, constant-time token verification, and strict input validation.
- **Observability**: Structured metrics and logging via `tracing` with OpenTelemetry tracing capability.
