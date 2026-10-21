# `rust-web-land` (`rwl`) — Universal Operator & User Guide

This guide explains how visual builders, business operators, warehouse personnel, factory floor workers, executives, and administrators interact with the `rust-web-land` (`rwl`) platform across desktop visual studios, mobile devices, and browser interfaces.

---

## 1. Visual App Builder Studio (`/builder`)

`rust-web-land` provides a visual drag-and-drop studio where anyone with **zero coding experience** can build complete web applications:

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                              VISUAL BUILDER STUDIO                                     │
│  ├── 1. Canvas Palette: Drag blocks (Containers, Headers, Virtual Grids, Charts, Forms) │
│  ├── 2. DocType Schema Modeler: Add fields, set types (Currency, Int, Date), mandatory │
│  ├── 3. Visual Logic Editor: Connect triggers, conditions, and actions with Rhai nodes │
│  ├── 4. Visual RBAC Matrix: Click checkboxes to assign view/edit/submit per role       │
│  └── 5. 1-Click Export & Deploy: Bundle to 1GB VPS package or push live update         │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

### 1.1 Visual Page Design
1. Launch the studio via `rwl studio --port 8080`.
2. Navigate to `http://127.0.0.1:8080/builder`.
3. Choose a page layout or drag components from the left drawer:
   - **Hero & Content Blocks**: Responsive typography, buttons, and media players.
   - **Data Grids**: Connect directly to any DocType to display live tables with sorting and search.
   - **Dynamic Forms**: Auto-generate complete entry forms directly from DocType schemas.
   - **Gantt & SPC Widgets**: Add industrial schedules and control charts with live bindings.

### 1.2 Schema & Form Builder
- Select **Schemas** in the builder sidebar.
- Click **+ New DocType** or edit an existing schema.
- Add fields with standard field types (`Data`, `Currency`, `Int`, `Float`, `Date`, `Select`, `Check`, `Text`).
- Mark fields as `Mandatory`, `Unique`, or set default expressions without writing SQL.

### 1.3 Visual Permissions & RBAC
- Open the **Permissions** tab.
- Assign granular permissions (`Read`, `Write`, `Create`, `Submit`, `Cancel`, `Delete`) across roles (`Customer`, `Staff`, `Manager`, `Admin`).

---

## 2. Role-Adaptive Multi-Persona Shells

`rust-web-land` dynamically projects specialized user interfaces based on the authenticated user's role (`TenantContext.user_session.roles`). Every persona shares the same underlying Rust binary while projecting an interaction paradigm tailored to the work context:

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
|  4. Manager / Lead   Executive triage queue, swipe approvals   Biometric auth,          Push alerts, cached |
|     (Approver)       (Swipe-R approve, Swipe-L reject), KPIs   Secure enclave signing   approval batch      |
|                                                                                                             |
|  5. Administrator    Cluster telemetry, database inspector,    Audit key signing,       Local emergency     |
|     (Executive)      dynamic schema synthesizer, Wasm manager  Syslog streaming         diagnostic shell    |
+─────────────────────────────────────────────────────────────────────────────────────────────────────────────+
```

---

## 3. Persona Walkthroughs

### 3.1 The Client / Customer Shell (`/portal`)
* **Target Audience:** B2C Shoppers, B2B Procurement Officers, Students, Streaming Subscribers.
* **Key Workflows:**
  - **Single-Click Checkout:** Instant guest-to-account conversion with balanced double-entry accounting.
  - **LMS Course Player:** Interactive syllabus tree with video player, synchronized notes, and inline quizzes.
  - **Typst Merkle Diplomas:** View and download cryptographically verified PDF/A graduation certificates with immutable SHA-256 Merkle root verification links.
  - **Video Streaming HUD:** Cinematic HLS media player with chapter scrubbers and natural language subtitle search.

### 3.2 The Warehouse & Field Worker Shell (`/worker`)
* **Target Audience:** Order Pickers, Forklift Operators, Couriers, Field Service Techs.
* **Key Invariants & Workflows:**
  - **$\ge 56\,\text{px}$ Touch Targets:** Massive buttons designed for single-handed use and heavy industrial gloves.
  - **Continuous Fast-Scan Mode:** Camera or hardware laser scanner stays active; scanning an SSCC-18 pallet barcode immediately advances the pick list with affirmative haptic feedback.
  - **Directed Pick Path HUD:** Visual aisle, bay, and shelf coordinates optimized using the Lin-Kernighan TSP heuristic.
  - **Offline Indicator:** Prominent banner confirming local SurrealKV storage state with active pending mutation counter (e.g. "14 scans pending cloud sync").

### 3.3 The Shopfloor MES Operator Shell (`/factory`)
* **Target Audience:** Machine Operators, CNC Machinists, Assembly Line Workers.
* **Key Invariants & Workflows:**
  - **Landscape Tablet View:** Designed for fixed machine work center mounts.
  - **Digital Job Traveler & Electronic Batch Record (EBR):** Step-by-step operational checklists with strict prerequisite sequencing.
  - **Dual-Witness Cryptographic Sign-Off:** High-assurance operations require secondary supervisor authentication before proceeding.
  - **Live SPC Control Charts:** Real-time Nelson-rule statistical process control charts alerting operators to 3-sigma excursions and trending drift.

### 3.4 The Executive Manager & Approver Shell (`/approvals`)
* **Target Audience:** Department Heads, CFOs, Project Managers.
* **Key Workflows:**
  - **Swipe-Driven Triage Deck:**
    - **Swipe Right:** Approve Purchase Order, Expense Claim, or Invoice with instant journal posting.
    - **Swipe Left:** Reject or request amendment with integrated voice-to-text dictation.
  - **Streaming KPI Tickers:** Real-time gross margin, cash balance, and AMR fleet utilization streams updating over WebSockets without page reload.

### 3.5 The System Administrator Cockpit (`/admin`)
* **Target Audience:** Systems Engineers, Site Reliability Engineers, DevOps.
* **Key Workflows:**
  - **Actix Worker Thread Gauges:** Real-time CPU, active connection, and channel backlog monitoring.
  - **SurrealDB Live Query Multiplexer:** Active client subscription inspector.
  - **Dynamic Schema Designer:** Create and compile new DocTypes into `SCHEMAFULL` SurrealQL tables at runtime.
  - **Wasmtime Fuel Fault Log:** Audit sandbox fuel exhaustion and memory isolation events.

---

## 4. Disconnected Operations & Edge Sync

When network connectivity is interrupted:
1. **Local Execution:** All POS checkout scans, stock ledger movements, and traveler sign-offs execute locally against the embedded SurrealKV store.
2. **Causal Vector Tracking:** Mutations are signed and recorded in a local causal vector clock $\vec{V}_{\text{device}}$.
3. **Automatic Cloud Reconnection:** When Wi-Fi or cellular connectivity is restored, `desk-app` streams the delta mutation batch to the Actix-web Live WebSocket actor.
4. **Deterministic Convergence:**
   - **Stock Balances:** Reconciled via commutative Positive-Negative (PN) Counters (zero lost picks/receipts).
   - **Document Records:** Reconciled via Last-Write-Wins (LWW) element sets with cryptographic tombstones.

---

## 5. Direct WooCommerce & WordPress Migration

Migrate legacy WordPress and WooCommerce stores into `rust-web-land` with high-throughput streaming ingestion:

```bash
# Ingest WooCommerce JSON or SQL dump into active tenant
curl -X POST http://127.0.0.1:8080/api/v1/trade/woocommerce-migrate \
  -H "X-Tenant-Id: tenant_store" \
  -H "Content-Type: application/json" \
  --data-binary @woocommerce_export.json
