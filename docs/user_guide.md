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

---

## 5. Enterprise Desk, Printable Formats & Payment Processing

### 5.1 The Unified Enterprise Desk (`/desk`)
The main administrative desk serves as the command center across all 11 core modules:
- **Instant OmniBar Filtering (`Ctrl+K`)**: Rapidly filter across modules, DocTypes, and operations in real-time.
- **Direct Workspace Shortcuts**: Launch directly into Sales Invoices, Purchase Orders, Items, Work Orders, and CRM Leads.
- **Multi-Persona Switcher**: Seamlessly switch between Enterprise Desk, Customer Portal, Warehouse Scanner, Shopfloor MES, Approval Deck, and System Admin Cockpit.

### 5.2 DocType List & Form Views
- **List View (`/desk/list/{doctype}`)**:
  - Live query filtering with search box.
  - Color-coded document status chips: `Draft` (Yellow), `Submitted` (Emerald), `Cancelled` (Crimson).
  - Quick action to create new records (`+ New {DocType}`).
- **Form View (`/desk/form/{doctype}/{id}`)**:
  - View and edit document attributes with strict decimal validation.
  - Lifecycle controls: **Save** (Draft), **Submit** (Lock against modifications), **Cancel** (Void), and **Delete**.
  - Direct breadcrumbs back to List View and Workspace.

### 5.3 Enterprise Printable Commercial Documents
Generate compliant commercial PDF/print outputs with `@media print` formatting:
- **Print View Route**:
  ```http
  GET /api/v2/document/{doctype}/{name}/print
  ```
- **Direct HTML Generation Route**:
  ```http
  POST /api/v2/print/document
  Content-Type: application/json

  {
    "doctype": "Sales Invoice",
    "name": "ACC-SINV-2026-00001",
    "customer": "Acme Corp",
    "total": "1500.00",
    "items": [
      { "item_code": "CONS-001", "qty": "10", "rate": "150.00", "amount": "1500.00" }
    ]
  }
  ```
- Includes official letterhead, multi-rate tax/VAT tranches, itemized tables, bank payment instructions (IBAN, BIC/SWIFT), and authorized signature areas.

### 5.4 Pluggable Payments & Automated Reconciliation
Process card charges and handle provider webhooks with automated double-entry GL reconciliation:
- **Process Payment Charge**:
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
- **Handle Payment Webhook**:
  ```http
  POST /api/v2/payments/webhook
  Content-Type: application/json

  {
    "event_type": "payment.succeeded",
    "intent_id": "PI-ACC-SINV-2026-00001",
    "amount": "1500.00",
    "reference": "CAP_MOCK_12345"
  }
  ```
- On successful payment capture, the `PaymentReconciliationEngine` automatically generates a balanced Journal Entry clearing Accounts Receivable and posting to the designated Bank/Clearing Ledger.

---

## 6. OTT Video Streaming, SSAI & FAST Linear Channels

`rustnext` provides a media delivery experience with broadcast-grade features:

### 6.1 Multi-Bitrate HLS v6 Streaming
- **Master Manifest**: Access adaptive bitrate video streams via RFC 8216 master playlist:
  ```http
  GET /api/v2/stream/hls/master.m3u8?content_id={id}
  ```
- **Automatic Resolution Switching**: Players seamlessly switch between 1080p, 720p, and 480p streams based on available network bandwidth.

### 6.2 Server-Side Ad Insertion (SSAI) with SCTE-35
- Requesting sub-stream playlists with `cue_scte35=true` enables frame-accurate `#EXT-X-DATERANGE` markers:
  ```http
  GET /api/v2/stream/hls/1080p.m3u8?content_id={id}&cue_scte35=true
  ```
- Ad splices occur on exact keyframe boundaries without client player rebuffering.

### 6.3 24/7 Linear FAST Channels & XMLTV EPG
- **Linear Stream Playhead**: Continuous virtual broadcast channel simulating linear cable/satellite television.
- **XMLTV EPG Syndication**: Electronic Program Guide accessible by standard IPTV and media center applications:
  ```http
  GET /api/v2/stream/fast/epg.xml
  ```

### 6.4 Stremio v1 Addon Discovery
- Install into any Stremio player via the community addon URL:
  ```http
  GET /stremio/v1/manifest.json
  ```
- Enables catalog browsing, stream playback, and metadata discovery directly inside the Stremio ecosystem.

---

## 7. Master Audio Streaming CMS & Subsonic Ecosystem

Experience studio-grade audio delivery, acoustic transcription suggestions, and full Subsonic client interoperability:

### 7.1 Lossless Audio Streaming & FLAC Compression
- Stream bit-perfect FLAC audio or compressed streams on demand:
  ```http
  GET /api/v2/audio/stream?track_id={id}&format=flac
  ```
- Ingested audio uploads are automatically analyzed and compressed to FLAC with $\ge 40\%$ lossless bandwidth reduction.

### 7.2 10-Band Graphic Studio Equalizer
- Apply studio EQ presets or custom band gains directly to the DSP pipeline:
  ```http
  POST /api/v2/audio/eq
  Content-Type: application/json

  {
    "preset": "AcousticWarmth",
    "gains_db": [2.0, 1.5, 0.5, 0.0, 0.0, 0.5, 1.0, 1.5, 2.0, 1.0]
  }
  ```
- **Built-in Presets:** `Flat`, `BassBoost`, `VocalAir`, `ClubDance`, `ClassicalHall`, `AcousticWarmth`.

### 7.3 Synchronized Karaoke LRC Lyrics
- Millisecond-accurate word/line timing for real-time karaoke display:
  ```http
  GET /api/v2/audio/lyrics?track_id={id}
  ```
- Supports byte-offset tracking, time synchronization (`[mm:ss.xx]`), and dual-language translation tags.

### 7.4 Acoustic Transcription Music Recommendations
- Suggests new tracks by analyzing real acoustic transcription features (spectral centroid, cadence, tempo, key harmonic changes), avoiding the echo-chamber pitfalls of simple text metadata tags:
  ```http
  GET /api/v2/audio/suggest?track_id={id}&limit=10
  ```

### 7.5 Procedural Sleep Sounds & Binaural Beats
- Synthesize uninterrupted sleep noise directly on the edge:
  ```http
  GET /api/v2/audio/sleep-noise?color=pink&duration_secs=3600&binaural_hz=4.0
  ```
- **Noise Profiles:** `White`, `Pink` ($1/f$ pink noise for sleep), `Brown` ($1/f^2$ brown noise for deep relaxation).
- **Binaural Wave Entrainment:** Delta ($0.5-4\,\text{Hz}$), Theta ($4-8\,\text{Hz}$), Alpha ($8-12\,\text{Hz}$).

### 7.6 Live Radio Directory & Multi-Client Audiobook Sync
- **SHOUTcast/Icecast Streaming:** Browse curated live global radio stations with real-time stream metadata extraction.
- **Audiobook Bookmarks:** Sync playback position down to the second across multiple devices with configurable sleep timers and automatic rewind on resumption.

### 7.7 Native Subsonic Client Interoperability
Connect any certified Subsonic client (Symfonium, DSub, Ultrasonic, Tempo, Jamstash) to your `rustnext` instance:
1. Open client account settings and choose **Subsonic** protocol.
2. Enter server URL: `https://your-domain.com`.
3. Provide your `rustnext` user credentials.
4. Enjoy instant streaming, offline sync, and playlist management across mobile and desktop.

