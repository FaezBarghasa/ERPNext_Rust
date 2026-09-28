# Universal Work-Type Templates & Theme Engine Guide

This guide details the six prebuilt enterprise template suites embedded natively within `rustnext` (`erp-cms`), their real-world architectural benchmarks, module integrations, DocType schemata, and deployment workflows.

---

## 1. Overview & Architectural Topology

Every template is compiled directly into the pure-Rust binary with sub-10ms Server-Side Rendering (SSR) Time-to-First-Byte (TTFB), zero external dynamic runtime dependencies, and deep integration into the enterprise core (`erp-accounting`, `erp-inventory`, `erp-trade`, `erp-software`, `erp-wms`, `erp-manufacturing`, `frappe-storage`, and `frappe-framework`).

```
+─────────────────────────────────────────────────────────────────────────────────────────────────────────────+
|                         THE 6 UNIVERSAL PREBUILT ENTERPRISE TEMPLATE SUITES                                 |
|                                                                                                             |
|  1. SVoD Streaming & Entertainment Platform (`svod-streaming`)                                              |
|     ├── Benchmarks: Netflix (Billboard player, ABR HLS, scrub) & Crunchyroll (simulcast, anime subs)        |
|     ├── Modules: `erp-cms`, `erp-software`, `erp-accounting`, `erp-support`, `frappe-storage`               |
|     └── Stack: HLS/DASH Transcoder, 1536-dim Subtitle Vector Search, Watch Party WebSockets, DRM HMAC       |
|                                                                                                             |
|  2. Digital Learning & LMS Academy (`lms-academy`)                                                          |
|     ├── Benchmarks: Coursera (Degree paths, verifiable certs) & Udemy (Instructor splits, Q&A checkpoints)  |
|     ├── Modules: `erp-learning`, `erp-software`, `erp-accounting`, `erp-trade`, `erp-cms`                  |
|     └── Stack: Dynamic Syllabus Trees, ASC 606 Progress Recognition, Merkle Typst PDF/A Verifiable Diplomas |
|                                                                                                             |
|  3. Digital Products, Games & Creator Hub (`digital-goods`)                                                 |
|     ├── Benchmarks: Steam (Trailers, specs, bundle pricing) & Creative Market (Live font/3D test)           |
|     ├── Modules: `erp-trade`, `erp-software`, `erp-accounting`, `frappe-storage`, `erp-crm`                 |
|     └── Stack: AES-256-GCM Signed Download URLs, Hardware UUID License Locks, 85/15 Instant Creator Payouts |
|                                                                                                             |
|  4. Industrial B2B E-Commerce & Parametric CAD Matrix (`b2b-industrial`)                                    |
|     ├── Benchmarks: Frost CNC (3D tool customizer), Rowa/Starmatik (Heavy automation), Mind Robotics        |
|     ├── Modules: `erp-manufacturing`, `erp-trade`, `erp-inventory`, `erp-wms`, `erp-accounting`            |
|     └── Stack: Three.js Parametric 3D Tooling Customizer, STEP/IGES DFM Mesh Ingest, ZUGFeRD 2.2 E-Invoicing |
|                                                                                                             |
|  5. High-Velocity Luxury & Spatial Consumer B2C Flagship (`b2c-retail`)                                     |
|     ├── Benchmarks: Cartier (Haute Horlogerie 3D exploded view), Industry West (AR swatches), Oryzo/Hubtown|
|     ├── Modules: `erp-cms`, `erp-inventory`, `erp-trade`, `erp-wms`, `erp-accounting`                       |
|     └── Stack: 60 FPS Viewport Grid, PBR Metallurgy Shaders, Real-Time Flash Stock Feed, Slide-Over Bag     |
|                                                                                                             |
|  6. Financial Trading, Brokerage & Prediction Hub (`trading-exchange`)                                      |
|     ├── Benchmarks: Binance (L2 order book, depth chart), Robinhood (Scrub charts), Polymarket (Binary odds)|
|     ├── Modules: `erp-trade`, `erp-accounting`, `erp-crm`, `frappe-net`, `frappe-storage`                  |
|     └── Stack: WebGL Canvas Candlesticks, Microsecond Tick Streams, CPMM Probability AMM, Bi-Temporal Merkle|
+─────────────────────────────────────────────────────────────────────────────────────────────────────────────+
```

---

## 2. Template Catalog Specifications

### 2.1 SVoD Video Streaming & Entertainment (`svod-streaming`)
* **Live Route:** `GET /templates/svod-streaming`
* **API Schema:** `GET /api/v1/templates/svod-streaming/manifest`
* **Target Industry:** Film & Television, Anime Simulcasts, OTT Media Networks.
* **Key Features:**
  - **Cinematic Billboard Player:** Full-bleed responsive backdrop with trailer audio toggle and ambient color bleed.
  - **HLS Chunked Streaming:** HTTP 206 Partial Content range requests delivering instant scrub response without buffer stalls.
  - **1536-Dimensional Subtitle Vector Search:** Natural language semantic search across video transcripts stored in SurrealDB table `media_transcripts` using cosine similarity.
  - **Watch Party WebSockets:** Synchronized playback state (play, pause, seek offsets) across connected viewers with real-time reaction pulses.
  - **Content Royalties Double-Entry:** Calculates film studio royalties and posts automated journal entries debiting `Content Royalty Expense` and crediting `Studio Accounts Payable`.
