# `rustnext` — Universal Operator & User Guide

This guide explains how operators, warehouse personnel, factory floor workers, executives, and administrators interact with the `rustnext` platform across desktop, mobile, and edge environments.

---

## 1. Role-Adaptive Multi-Persona Shells

`rustnext` dynamically projects specialized user interfaces based on the authenticated user's role (`TenantContext.user_session.roles`). Every persona shares the same underlying Rust binary while projecting an interaction paradigm tailored to the work context:

```
+─────────────────────────────────────────────────────────────────────────────────────────────────────────────+
|                         UNIVERSAL ROLE-ADAPTIVE PROJECTION MATRIX                                           |
|                                                                                                             |
|  Role / Persona      Primary Interface Paradigm                Hardware Bindings        Offline Target      |
|  ──────────────────  ────────────────────────────────────────  ───────────────────────  ─────────────────── |
|  1. Client           Consumer-grade self-service, storefront,  WebAuthn, Card Token,    Browsing cache,     |
|     (Customer/Buyer) video HUD, LMS learning syllabus          Push Notifications       saved cart          |
|                                                                                                             |
|  2. Warehouse Worker Ruggedized high-contrast handheld view,   Continuous Barcode/RFID, 100% Offline Batch  |
|     (Picker/Packer)  massive touch targets (>= 56px), haptics   Vibration motor, BLE HU  Bin Pick Path Queue |
|                                                                                                             |
|  3. Shopfloor Worker Digital traveler card, step-by-step EBR,  Machine SCADA / Modbus,  Local shift buffer, |
|     (MES Operator)   dual-witness sign-offs, scrap counters    NFC badge sign-in        sensor replay       |
|                                                                                                             |
|  4. Manager / Lead   Executive triage queue, swipe approvals   Biometric biometric auth,Push alerts, cached |
|     (Approver)       (Swipe-R approve, Swipe-L reject), KPIs   Secure enclave signing   approval batch      |
|                                                                                                             |
|  5. Administrator    Cluster telemetry, database inspector,    Audit key signing,       Local emergency     |
|     (Executive)      dynamic schema synthesizer, Wasm manager  Syslog streaming         diagnostic shell    |
+─────────────────────────────────────────────────────────────────────────────────────────────────────────────+
```

---

## 2. Persona Walkthroughs

### 2.1 The Client / Customer Shell (`/portal`)
* **Target Audience:** B2C Shoppers, B2B Procurement Officers, Students, Streaming Subscribers.
* **Key Workflows:**
  - **Single-Click Checkout:** Instant guest-to-account conversion with balanced double-entry accounting.
  - **LMS Course Player:** Interactive syllabus tree with video player, synchronized notes, and inline quizzes.
  - **Typst Merkle Diplomas:** View and download cryptographically verified PDF/A graduation certificates with immutable SHA-256 Merkle root verification links.
  - **Video Streaming HUD:** Cinematic HLS media player with chapter scrubbers and 1536-dim natural language subtitle search.

---

### 2.2 The Warehouse & Field Worker Shell (`/worker`)
* **Target Audience:** Order Pickers, Forklift Operators, Couriers, Field Service Techs.
* **Key Invariants & Workflows:**
  - **$\ge 56\,\text{px}$ Touch Targets:** Massive buttons designed for single-handed use and heavy industrial gloves.
  - **Continuous Fast-Scan Mode:** Camera or hardware laser scanner stays active; scanning an SSCC-18 pallet barcode immediately advances the pick list with affirmative haptic feedback.
  - **Directed Pick Path HUD:** Visual aisle, bay, and shelf coordinates optimized using the Lin-Kernighan TSP heuristic.
  - **Offline Indicator:** Prominent banner confirming local SurrealKV storage state with active pending mutation counter (e.g. "14 scans pending cloud sync").

---

### 2.3 The Shopfloor MES Operator Shell (`/factory`)
* **Target Audience:** Machine Operators, CNC Machinists, Assembly Line Workers.
* **Key Invariants & Workflows:**
  - **Landscape Tablet View:** Designed for fixed machine work center mounts.
  - **Digital Job Traveler & Electronic Batch Record (EBR):** Step-by-step operational checklists with strict prerequisite sequencing.
  - **Dual-Witness Cryptographic Sign-Off:** High-assurance operations require secondary supervisor authentication before proceeding.
  - **Live SPC Control Charts:** Real-time Nelson-rule statistical process control charts alerting operators to 3-sigma excursions and trending drift.

---

### 2.4 The Executive Manager & Approver Shell (`/approvals`)
* **Target Audience:** Department Heads, CFOs, Project Managers.
* **Key Workflows:**
  - **Swipe-Driven Triage Deck:**
    - **Swipe Right:** Approve Purchase Order, Expense Claim, or Invoice with instant journal posting.
    - **Swipe Left:** Reject or request amendment with integrated voice-to-text dictation.
  - **Streaming KPI Tickers:** Real-time gross margin, cash balance, and AMR fleet utilization streams updating over WebSockets without page reload.

---

### 2.5 The System Administrator Cockpit (`/admin`)
* **Target Audience:** Systems Engineers, Site Reliability Engineers, DevOps.
* **Key Workflows:**
  - **Actix Worker Thread Gauges:** Real-time CPU, active connection, and channel backlog monitoring.
  - **SurrealDB Live Query Multiplexer:** Active client subscription inspector.
  - **Dynamic Schema Designer:** Create and compile new DocTypes into `SCHEMAFULL` SurrealQL tables at runtime.
  - **Wasmtime Fuel Fault Log:** Audit sandbox fuel exhaustion and memory isolation events.

---

## 3. Disconnected Operations & Edge Sync

When network connectivity is interrupted:
1. **Local Execution:** All POS checkout scans, stock ledger movements, and traveler sign-offs execute locally against the embedded SurrealKV store.
2. **Causal Vector Tracking:** Mutations are signed and recorded in a local causal vector clock $\vec{V}_{\text{device}}$.
3. **Automatic Cloud Reconnection:** When Wi-Fi or cellular connectivity is restored, `desk-app` streams the delta mutation batch to the Actix-web Live WebSocket actor.
4. **Deterministic Convergence:**
   - **Stock Balances:** Reconciled via commutative Positive-Negative (PN) Counters (zero lost picks/receipts).
   - **Document Records:** Reconciled via Last-Write-Wins (LWW) element sets with cryptographic tombstones.

---

## 4. Direct WooCommerce & WordPress Migration

Migrate legacy WordPress and WooCommerce stores into `rustnext` with high-throughput streaming ingestion:

```bash
# Ingest WooCommerce JSON or SQL dump into active tenant
curl -X POST http://127.0.0.1:8080/api/v1/trade/woocommerce-migrate \
  -H "X-Tenant-Id: tenant_store" \
  -H "Content-Type: application/json" \
  --data-binary @woocommerce_export.json
```

The ingestion engine transforms legacy relational rows directly into native `tab_item`, `tab_customer`, and `tab_sales_invoice` SurrealDB records at $\ge 10{,}000$ records in $\le 12$ seconds.
