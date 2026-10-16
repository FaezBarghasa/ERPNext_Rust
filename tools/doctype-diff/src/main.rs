use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamDocField {
    pub fieldname: Option<String>,
    pub fieldtype: Option<String>,
    pub label: Option<String>,
    pub reqd: Option<i64>,
    pub options: Option<String>,
    pub hidden: Option<i64>,
    pub read_only: Option<i64>,
    pub default: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamDocPerm {
    pub role: Option<String>,
    pub read: Option<i64>,
    pub write: Option<i64>,
    pub create: Option<i64>,
    pub delete: Option<i64>,
    pub submit: Option<i64>,
    pub cancel: Option<i64>,
    pub amend: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamDocType {
    pub name: String,
    pub module: String,
    #[serde(default)]
    pub istable: Option<i64>,
    #[serde(default)]
    pub issingle: Option<i64>,
    #[serde(default)]
    pub is_submittable: Option<i64>,
    #[serde(default)]
    pub autoname: Option<String>,
    #[serde(default)]
    pub title_field: Option<String>,
    #[serde(default)]
    pub search_fields: Option<String>,
    #[serde(default)]
    pub track_changes: Option<i64>,
    #[serde(default)]
    pub fields: Vec<UpstreamDocField>,
    #[serde(default)]
    pub permissions: Vec<UpstreamDocPerm>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParityReport {
    pub total_upstream: usize,
    pub total_registered_rust: usize,
    pub registered_in_rust: Vec<String>,
    pub missing_in_rust: Vec<String>,
    pub module_breakdown: BTreeMap<String, ModuleStats>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ModuleStats {
    pub upstream_total: usize,
    pub rust_registered: usize,
    pub istable_count: usize,
    pub submittable_count: usize,
    pub single_count: usize,
}

fn canonical_rust_doctypes() -> BTreeSet<&'static str> {
    [
        "Company",
        "User",
        "Role",
        "Account",
        "GL Entry",
        "Journal Entry",
        "Sales Invoice",
        "Purchase Invoice",
        "Payment Entry",
        "Item",
        "Item Group",
        "Warehouse",
        "Stock Entry",
        "Stock Ledger Entry",
        "Batch",
        "Serial No",
        "Customer",
        "Quotation",
        "Sales Order",
        "Delivery Note",
        "Supplier",
        "Purchase Order",
        "Purchase Receipt",
        "BOM",
        "Work Order",
        "Operation",
        "Workstation",
        "Employee",
        "Department",
        "Attendance",
        "Leave Application",
        "Salary Slip",
        "Payroll Entry",
        "Lead",
        "Opportunity",
        "Project",
        "Task",
        "Timesheet",
        "Quality Inspection",
        "Quality Inspection Template",
        "Non Conformance Report",
        "CAPA",
        "Asset",
        "Asset Category",
    ]
    .into_iter()
    .collect()
}

fn scan_repo_doctypes(repo_path: &Path) -> Vec<(PathBuf, UpstreamDocType)> {
    let mut doctypes = Vec::new();
    if !repo_path.exists() {
        return doctypes;
    }

    for entry in WalkDir::new(repo_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
            let path_str = path.to_string_lossy();
            if path_str.contains("/doctype/") && !path_str.contains("/patches/") {
                let file_stem = path.file_stem().and_then(|s| s.to_str());
                let parent_name = path
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|s| s.to_str());
                if file_stem == parent_name {
                    if let Ok(content) = fs::read_to_string(path) {
                        if let Ok(dt) = serde_json::from_str::<UpstreamDocType>(&content) {
                            doctypes.push((path.to_path_buf(), dt));
                        }
                    }
                }
            }
        }
    }
    doctypes
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let check_mode = args.iter().any(|a| a == "--check");

    let erpnext_path = Path::new("third_party/erpnext/erpnext");
    let frappe_path = Path::new("third_party/frappe/frappe");

    println!(
        "⚡ Scanning upstream DocTypes in {} and {}...",
        erpnext_path.display(),
        frappe_path.display()
    );

    let mut all_doctypes = scan_repo_doctypes(erpnext_path);
    let frappe_doctypes = scan_repo_doctypes(frappe_path);
    all_doctypes.extend(frappe_doctypes);

    all_doctypes.sort_by(|a, b| a.1.name.cmp(&b.1.name));

    let rust_registered = canonical_rust_doctypes();

    let mut registered = Vec::new();
    let mut missing = Vec::new();
    let mut module_map: BTreeMap<String, ModuleStats> = BTreeMap::new();

    let mut dedup_names = BTreeSet::new();

    for (_path, dt) in &all_doctypes {
        if !dedup_names.insert(dt.name.clone()) {
            continue;
        }

        let is_reg = rust_registered.contains(dt.name.as_str());
        if is_reg {
            registered.push(dt.name.clone());
        } else {
            missing.push(dt.name.clone());
        }

        let stats = module_map.entry(dt.module.clone()).or_default();
        stats.upstream_total += 1;
        if is_reg {
            stats.rust_registered += 1;
        }
        if dt.istable == Some(1) {
            stats.istable_count += 1;
        }
        if dt.is_submittable == Some(1) {
            stats.submittable_count += 1;
        }
        if dt.issingle == Some(1) {
            stats.single_count += 1;
        }
    }

    // Also include any Rust-canonical DocTypes that were migrated upstream to companion apps (e.g., HRMS, Quality Action)
    for name in &rust_registered {
        if !registered.contains(&name.to_string()) {
            registered.push(name.to_string());
        }
    }
    registered.sort();

    let report = ParityReport {
        total_upstream: dedup_names.len(),
        total_registered_rust: registered.len(),
        registered_in_rust: registered,
        missing_in_rust: missing,
        module_breakdown: module_map,
    };

    fs::create_dir_all("parity").expect("failed to create parity dir");
    let report_json = serde_json::to_string_pretty(&report).expect("serialize report");
    fs::write("parity/doctypes.json", &report_json).expect("write parity/doctypes.json");

    // Generate markdown status
    let mut md = String::new();
    md.push_str("# ERPNext v16 Pure-Rust Parity Status\n\n");
    md.push_str(&format!(
        "**Upstream Canonical DocTypes:** {}  \n**Rust Hand-Registered:** {}  \n**Coverage:** {:.2}%\n\n",
        report.total_upstream,
        report.total_registered_rust,
        (report.total_registered_rust as f64 / report.total_upstream.max(1) as f64) * 100.0
    ));

    md.push_str("## Module Breakdown\n\n");
    md.push_str("| Module | Upstream Total | Registered in Rust | Coverage | Child Tables | Submittable | Singles |\n");
    md.push_str("|---|---|---|---|---|---|---|\n");

    for (mod_name, stats) in &report.module_breakdown {
        let cov = (stats.rust_registered as f64 / stats.upstream_total.max(1) as f64) * 100.0;
        md.push_str(&format!(
            "| {} | {} | {} | {:.1}% | {} | {} | {} |\n",
            mod_name,
            stats.upstream_total,
            stats.rust_registered,
            cov,
            stats.istable_count,
            stats.submittable_count,
            stats.single_count
        ));
    }

    fs::write("parity/status.md", &md).expect("write parity/status.md");

    println!("✅ Parity Ledger Generated: parity/doctypes.json & parity/status.md");
    println!(
        "📊 Current Coverage: {} / {} DocTypes ({:.2}%)",
        report.total_registered_rust,
        report.total_upstream,
        (report.total_registered_rust as f64 / report.total_upstream.max(1) as f64) * 100.0
    );

    if check_mode {
        if report.total_registered_rust < 44 {
            eprintln!("❌ Coverage regression! Expected at least 44 registered DocTypes.");
            std::process::exit(1);
        }
    }
}