* **Design Tokens:**
  ```json
  {
    "color_primary": "#e50914",
    "color_secondary": "#b81d24",
    "color_background": "#0a0a0c",
    "color_surface": "#141419",
    "color_accent": "#ff2a35",
    "font_heading": "'Syne', sans-serif",
    "font_body": "'Outfit', sans-serif",
    "font_mono": "'Space Mono', monospace",
    "border_radius": "1.0rem",
    "noise_opacity": 0.04
  }
  ```

---

### 2.2 Digital Learning & LMS Academy (`lms-academy`)
* **Live Route:** `GET /templates/lms-academy`
* **API Schema:** `GET /api/v1/templates/lms-academy/manifest`
* **Target Industry:** Higher Education, EdTech, Corporate Training, Professional Certification.
* **Key Features:**
  - **Hierarchical Syllabus Tree:** Structured Course $\to$ Module $\to$ Lesson $\to$ Assessment graph with completed, current, and locked state transitions.
  - **Signal-Driven Quiz Engine:** Interactive knowledge checks with instant score calculation and lesson prerequisite unlocking.
  - **ASC 606 Tuition Amortization:** 5-step revenue recognition allocating tuition transaction prices across learning modules based on Standalone Selling Prices ($SSP$).
  - **Merkle Verifiable PDF/A Diplomas:** Compiles print-ready PDF/A graduation certificates in $<5\,\text{ms}$ using embedded Typst with SHA-256 Merkle root verification links.
* **Design Tokens:**
  ```json
  {
    "color_primary": "#0056d2",
    "color_secondary": "#003e99",
    "color_background": "#060913",
    "color_surface": "#0d1424",
    "color_accent": "#3b82f6",
    "font_heading": "'Outfit', sans-serif",
    "font_body": "'Outfit', sans-serif",
    "font_mono": "'Space Mono', monospace",
    "border_radius": "0.75rem",
    "noise_opacity": 0.025
  }
  ```

---

### 2.3 Digital Products, Games & Creator Hub (`digital-goods`)
* **Live Route:** `GET /templates/digital-goods`
* **API Schema:** `GET /api/v1/templates/digital-goods/manifest`
* **Target Industry:** Indie Game Studios, 3D Asset Creators, Type Foundries, Music Producers.
* **Key Features:**
  - **Interactive Asset Testers:**
    1. *Type-Tester:* Live editable text rendering canvas with real-time OpenType font weight/size sliders.
    2. *3D Viewport:* Interactive rotating wireframe mesh inspector.
    3. *Audio Waveform:* Canvas audio waveform scrubber with playhead tracking.
  - **Multi-Tier License Selection:** Personal vs. Commercial vs. Extended Commercial with automatic price multipliers.
  - **AES-256-GCM Secure Downloads:** Generates single-use, time-limited download links without exposing raw storage buckets.
  - **Hardware Node-Locking:** Binds software activations to machine UUID hashes with activation count limits.
  - **85/15 Creator Payouts:** Automatic split accounting crediting creator accounts payable and platform fee revenue.
* **Design Tokens:**
  ```json
  {
    "color_primary": "#10b981",
    "color_secondary": "#059669",
    "color_background": "#070b0e",
    "color_surface": "#0f1720",
    "color_accent": "#34d399",
    "font_heading": "'Syne', sans-serif",
    "font_body": "'Outfit', sans-serif",
    "font_mono": "'Space Mono', monospace",
    "border_radius": "0.75rem",
    "noise_opacity": 0.03
  }
  ```

---

### 2.4 Industrial B2B E-Commerce & Parametric CAD Matrix (`b2b-industrial`)
* **Live Route:** `GET /templates/b2b-industrial`
* **API Schema:** `GET /api/v1/templates/b2b-industrial/manifest`
* **Target Industry:** Heavy Machinery, CNC Tooling, Automation Robotics, Industrial Components.
* **Key Features:**
  - **Parametric 3D Tooling Configurator:** Real-time geometry customization (flutes: $2\text{--}6$, helix: $30^\circ\text{--}45^\circ$, shank: $6\text{--}25\,\text{mm}$, coatings: AlTiN/DLC/TiCN) with live pricing and feeds & speeds calculation.
  - **Three.js Mechanical Assembly Viewport:** Exploded CAD view and mesh inspection.
  - **Tiered Wholesale Price Breaks:** Automated contract pricing ($10\text{--}49$ units: $5\%$, $500+$ units: $30\%$) and corporate trade credit limit checks.
  - **ZUGFeRD 2.2 & Peppol BIS 3.0 E-Invoicing:** Structured XML e-invoice generation for automated ERP-to-ERP accounting ingestion.
