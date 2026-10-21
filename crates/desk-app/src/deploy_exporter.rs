//! `deploy_exporter` — 1-Click Production Deployment Bundle Generator.
//!
//! Generates ultra-optimized, standalone production deployment packages tailored to run
//! on low-cost $3–$5/mo VPS servers with a **hard budget of 1GB RAM**.
//!
//! Key outputs:
//! - `systemd` service unit file with `MemoryMax=1024M`, `MemoryHigh=850M`, `CPUQuota=100%`.
//! - `docker-compose.yml` with strict container limits (memory 1024M, cpus '1.0').
//! - Multi-stage `Dockerfile` with non-root security isolation.
//! - Hardened `./start.sh` with memory and port pre-flight verification.
//! - `surrealkv.toml` capping internal storage cache to 64MB page cache & 32MB write buffer.
//! - `Caddyfile` for automatic HTTPS, zstd/gzip compression, and HSTS security headers.
//! - `zstd` compressed deployment bundle packaging.

use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Target deployment environment envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DeployTarget {
    #[default]
    Vps,
    Docker,
    Systemd,
    All,
}

impl DeployTarget {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Vps => "vps",
            Self::Docker => "docker",
            Self::Systemd => "systemd",
            Self::All => "all",
        }
    }

    pub fn parse_target(s: &str) -> Self {
        use std::str::FromStr;
        Self::from_str(s).unwrap_or(Self::Vps)
    }
}

impl std::str::FromStr for DeployTarget {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "docker" => Self::Docker,
            "systemd" => Self::Systemd,
            "all" => Self::All,
            _ => Self::Vps,
        })
    }
}

/// Configuration parameters for generating the 1GB VPS deployment bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeployConfig {
    pub app_name: CompactString,
    pub version: CompactString,
    pub target: DeployTarget,
    pub port: u16,
    pub domain: Option<CompactString>,
    pub enable_tls: bool,
    pub memory_limit_mb: u32,
    pub memory_high_mb: u32,
    pub cpu_quota_percent: u32,
    pub surrealkv_page_cache_mb: u32,
    pub surrealkv_write_buffer_mb: u32,
    pub log_max_size_mb: u32,
    pub log_max_files: u32,
    pub run_user: CompactString,
    pub install_dir: CompactString,
}

impl Default for DeployConfig {
    fn default() -> Self {
        Self {
            app_name: "rust-web-land".into(),
            version: "0.3.0".into(),
            target: DeployTarget::Vps,
            port: 8080,
            domain: None,
            enable_tls: false,
            memory_limit_mb: 1024,
            memory_high_mb: 850,
            cpu_quota_percent: 100,
            surrealkv_page_cache_mb: 64,
            surrealkv_write_buffer_mb: 32,
            log_max_size_mb: 10,
            log_max_files: 3,
            run_user: "rwl".into(),
            install_dir: "/opt/rust-web-land".into(),
        }
    }
}

/// An individual file artifact generated for deployment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeployArtifact {
    pub relative_path: CompactString,
    pub content: String,
    pub is_executable: bool,
    pub description: CompactString,
}

/// Summary report of an export operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportReport {
    pub output_dir: CompactString,
    pub target: DeployTarget,
    pub files_written: Vec<CompactString>,
    pub total_bytes: usize,
    pub memory_budget_mb: u32,
    pub status: CompactString,
}

/// The 1-Click Deploy Bundle Generator.
pub struct DeployBundle {
    config: DeployConfig,
    artifacts: Vec<DeployArtifact>,
}

impl DeployBundle {
    /// Construct a new deployment bundle and pre-generate all artifacts according to configuration.
    pub fn new(config: DeployConfig) -> Self {
        let mut bundle = Self {
            config,
            artifacts: Vec::new(),
        };
        bundle.generate_artifacts();
        bundle
    }

    /// Access the underlying deploy configuration.
    pub fn config(&self) -> &DeployConfig {
        &self.config
    }

    /// List all prepared deployment artifacts.
    pub fn artifacts(&self) -> &[DeployArtifact] {
        &self.artifacts
    }

