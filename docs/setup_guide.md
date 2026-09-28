# `rustnext` — Developer & System Administrator Setup Guide

This document provides step-by-step instructions to install, build, configure, provision, and deploy `rustnext` across development workstations, constrained edge gateways, and planetary cloud clusters.

---

## 1. System Prerequisites

* **Rust Toolchain:** Rust 1.85.0+ (2024 Edition) with `cargo` and `rustfmt`:
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  rustup update stable
  ```
* **Host Platform:** Linux (Pop!_OS / Ubuntu / Debian / Fedora / Arch), macOS, or Windows (WSL2).
* **Optional Mobile Toolchains:**
  - **Android NDK:** API Level 26+ (for native `.apk` compilation via `cargo-apk`).
  - **Node/Pnpm:** For building static WebAssembly assets (optional; zero external runtime dependencies are required for production SSR).

---

## 2. Quickstart in 3 Minutes

### Step 1: Clone and Verify Build
```bash
git clone https://github.com/FaezBarghasa/ERPNext_Rust.git
cd ERPNext_Rust

# Verify compilation across all 21 crates
cargo check --workspace
```

### Step 2: Provision a Tenant Site via `rbench`
Deploy any of the prebuilt enterprise template suites in under 2 seconds:
```bash
# Provision the SVoD Video Streaming Platform for demo.rustnext.org
cargo run -p rbench -- site deploy \
  --site-name demo.rustnext.org \
  --template svod-streaming \
  --admin-email admin@enterprise.local
```

### Step 3: Launch the Production Web Server
```bash
# Launch frappe-net HTTP/1.1, HTTP/2, HTTP/3, and WebSocket server
cargo run -p frappe-net
```
* **Universal Template Portal:** `http://127.0.0.1:8080/templates`
* **Direct Template Site:** `http://127.0.0.1:8080/templates/svod-streaming`
* **Health Check:** `http://127.0.0.1:8080/health`
* **Template Catalog JSON API:** `http://127.0.0.1:8080/api/v1/templates`

---

## 3. Micro-Topology Deployment (<64MB RSS)

For edge POS terminals, field kiosks, and single-board computers (Raspberry Pi 4/5, STM32MP1):

```bash
# Provision tenant in constrained memory mode
cargo run -p rbench -- site deploy \
  --site-name pos-edge.local \
  --template b2c-retail \
  --micro

# Run server with micro-topology parameters (8MB write buffer, 16MB block cache)
cargo run -p frappe-net -- --micro
```

---

## 4. Production Automated TLS / ACME Setup

`rustnext` features an in-process Let's Encrypt automated certificate resolver (`rustls-acme`), eliminating external reverse proxies like Nginx or Traefik:

```bash
cargo run -p frappe-net -- \
  --host 0.0.0.0 \
  --port 443 \
  --acme-domain enterprise.yourdomain.com \
  --acme-contact-email security@yourdomain.com \
  --acme-cache-dir /var/lib/rustnext/acme_certs
```

---

## 5. Running the Reactive Universal Desk Shell

Launch the cross-platform Dioxus client shell (`desk-app`):

```bash
# Launch interactive desktop workstation client
cargo run -p desk-app
```

The shell boots with active modules:
- Virtualized Data Grid (1,000,000 rows at 60 FPS)
- Dynamic Form Engine compiled from `DocTypeSchema`
- Critical Path Method (CPM) Gantt schedule viewer
- Real-time Statistical Process Control (SPC) Nelson-rule chart
- 3D Warehouse slotting and VDA 5050 AMR robot tracker
- Hardware Abstraction Layer (HAL) for barcode scanning and biometrics
- Role-Adaptive Multi-Persona Shell switcher (`/portal`, `/worker`, `/factory`, `/approvals`, `/admin`)

---

## 6. Mobile Application Packaging (Android & PWA)

### Android Native Bundle (`.apk` / `.aab`)
```bash
# Install cargo-apk if not already installed
cargo install cargo-apk

# Build release APK targeting aarch64-linux-android
cargo apk build --release --package desk-app --target aarch64-linux-android
```

### Progressive Web App (PWA) Substrate
The PWA manifest and caching Service Worker (`sw.js`) are dynamically compiled from `desk_app::mobile_pwa`:
- Manifest endpoint: `/manifest.webmanifest`
- Offline Service Worker: `/sw.js` (Cache-First pre-cache for WASM; Stale-While-Revalidate for live SurrealQL updates).

---

## 7. Verification & Quality Assurance Gates

Run the standard quality gate matrix before committing code:

```bash
# 1. Format check
cargo fmt --check

# 2. Strict Clippy check with zero warnings allowed
cargo clippy --workspace --all-targets -- -D warnings

# 3. Full test suite execution across all 21 crates
cargo test --workspace
```
