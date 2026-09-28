//! Command Line Interface Driver for `rustnext` / `frappe-net`.
//!
//! Provides subcommands: `start`, `migrate`, `tenant`, and `benchmark`.

use clap::{Args, Parser, Subcommand};

/// Planetary-scale pure-Rust ERP, CMS, and Digital Commerce OS runtime.
#[derive(Parser, Debug)]
#[command(
    name = "rustnext",
    version,
    about = "High-performance Frappe/ERPNext engine in pure Rust"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the Actix-web server and SurrealDB multi-tenant core.
    Start(StartArgs),
    /// Execute database schema migrations across tenants.
    Migrate(MigrateArgs),
    /// Manage tenant environments, domains, and namespaces.
    Tenant(TenantArgs),
    /// Run synthetic load generator and latency micro-benchmarks.
    Benchmark(BenchmarkArgs),
}

#[derive(Args, Debug, Clone)]
pub struct StartArgs {
    /// Bind address for the HTTP/WebSocket server.
    #[arg(short, long, default_value = "127.0.0.1:8000")]
    pub bind: String,

    /// Run in sub-64MB resident memory Micro-Mode (capped buffers, single-core profile).
    #[arg(long, default_value_t = false)]
    pub micro: bool,

    /// Primary domain for automated ACME TLS certificate negotiation.
    #[arg(long)]
    pub acme_domain: Option<String>,

    /// Number of worker threads (defaults to logical CPU count, or 1 in micro-mode).
    #[arg(short, long)]
    pub workers: Option<usize>,
}

#[derive(Args, Debug, Clone)]
pub struct MigrateArgs {
    /// Target specific tenant identifier (or all if omitted).
    #[arg(short, long)]
    pub tenant: Option<String>,

    /// Dry run mode (print differential DDL statements without applying).
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
}

#[derive(Args, Debug, Clone)]
pub struct TenantArgs {
    #[command(subcommand)]
    pub action: TenantCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum TenantCommands {
    /// Create a new isolated tenant namespace.
    Create {
        /// Unique tenant identifier (alphanumeric).
        name: String,
        /// Primary domain mapping.
        #[arg(short, long)]
        domain: Option<String>,
    },
    /// List all registered tenant namespaces.
    List,
    /// Delete a tenant namespace.
    Delete {
        /// Tenant identifier to remove.
        name: String,
        /// Force removal without confirmation.
        #[arg(short, long, default_value_t = false)]
        force: bool,
    },
}

#[derive(Args, Debug, Clone)]
pub struct BenchmarkArgs {
    /// Target URL for benchmarking.
    #[arg(short, long, default_value = "http://127.0.0.1:8000/health")]
    pub url: String,

    /// Number of concurrent clients.
    #[arg(short, long, default_value_t = 50)]
    pub concurrency: usize,

    /// Benchmark duration in seconds.
    #[arg(short, long, default_value_t = 10)]
    pub duration: u64,
}