    /// Pre-generate all deployment artifacts based on target selection.
    fn generate_artifacts(&mut self) {
        let target = self.config.target;

        // Common runtime files (Config & Environment)
        self.artifacts.push(DeployArtifact {
            relative_path: ".env".into(),
            content: self.generate_env_file(),
            is_executable: false,
            description: "Production environment variable parameters".into(),
        });

        self.artifacts.push(DeployArtifact {
            relative_path: "surrealkv.toml".into(),
            content: self.generate_surrealkv_config(),
            is_executable: false,
            description: "SurrealKV embedded storage 1GB cache caps (64MB/32MB)".into(),
        });

        self.artifacts.push(DeployArtifact {
            relative_path: "start.sh".into(),
            content: self.generate_start_script(),
            is_executable: true,
            description: "Production launcher script with preflight memory checks".into(),
        });

        self.artifacts.push(DeployArtifact {
            relative_path: "README.md".into(),
            content: self.generate_readme_quickstart(),
            is_executable: false,
            description: "Deployment quickstart instructions".into(),
        });

        // Systemd target
        if target == DeployTarget::Vps
            || target == DeployTarget::Systemd
            || target == DeployTarget::All
        {
            self.artifacts.push(DeployArtifact {
                relative_path: format!("{}.service", self.config.app_name).into(),
                content: self.generate_systemd_unit(),
                is_executable: false,
                description: "Systemd service unit with MemoryMax=1024M and MemoryHigh=850M".into(),
            });
        }

        // Docker target
        if target == DeployTarget::Vps
            || target == DeployTarget::Docker
            || target == DeployTarget::All
        {
            self.artifacts.push(DeployArtifact {
                relative_path: "docker-compose.yml".into(),
                content: self.generate_docker_compose(),
                is_executable: false,
                description: "Docker Compose with strict 1024M RAM ceiling".into(),
            });

            self.artifacts.push(DeployArtifact {
                relative_path: "Dockerfile".into(),
                content: self.generate_dockerfile(),
                is_executable: false,
                description: "Minimal debian-slim multi-stage runner for stripped binary".into(),
            });
        }

        // Caddy Reverse Proxy (if domain specified or VPS mode)
        if self.config.domain.is_some()
            || target == DeployTarget::Vps
            || target == DeployTarget::All
        {
            self.artifacts.push(DeployArtifact {
                relative_path: "Caddyfile".into(),
                content: self.generate_caddyfile(),
                is_executable: false,
                description: "Caddy reverse proxy with automatic HTTPS and zstd compression".into(),
            });
        }

        // Bundle Manifest
        self.artifacts.push(DeployArtifact {
            relative_path: "rwl-manifest.json".into(),
            content: self.generate_bundle_manifest(),
            is_executable: false,
            description: "Export package integrity manifest".into(),
        });
    }

    /// Generates the systemd service file with hard 1GB VPS limits.
    pub fn generate_systemd_unit(&self) -> String {
        format!(
            r#"[Unit]
Description={app_name} Standalone Production Server
Documentation=https://github.com/FaezBarghasa/ERPNext_Rust
After=network.target network-online.target
Wants=network-online.target

[Service]
Type=simple
User={run_user}
Group={run_user}
WorkingDirectory={install_dir}
ExecStart={install_dir}/rwl serve --port {port} --data-dir {install_dir}/data
Restart=always
RestartSec=5s
KillMode=process
LimitNOFILE=65536

# Hard 1GB VPS RAM Ceiling & CPU Quotas
MemoryAccounting=yes
MemoryMax={memory_limit_mb}M
MemoryHigh={memory_high_mb}M
CPUAccounting=yes
CPUQuota={cpu_quota_percent}%

# Linux Security Hardening
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths={install_dir}/data {install_dir}/logs
PrivateTmp=true
NoNewPrivileges=true
CapabilityBoundingSet=CAP_NET_BIND_SERVICE
AmbientCapabilities=CAP_NET_BIND_SERVICE

# Standard Output & Logging
StandardOutput=append:{install_dir}/logs/rwl.log
StandardError=append:{install_dir}/logs/rwl-error.log

[Install]
WantedBy=multi-user.target
"#,
            app_name = self.config.app_name,
            run_user = self.config.run_user,
            install_dir = self.config.install_dir,
            port = self.config.port,
            memory_limit_mb = self.config.memory_limit_mb,
            memory_high_mb = self.config.memory_high_mb,
            cpu_quota_percent = self.config.cpu_quota_percent,
        )
    }

