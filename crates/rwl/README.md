# ⚡ `rwl` — rust-web-land Unified CLI & 1-Click Launch Kit

The official command-line interface, project scaffolder, and standalone production runner for **rust-web-land** (`rwl`), the pure-Rust universal no-code web application platform.

---

## 🛠️ Unified Command Suite

```bash
# 1. Initialize a new application from a turnkey blueprint (cms, lms, saas, erp)
rwl init my-store --blueprint erp

# 2. Launch the visual builder studio locally at http://127.0.0.1:8080/builder
rwl studio --port 8080

# 3. Package an optimized 1GB-RAM-budgeted deployment bundle
rwl export --target vps --output ./deploy

# 4. Deploy incremental zero-downtime updates to a live production VPS
rwl deploy --host 192.168.1.100 --port 8080 --token secret123

# 5. Run the standalone production server with embedded SurrealKV database
rwl serve --port 8080 --host 0.0.0.0 --data-dir ./data/rust-web-land.db

# 6. Display runtime resource envelope, memory usage, and health status
rwl status
```

---

## 📦 What `rwl export` Produces

When running `rwl export --target vps`, the CLI generates a production-ready delivery bundle tailored for a low-cost $3–$5/month Linux VPS:

| Artifact | Purpose | Resource Limit / Feature |
| :--- | :--- | :--- |
| `rwl-net` / `rwl-server` | Stripped standalone server binary | $< 25\,\text{MB}$ size, zero external daemons |
| `rust-web-land.service` | Production systemd service unit | `MemoryMax=1024M`, `MemoryHigh=850M` |
| `start.sh` | Tuned launcher script | Configures embedded SurrealKV storage cache |
| `docker-compose.yml` | Optional OCI container configuration | Enforces strict cgroup memory caps |

---

## 📖 Related Guides

- 🚀 [**3-Minute Quickstart Guide**](../../docs/quickstart_3min_guide.md)
- 🏛 [**System Architecture Specification**](../../docs/architecture.md)
- 🛠 [**Developer & Administrator Setup Guide**](../../docs/setup_guide.md)
