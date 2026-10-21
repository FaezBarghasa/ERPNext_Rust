# `rust-web-land` (`rwl`) — Developer & System Administrator Setup Guide

This document provides step-by-step instructions to install, build, configure, provision, and deploy `rust-web-land` (`rwl`) across local development workstations, constrained edge appliances, and cheap $3–$5/month Linux VPS production environments.

---

## 1. System Prerequisites

* **Rust Toolchain:** Rust 1.85.0+ (2024 Edition) with `cargo` and `rustfmt`:
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  rustup update stable
  ```
* **Host Operating System:** Linux (Pop!_OS / Ubuntu / Debian / Fedora / Arch), macOS, or Windows (WSL2).
* **Target Production Envelope:** Cheap $3–$5/month Linux VPS or low-power ARM single-board computers (Raspberry Pi 4/5, Orange Pi) strictly capped at **$\le 1\,\text{GB}$ RAM**.
* **Zero External DB Daemons:** Embedded SurrealKV database engine is compiled natively into the binary. No external MariaDB, Redis, or PostgreSQL instances are required.

---

## 2. Fast 3-Minute Quickstart

### Step 1: Clone Repository and Build CLI

```bash
git clone https://github.com/FaezBarghasa/ERPNext_Rust.git
cd ERPNext_Rust

# Build the unified rwl CLI binary
cargo build --release -p rwl

# Create alias or add to PATH
alias rwl="./target/release/rwl"
```

### Step 2: Initialize a Project from an Industry Blueprint

Scaffold a complete platform with pre-configured schemas, visual pages, and permissions in $<100\,\text{ms}$:

```bash
# Available blueprints: cms, lms, saas, erp
rwl init my-store --blueprint erp
cd my-store
```

### Step 3: Launch Visual Builder Studio

```bash
# Launches visual drag-and-drop workspace at http://127.0.0.1:8080/builder
rwl studio --port 8080
```

Open **`http://127.0.0.1:8080/builder`** in your browser, or launch the native Dioxus workstation client via `cargo run -p desk-app`.

---

## 3. Production VPS Deployment & 1GB Budget Envelope

### 3.1 Package the Production Bundle

Generate a lightweight, self-contained production bundle pre-configured for a 1GB VPS host:

```bash
rwl export --target vps --output ./deploy
```

The exported package includes:
- Stripped static server binary (`rwl-server` / `frappe-net`).
- Tuned `start.sh` launcher configuring embedded SurrealKV storage.
- Production `rust-web-land.service` systemd unit enforcing `MemoryMax=1024M` and `MemoryHigh=850M`.
- Optional `docker-compose.yml` with strict cgroup memory caps.

### 3.2 Systemd Production Service Setup

Install the service on your target Debian/Ubuntu VPS:

```bash
# Copy systemd unit file
sudo cp deploy/rust-web-land.service /etc/systemd/system/

# Reload systemd and start service
sudo systemctl daemon-reload
sudo systemctl enable --now rust-web-land.service

# Verify health and memory consumption (typically < 45MB idle RSS)
sudo systemctl status rust-web-land.service
```

### 3.3 Zero-Downtime Hot Upgrades via `rwl deploy`

Deploy incremental updates, new pages, and schema diffs to your live VPS without dropping user connections:

```bash
rwl deploy --host vps.yourdomain.com --port 8080 --token <SECRET_TOKEN>
```

---

## 4. Production Automated TLS / ACME Setup

`rust-web-land` features an in-process Let's Encrypt automated certificate resolver (`rustls-acme`), eliminating external reverse proxies like Nginx or Traefik:

```bash
cargo run -p frappe-net -- \
  --host 0.0.0.0 \
  --port 443 \
  --acme-domain enterprise.yourdomain.com \
  --acme-contact-email security@yourdomain.com \
  --acme-cache-dir /var/lib/rust-web-land/acme_certs
```

---

## 5. Standalone Production Server

Run the server directly with embedded SurrealKV:

```bash
rwl serve --port 8080 --host 0.0.0.0 --data-dir ./data/rust-web-land.db
```

### Key API & Interface Endpoints

* **Unified Enterprise Desk:** `http://127.0.0.1:8080/desk`
* **Visual Builder Studio:** `http://127.0.0.1:8080/builder`
* **DocType List View:** `http://127.0.0.1:8080/desk/list/sales_invoice`
* **DocType Form View:** `http://127.0.0.1:8080/desk/form/sales_invoice/new`
* **Printable Invoice Engine:** `http://127.0.0.1:8080/api/v2/document/sales_invoice/{name}/print`
* **Payment API Endpoint:** `http://127.0.0.1:8080/api/v2/payments/charge`
* **Universal Template Portal:** `http://127.0.0.1:8080/templates`
* **OTT Video HLS Master:** `http://127.0.0.1:8080/api/v2/streaming/hls/{media_id}/master.m3u8`
* **Subsonic REST v1.16+ API:** `http://127.0.0.1:8080/rest/ping.view`
* **Health & Liveness Check:** `http://127.0.0.1:8080/healthz/ready`

---

## 6. Mobile Application Packaging (Android & PWA)

### Android Native Bundle (`.apk`)

```bash
# Install cargo-apk if not already present
cargo install cargo-apk

# Build release APK targeting aarch64-linux-android
cargo apk build --release --package desk-app --target aarch64-linux-android
```

### Progressive Web App (PWA) Substrate

The PWA manifest and caching Service Worker (`sw.js`) are dynamically compiled from `desk_app::mobile_pwa`:
* Manifest endpoint: `/manifest.webmanifest`
* Offline Service Worker: `/sw.js` (Cache-First pre-cache for WASM; Stale-While-Revalidate for live SurrealQL updates).

---

## 7. Verification & Quality Assurance Gates

Run the standard quality gate matrix before committing code:

```bash
# 1. Format check
cargo fmt --check

# 2. Strict Clippy check with zero warnings allowed
cargo clippy --workspace --all-targets -- -D warnings

# 3. Full test suite execution across all 32 crates
cargo test --workspace

# 4. Crash recovery and ledger drift verification
cargo test -p rbench --test crash_recovery_tests

# 5. Dedicated security hardening test suite
cargo test -p rbench --test security_hardening_tests
```