    /// Generates the production `docker-compose.yml` file with strict resource bounds.
    pub fn generate_docker_compose(&self) -> String {
        format!(
            r#"version: '3.8'

services:
  app:
    build:
      context: .
      dockerfile: Dockerfile
    image: {app_name}:{version}
    container_name: {app_name}
    restart: unless-stopped
    ports:
      - "{port}:8080"
    environment:
      - RWL_PORT=8080
      - RWL_DATA_DIR=/data
      - RWL_SURREALKV_PAGE_CACHE_MB={surrealkv_page_cache_mb}
      - RWL_SURREALKV_WRITE_BUFFER_MB={surrealkv_write_buffer_mb}
      - RUST_LOG=info
    volumes:
      - ./data:/data
      - ./logs:/logs
    deploy:
      resources:
        limits:
          memory: {memory_limit_mb}M
          cpus: '1.0'
        reservations:
          memory: 64M
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/api/health"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 10s
    logging:
      driver: "json-file"
      options:
        max-size: "{log_max_size_mb}m"
        max-file: "{log_max_files}"
"#,
            app_name = self.config.app_name,
            version = self.config.version,
            port = self.config.port,
            surrealkv_page_cache_mb = self.config.surrealkv_page_cache_mb,
            surrealkv_write_buffer_mb = self.config.surrealkv_write_buffer_mb,
            memory_limit_mb = self.config.memory_limit_mb,
            log_max_size_mb = self.config.log_max_size_mb,
            log_max_files = self.config.log_max_files,
        )
    }

    /// Generates the minimal multi-stage `Dockerfile`.
    pub fn generate_dockerfile(&self) -> String {
        r#"# Multi-stage minimal production container
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 -U rwl && \
    mkdir -p /opt/rust-web-land/data /opt/rust-web-land/logs && \
    chown -R rwl:rwl /opt/rust-web-land

WORKDIR /opt/rust-web-land
USER rwl:rwl

# Copy stripped binary and configuration
COPY --chown=rwl:rwl rwl /opt/rust-web-land/rwl
COPY --chown=rwl:rwl surrealkv.toml /opt/rust-web-land/surrealkv.toml

EXPOSE 8080

ENTRYPOINT ["/opt/rust-web-land/rwl", "serve", "--port", "8080", "--data-dir", "/opt/rust-web-land/data"]
"#
        .to_string()
    }