```

The ingestion engine transforms legacy relational rows directly into native `tab_item`, `tab_customer`, and `tab_sales_invoice` SurrealDB records at $\ge 10{,}000$ records in $\le 12$ seconds.

---

## 6. Enterprise Desk, Printable Formats & Payment Processing

### 6.1 The Unified Enterprise Desk (`/desk`)
The main administrative desk serves as the command center across all modules:
- **Instant OmniBar Filtering (`Ctrl+K`)**: Rapidly filter across modules, DocTypes, and operations in real-time.
- **Direct Workspace Shortcuts**: Launch directly into Sales Invoices, Purchase Orders, Items, Work Orders, and CRM Leads.
- **Multi-Persona Switcher**: Seamlessly switch between Enterprise Desk, Customer Portal, Warehouse Scanner, Shopfloor MES, Approval Deck, and System Admin Cockpit.

### 6.2 DocType List & Form Views
- **List View (`/desk/list/{doctype}`)**:
  - Live query filtering with search box.
  - Color-coded document status chips: `Draft` (Yellow), `Submitted` (Emerald), `Cancelled` (Crimson).
  - Quick action to create new records (`+ New {DocType}`).
- **Form View (`/desk/form/{doctype}/{id}`)**:
  - View and edit document attributes with strict decimal validation.
  - Lifecycle controls: **Save** (Draft), **Submit** (Lock against modifications), **Cancel** (Void), and **Delete**.
  - Direct breadcrumbs back to List View and Workspace.

### 6.3 Enterprise Printable Commercial Documents
Generate compliant commercial PDF/print outputs with `@media print` formatting:
```http
GET /api/v2/document/{doctype}/{name}/print
```

### 6.4 Pluggable Payments & Automated Reconciliation
Process card charges and handle provider webhooks with automated double-entry GL reconciliation:
```http
POST /api/v2/payments/charge
Content-Type: application/json

{
  "invoice_id": "ACC-SINV-2026-00001",
  "amount": "1500.00",
  "currency": "USD",
  "party_id": "CUST-00001",
  "payment_method": "credit_card",
  "capture": true
}
```

---

## 7. OTT Video Streaming, SSAI & FAST Linear Channels

`rust-web-land` provides a broadcast-grade media delivery experience:

* **Adaptive Bitrate HLS v6**: `GET /api/v2/streaming/hls/{media_id}/master.m3u8`
* **Server-Side Ad Insertion (SSAI)**: Frame-accurate SCTE-35 `#EXT-X-DATERANGE` cue markers.
* **FAST Linear 24/7 Channels**: XMLTV EPG schedule syndication at `GET /api/v2/streaming/fast/channels/{channel_id}/live.m3u8`.
* **Stremio v1 Addon Protocol**: Full catalog and stream discovery via `GET /stremio/v1/manifest.json`.

---

## 8. Master Audio Streaming CMS & Subsonic Ecosystem

* **Bit-Perfect Lossless FLAC**: Stream FLAC or transcode on demand at `GET /api/v2/audio/stream`.
* **10-Band Studio Graphic EQ**: Studio presets (`Flat`, `BassBoost`, `VocalAir`, `ClubDance`, `ClassicalHall`, `AcousticWarmth`) at `POST /api/v2/audio/eq`.
* **Synchronized Karaoke Lyrics**: Millisecond-accurate LRC scrolling at `GET /api/v2/audio/lyrics`.
* **Subsonic REST API v1.16+**: Connect any certified Subsonic client (Symfonium, DSub, Ultrasonic, Tempo) to `http://127.0.0.1:8080/rest/ping.view`.