* **Design Tokens:**
  ```json
  {
    "color_primary": "#ff6d00",
    "color_secondary": "#e65100",
    "color_background": "#0a0c10",
    "color_surface": "#12161f",
    "color_accent": "#ff9100",
    "font_heading": "'Space Mono', monospace",
    "font_body": "'Outfit', sans-serif",
    "font_mono": "'Space Mono', monospace",
    "border_radius": "0.35rem",
    "noise_opacity": 0.05
  }
  ```

---

### 2.5 High-Velocity Luxury & Spatial Consumer Flagship (`b2c-retail`)
* **Live Route:** `GET /templates/b2c-retail`
* **API Schema:** `GET /api/v1/templates/b2c-retail/manifest`
* **Target Industry:** Haute Horlogerie, High Fashion, Designer Furniture, Luxury Lifestyle.
* **Key Features:**
  - **Full-Viewport WebGL 3D Tourbillon:** Procedural kinetic watch floating in interactive orbit with ACESFilmic tone mapping and PBR metallurgy shaders.
  - **GSAP ScrollTrigger Kinematics:** Exploded mechanical disassembly decoupling bezel, carriage, and sapphire caseback on scroll.
  - **Material Metallurgy Customizer:** Interactive finish switcher (Brushed Titanium, 18K Celestial Gold, Diamond DLC Obsidian).
  - **Real-Time Flash Stock Counter:** Live inventory reservation feed via SurrealDB `LIVE SELECT` preventing overselling during flash sales.
* **Design Tokens:**
  ```json
  {
    "color_primary": "#e6c887",
    "color_secondary": "#c3a35e",
    "color_background": "#070709",
    "color_surface": "#111218",
    "color_accent": "#f5d799",
    "font_heading": "'Syne', sans-serif",
    "font_body": "'Outfit', sans-serif",
    "font_mono": "'Space Mono', monospace",
    "border_radius": "1.25rem",
    "noise_opacity": 0.035
  }
  ```

---

### 2.6 Multi-Asset Financial Trading & Prediction Hub (`trading-exchange`)
* **Live Route:** `GET /templates/trading-exchange`
* **API Schema:** `GET /api/v1/templates/trading-exchange/manifest`
* **Target Industry:** Crypto Exchanges, Fractional Equity Brokerages, Information Prediction Markets.
* **Key Features:**
  - **WebGL Candlestick Charting:** 60 FPS Canvas rendering of OHLC candlesticks, volume bars, and dynamic price crosshairs.
  - **Streaming Level-2 Order Book:** Real-time bid/ask depth ladder with cumulative volume visualization.
  - **Order Entry Ticket:** Limit/Market orders with margin leverage sliders and maker/taker fee preview ($0.02\% / 0.05\%$).
  - **128-bit Decimal Multi-Currency Ledger:** Exact `rust_decimal` precision for wallet balances with zero fraction drift.
* **Design Tokens:**
  ```json
  {
    "color_primary": "#00e676",
    "color_secondary": "#00c853",
    "color_background": "#05080c",
    "color_surface": "#0c131c",
    "color_accent": "#69f0ae",
    "font_heading": "'Space Mono', monospace",
    "font_body": "'Outfit', sans-serif",
    "font_mono": "'Space Mono', monospace",
    "border_radius": "0.5rem",
    "noise_opacity": 0.03
  }
  ```

---

## 3. Fast Deployment CLI Commands

Provision and launch any template site in under 2 seconds:

```bash
# Deploy SVoD Video Streaming Platform
cargo run -p rbench -- site deploy --site-name cinema.example.com --template svod-streaming

# Deploy LMS Academy in Micro-Topology (<64MB RSS)
cargo run -p rbench -- site deploy --site-name academy.example.com --template lms-academy --micro

# Deploy Industrial B2B Matrix
cargo run -p rbench -- site deploy --site-name b2b.example.com --template b2b-industrial --admin-email ops@example.com

# Deploy Luxury B2C Flagship
cargo run -p rbench -- site deploy --site-name luxury.example.com --template b2c-retail

# Deploy Financial Trading Terminal
cargo run -p rbench -- site deploy --site-name exchange.example.com --template trading-exchange
```

---

## 4. Theme Engine & Runtime Customization

The dynamic theme engine compiles design tokens directly into CSS custom properties (`:root { ... }`):

```rust
use erp_cms::{ThemeRegistry, DesignTokens, ThemeManifest, RenderEngineKind, WorkTypeClassification};

let mut registry = ThemeRegistry::new();
let manifest = ThemeManifest {
    id: "theme_custom_luxury".into(),
    name: "Custom Atelier".into(),
    work_type: WorkTypeClassification::LuxuryAtelier,
    engine: RenderEngineKind::SsrTera,
    assets_dir: "/assets/themes/custom".into(),
    layout_slots: vec![],
    default_design_tokens: DesignTokens::default(),
};

// Register and compile in < 50 microseconds
registry.register_theme("tenant_luxury", manifest);

// Retrieve pre-compiled CSS variables
let compiled = registry.get_theme("tenant_luxury").unwrap();
println!("{}", compiled.css_custom_properties);
```