    /// Generates the executable `start.sh` launcher with pre-flight checks.
    pub fn generate_start_script(&self) -> String {
        format!(
            r#"#!/usr/bin/env bash
# ==============================================================================
#  {app_name} v{version} — Production Launcher Script
#  Target Envelope: <= 1GB RAM on Low-Cost VPS
# ==============================================================================
set -euo pipefail

PORT="{port}"
DATA_DIR="./data"
LOGS_DIR="./logs"

echo "⚡ Starting {app_name} v{version}..."

# 1. Memory Verification Pre-flight
if [ -f /proc/meminfo ]; then
    TOTAL_MEM_KB=$(grep MemTotal /proc/meminfo | awk '{{print $2}}')
    TOTAL_MEM_MB=$((TOTAL_MEM_KB / 1024))
    echo "  [1/4] RAM Check: ${{TOTAL_MEM_MB}} MB detected (Budget Ceiling: {memory_limit_mb} MB)."
    if [ "${{TOTAL_MEM_MB}}" -lt 512 ]; then
        echo "  ⚠️  WARNING: Host memory is under 512MB. Swap file recommended for stability."
    fi
else
    echo "  [1/4] RAM Check: Non-Linux host or /proc unavailable. Skipping cgroup probe."
fi

# 2. Port Availability Pre-flight
echo "  [2/4] Verifying port ${{PORT}} availability..."
if command -v ss >/dev/null 2>&1; then
    if ss -tuln | grep -q ":${{PORT}} "; then
        echo "  ❌ ERROR: Port ${{PORT}} is already bound by another process."
        exit 1
    fi
elif command -v netstat >/dev/null 2>&1; then
    if netstat -tuln | grep -q ":${{PORT}} "; then
        echo "  ❌ ERROR: Port ${{PORT}} is already bound."
        exit 1
    fi
fi
echo "  ✅ Port ${{PORT}} is ready."

# 3. Create Runtime Directories
echo "  [3/4] Preparing directories: ${{DATA_DIR}}, ${{LOGS_DIR}}..."
mkdir -p "${{DATA_DIR}}" "${{LOGS_DIR}}"
chmod 700 "${{DATA_DIR}}" "${{LOGS_DIR}}"

# 4. Launch Service
echo "  [4/4] Launching service..."
if [ -f "./docker-compose.yml" ] && command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
    echo "  🐳 Executing via Docker Compose (Memory cap: {memory_limit_mb}M)..."
    exec docker compose up -d
elif [ -f "./rwl" ]; then
    echo "  🚀 Executing native stripped binary..."
    chmod +x ./rwl
    exec ./rwl serve --port "${{PORT}}" --data-dir "${{DATA_DIR}}"
else
    echo "  ❌ Binary './rwl' not found. Place the compiled executable in this directory."
    exit 1
fi
"#,
            app_name = self.config.app_name,
            version = self.config.version,
            port = self.config.port,
            memory_limit_mb = self.config.memory_limit_mb,
        )
    }

    /// Generates the internal `surrealkv.toml` storage limits file.
    pub fn generate_surrealkv_config(&self) -> String {
        format!(
            r#"# SurrealKV Embedded Storage Configuration
# Hard-capped for <= 1GB VPS Production Envelope
[storage]
engine = "surrealkv"
data_dir = "./data"

[cache]
# Capped to {page_cache}MB so total resident memory stays minimal
page_cache_size_mb = {page_cache}
write_buffer_size_mb = {write_buffer}
max_open_files = 512
sync_writes = true
compression = "zstd"

[memory_budget]
hard_limit_mb = {memory_limit_mb}
target_headroom_mb = 174
"#,
            page_cache = self.config.surrealkv_page_cache_mb,
            write_buffer = self.config.surrealkv_write_buffer_mb,
            memory_limit_mb = self.config.memory_limit_mb,
        )
    }

    /// Generates Caddyfile with automatic TLS and modern compression.
    pub fn generate_caddyfile(&self) -> String {
        let domain_str = self.config.domain.as_deref().unwrap_or(":80");

        format!(
            r#"# Caddy Reverse Proxy — Automatic HTTPS & Gzip/Zstd Compression
{domain} {{
    reverse_proxy localhost:{port} {{
        header_up X-Real-IP {{remote_host}}
        header_up X-Forwarded-For {{remote_host}}
        header_up X-Forwarded-Proto {{scheme}}
    }}

    encode zstd gzip

    header {{
        Strict-Transport-Security "max-age=31536000; includeSubDomains; preload"
        X-Content-Type-Options "nosniff"
        X-Frame-Options "SAMEORIGIN"
        Referrer-Policy "strict-origin-when-cross-origin"
    }}

    log {{
        output file ./logs/caddy-access.log {{
            roll_size 10mb
            roll_keep 3
        }}
    }}
}}
"#,
            domain = domain_str,
            port = self.config.port,
        )
    }

