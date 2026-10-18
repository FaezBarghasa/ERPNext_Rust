# `desk-app` — Cross-Platform Universal Shell, Persona Projections & Edge HAL

`desk-app` is the client runtime shell for the enterprise OS, built in pure Rust with Dioxus. It runs natively across desktop workstations (Linux, macOS, Windows) and mobile handhelds (Android, iOS) as well as offline-capable Progressive Web Apps (PWA).

---

## 🏛️ Architectural Overview

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                                   desk-app SHELL                                       │
│  ├── Role-Adaptive Persona Router (`persona_shell.rs`)                                 │
│  │   ├── ClientCustomer (/portal): E-commerce, LMS syllabus, Typst diploma            │
│  │   ├── WarehouseWorker (/worker): >=56px touch targets, fast-scan, TSP pick path     │
│  │   ├── ShopfloorMes (/factory): Digital traveler, dual-witness sign-off, live SPC    │
│  │   ├── ManagerApprover (/approvals): Swipe triage deck, streaming KPI tickers        │
│  │   └── SystemAdmin (/admin): Actix worker gauges, SurrealQL live multiplexer         │
│  │                                                                                     │
│  ├── Polymorphic Hardware Abstraction Layer (`hal.rs`)                                 │
│  │   ├── Android JNI: Native camera barcode scanning, Zebra BLE scanner, GPS           │
│  │   └── W3C Web APIs: WebAuthn biometrics, Web Bluetooth, ESC/POS thermal printer    │
│  │                                                                                     │
│  ├── Disconnected Edge Transactions & Semilattices (`edge_sync.rs`)                    │
│  │   ├── Causal Vector Clocks (`VectorClock`) tracking edge mutation lineage           │
│  │   ├── Local Mutation Buffer (`LocalMutationBuffer`) writing to SurrealKV            │
│  │   └── Cloud Sync Arbiter (`CloudSyncArbiter`) executing conflict-free CRDT joins    │
│  │                                                                                     │
│  └── PWA Substrate (`mobile_pwa.rs`)                                                   │
│      ├── W3C Web App Manifest generator (`manifest.webmanifest`)                       │
│      └── Offline Service Worker (`sw.js`) with Cache-First WASM caching                │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 📱 Persona Shell Endpoints

| Route | Persona | Primary Hardware Integration | UX Paradigm |
| :--- | :--- | :--- | :--- |
| `/portal` | **Client / Customer** | WebAuthn biometrics, credit card tokens | Consumer storefront, video HUD, LMS player |
| `/worker` | **Warehouse Worker** | Continuous laser/camera barcode, haptics | High-contrast, $\ge 56\,\text{px}$ touch targets, TSP pick path |
| `/factory` | **Shopfloor MES** | SCADA / Modbus, NFC badge reader | Step-by-step traveler checklists, dual witness |
| `/approvals` | **Manager / Approver** | Secure Enclave biometrics, push alerts | Swipe-driven card deck (Right: Approve, Left: Reject) |
| `/admin` | **System Administrator** | Audit signing keys, syslog stream | System telemetry gauges, SurrealQL query console |

---

## 🛠️ Usage & Compilation

```bash
# Launch native desktop application
cargo run -p desk-app

# Compile release APK for Android
cargo apk build --release --package desk-app --target aarch64-linux-android
```

