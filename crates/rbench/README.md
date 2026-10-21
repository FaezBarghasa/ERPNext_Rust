# `rbench` — Site Orchestration, Packaging & High-Throughput Load Testing CLI

`rbench` is the site orchestration and verification command-line tool for multi-tenant site provisioning, automated migrations, distribution packaging, TypeScript SDK generation, and high-concurrency load testing for `rust-web-land` (`rwl`).

---

## 🚀 Key Commands

### 1. Site Deployment & Template Provisioning
Deploy any of the 10 prebuilt enterprise template suites with $<2000\,\text{ms}$ cold start:
```bash
cargo run -p rbench -- site deploy \
  --site-name shop.enterprise.local \
  --template svod-streaming \
  --admin-email admin@enterprise.local \
  [--micro]
```

**Supported Template Suites:**
* `svod-streaming`: Multi-bitrate HLS v6, SSAI SCTE-35 ad splicing, FAST linear syndication.
* `lms-academy`: Course graph syllabus, video lessons, Typst Merkle-verified diplomas.
* `digital-goods`: Interactive live type-tester, variable font licensing, signed HMAC downloads.
* `b2b-industrial`: Parametric 3D CAD viewer, volume pricing tiers, ZUGFeRD 2.2 invoices.
* `b2c-retail`: Three.js luxury exploded calibres, metallurgy shader switching, real-time cart.
* `trading-exchange`: High-density L2 order book, continuous WebSocket price feeds, trade slips.
* `real-estate-luxury`: Interactive floorplans, 3D room tours, dynamic mortgage estimator.
* `healthcare-clinic`: HIPAA-compliant appointment booking, doctor schedules, telehealth portals.
* `restaurant-culinary`: Visual table seating floorplan, dietary allergen tags, reservation queue.
* `news-editorial`: Scroll reading progress, estimated read times, breaking news ticker, paywalls.

### 2. Tenant Lifecycle Management
```bash
# Provision new database tenant with encryption keys and admin credentials
cargo run -p rbench -- new-site tenant1.enterprise.local

# Remove site with multi-stage cryptographic data shredding
cargo run -p rbench -- drop-site tenant1.enterprise.local
```

### 3. Zero-Downtime Migrations
```bash
# Perform bitemporal schema reconciliation across DocTypes
cargo run -p rbench -- migrate [--skip-fixtures]
```

### 4. Backups & Point-in-Time Restores
```bash
# Streamed compressed snapshot with zstd and SHA-256 Merkle root proof
cargo run -p rbench -- backup --site tenant1.enterprise.local --out /var/backups/

# Restore from snapshot archive with optional partial-restore filters
cargo run -p rbench -- restore --site tenant1.enterprise.local --archive /var/backups/snapshot.tar.zst
```

### 5. SDK Generation
```bash
# Generate strict TypeScript SDK with runtime type validators
cargo run -p rbench -- sdk generate --lang typescript --output ./sdk/typescript
```

### 6. High-Throughput Load Testing & Benchmarking
```bash
# Benchmark General Ledger, Stock Ledger, and document insertion TPS
cargo run -p rbench -- benchmark \
  --concurrency 128 \
  --duration 30s \
  --target-tps 25000
```
Measures throughput (TPS), latency distribution (p50, p99, p99.9), memory RSS footprint, and CPU utilization under sustained load.