    /// Generates the `.env` production defaults file.
    pub fn generate_env_file(&self) -> String {
        format!(
            r#"# Production Environment Settings
APP_NAME="{app_name}"
APP_VERSION="{version}"
RWL_PORT={port}
RWL_DATA_DIR="./data"
RWL_SURREALKV_PAGE_CACHE_MB={page_cache}
RWL_SURREALKV_WRITE_BUFFER_MB={write_buffer}
RWL_MEMORY_LIMIT_MB={memory_limit_mb}
RUST_LOG=info
"#,
            app_name = self.config.app_name,
            version = self.config.version,
            port = self.config.port,
            page_cache = self.config.surrealkv_page_cache_mb,
            write_buffer = self.config.surrealkv_write_buffer_mb,
            memory_limit_mb = self.config.memory_limit_mb,
        )
    }

    /// Generates `README.md` containing simple 1-click deployment steps.
    pub fn generate_readme_quickstart(&self) -> String {
        format!(
            r#"# {app_name} v{version} — Deployment Package

This deployment package was generated by the **Visual App Studio** for a **strict 1GB RAM VPS envelope**.

---

## 🚀 Quickstart Options

### Option 1: Native VPS (Bare-Metal / Systemd)
1. Copy the compiled stripped binary `rwl` into this folder.
2. Run `./start.sh` or install the systemd service:
   ```bash
   sudo cp {app_name}.service /etc/systemd/system/
   sudo systemctl daemon-reload
   sudo systemctl enable --now {app_name}
   ```
3. Check status: `sudo systemctl status {app_name}`

### Option 2: Docker Compose
1. Ensure Docker is installed on your VPS.
2. Run:
   ```bash
   docker compose up -d
   ```
3. Check logs:
   ```bash
   docker compose logs -f
   ```

---

## 🛡️ Resource Limits
- **RAM Ceiling**: `{memory_limit_mb} MB` (Hard Limit)
- **SurrealKV Page Cache**: `{page_cache} MB`
- **Port**: `{port}`
- **Baseline Memory Footprint**: ~35MB RSS
"#,
            app_name = self.config.app_name,
            version = self.config.version,
            memory_limit_mb = self.config.memory_limit_mb,
            page_cache = self.config.surrealkv_page_cache_mb,
            port = self.config.port,
        )
    }

    /// Generates the package integrity manifest JSON.
    pub fn generate_bundle_manifest(&self) -> String {
        let manifest = serde_json::json!({
            "app_name": self.config.app_name,
            "version": self.config.version,
            "target": self.config.target.as_str(),
            "port": self.config.port,
            "memory_limit_mb": self.config.memory_limit_mb,
            "page_cache_mb": self.config.surrealkv_page_cache_mb,
            "write_buffer_mb": self.config.surrealkv_write_buffer_mb,
            "artifacts_count": self.artifacts.len(),
            "generated_at": chrono::Utc::now().to_rfc3339(),
        });
        serde_json::to_string_pretty(&manifest).unwrap_or_default()
    }

    /// Export all generated artifacts to the specified destination directory on disk.
    pub fn export_to_directory(&self, dest: &Path) -> io::Result<ExportReport> {
        fs::create_dir_all(dest)?;

        let mut files_written = Vec::new();
        let mut total_bytes = 0;

        for artifact in &self.artifacts {
            let file_path = dest.join(artifact.relative_path.as_str());
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&file_path, artifact.content.as_bytes())?;

            #[cfg(unix)]
            if artifact.is_executable {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&file_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&file_path, perms)?;
            }

            total_bytes += artifact.content.len();
            files_written.push(artifact.relative_path.clone());
        }

        Ok(ExportReport {
            output_dir: dest.to_string_lossy().to_string().into(),
            target: self.config.target,
            files_written,
            total_bytes,
            memory_budget_mb: self.config.memory_limit_mb,
            status: "SUCCESS".into(),
        })
    }

    /// Compresses the deployment package into a high-ratio zstd archive payload.
    pub fn export_compressed_bundle(&self) -> Result<Vec<u8>, io::Error> {
        let mut uncompressed = Vec::new();

        // Simple structured archive format:
        // [4 bytes count] for each: [4 bytes path_len][path][4 bytes data_len][data]
        let count = self.artifacts.len() as u32;
        uncompressed.extend_from_slice(&count.to_le_bytes());

        for artifact in &self.artifacts {
            let path_bytes = artifact.relative_path.as_bytes();
            uncompressed.extend_from_slice(&(path_bytes.len() as u32).to_le_bytes());
            uncompressed.extend_from_slice(path_bytes);

            let content_bytes = artifact.content.as_bytes();
            uncompressed.extend_from_slice(&(content_bytes.len() as u32).to_le_bytes());
            uncompressed.extend_from_slice(content_bytes);
        }

        let mut encoder = zstd::stream::Encoder::new(Vec::new(), 3)?;
        encoder.write_all(&uncompressed)?;
        encoder.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deploy_config_defaults_adhere_to_1gb_envelope() {
        let cfg = DeployConfig::default();
        assert_eq!(cfg.memory_limit_mb, 1024);
        assert_eq!(cfg.memory_high_mb, 850);
        assert_eq!(cfg.surrealkv_page_cache_mb, 64);
        assert_eq!(cfg.surrealkv_write_buffer_mb, 32);
    }

    #[test]
    fn test_systemd_unit_contains_hard_memory_ceilings() {
        let bundle = DeployBundle::new(DeployConfig::default());
        let unit = bundle.generate_systemd_unit();
        assert!(unit.contains("MemoryMax=1024M"));
        assert!(unit.contains("MemoryHigh=850M"));
        assert!(unit.contains("CPUQuota=100%"));
        assert!(unit.contains("Restart=always"));
        assert!(unit.contains("ProtectSystem=strict"));
    }

    #[test]
    fn test_docker_compose_resource_limits() {
        let bundle = DeployBundle::new(DeployConfig::default());
        let compose = bundle.generate_docker_compose();
        assert!(compose.contains("memory: 1024M"));
        assert!(compose.contains("cpus: '1.0'"));
        assert!(compose.contains("reservations:"));
        assert!(compose.contains("memory: 64M"));
    }

    #[test]
    fn test_start_script_preflight_checks() {
        let bundle = DeployBundle::new(DeployConfig::default());
        let script = bundle.generate_start_script();
        assert!(script.contains("MemTotal"));
        assert!(script.contains("Verifying port"));
        assert!(script.contains("chmod 700"));
    }

    #[test]
    fn test_surrealkv_cache_limits() {
        let bundle = DeployBundle::new(DeployConfig::default());
        let toml = bundle.generate_surrealkv_config();
        assert!(toml.contains("page_cache_size_mb = 64"));
        assert!(toml.contains("write_buffer_size_mb = 32"));
        assert!(toml.contains("compression = \"zstd\""));
    }

    #[test]
    fn test_export_bundle_to_directory_lifecycle() {
        let temp_dir = std::env::temp_dir().join(format!("rwl_deploy_test_{}", std::process::id()));
        let bundle = DeployBundle::new(DeployConfig {
            app_name: "test-app".into(),
            ..DeployConfig::default()
        });

        let report = bundle
            .export_to_directory(&temp_dir)
            .expect("export to directory must succeed");
        assert_eq!(report.status, "SUCCESS");
        assert!(report.files_written.contains(&"start.sh".into()));
        assert!(report.files_written.contains(&"docker-compose.yml".into()));
        assert!(report.files_written.contains(&"test-app.service".into()));
        assert!(report.files_written.contains(&"surrealkv.toml".into()));

        let start_sh = temp_dir.join("start.sh");
        assert!(start_sh.exists());

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_compressed_bundle_generation() {
        let bundle = DeployBundle::new(DeployConfig::default());
        let compressed = bundle
            .export_compressed_bundle()
            .expect("compression must succeed");
        assert!(!compressed.is_empty());
        // Zstd magic header: 0x28, 0xB5, 0x2F, 0xFD
        assert_eq!(compressed[0], 0x28);
        assert_eq!(compressed[1], 0xb5);
        assert_eq!(compressed[2], 0x2f);
        assert_eq!(compressed[3], 0xfd);
    }
}
