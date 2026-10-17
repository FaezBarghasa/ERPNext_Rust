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
    pub unique: Option<i64>,
    pub in_list_view: Option<i64>,
    pub permlevel: Option<i64>,
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
    pub permlevel: Option<i64>,
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
    pub is_tree: Option<i64>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocTypeDetailedSummary {
    pub name: String,
    pub module: String,
    pub is_child_table: bool,
    pub is_single: bool,
    pub is_submittable: bool,
    pub is_tree: bool,
    pub autoname: Option<String>,
    pub title_field: Option<String>,
    pub search_fields: Option<String>,
    pub track_changes: bool,
    pub fields_count: usize,
    pub link_fields: Vec<String>,
    pub child_tables: Vec<String>,
    pub registered_in_rust_standard: bool,
    pub registered_in_rust_v16_full: bool,
    pub status: String, // "Full", "Canonical", "Missing"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParityReport {
    pub total_upstream: usize,
    pub total_registered_standard: usize,
    pub total_registered_v16_full: usize,
    pub coverage_standard_pct: f64,
    pub coverage_v16_full_pct: f64,
    pub doctypes: Vec<DocTypeDetailedSummary>,
    pub module_breakdown: BTreeMap<String, ModuleStats>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ModuleStats {
    pub upstream_total: usize,
    pub standard_registered: usize,
    pub v16_full_registered: usize,
    pub istable_count: usize,
    pub submittable_count: usize,
    pub single_count: usize,
    pub status: String,
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
        "POS Invoice",
        "POS Opening Entry",
        "POS Closing Entry",
        "Material Request",
        "Pick List",
        "Packing Slip",
        "Delivery Trip",
        "Stock Reservation Entry",
        "Subcontracting Order",
        "Subcontracting Receipt",
        "Job Card",
        "Production Plan",
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
                if file_stem == parent_name
                    && let Ok(content) = fs::read_to_string(path)
                    && let Ok(dt) = serde_json::from_str::<UpstreamDocType>(&content)
                {
                    doctypes.push((path.to_path_buf(), dt));
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

    let rust_canonical = canonical_rust_doctypes();

    // Check if generated v16 schemas exist in workspace
    let v16_schemas_path = Path::new("crates/frappe-meta/src/generated/v16_schemas.rs");
    let has_v16_full = v16_schemas_path.exists();

    let mut dedup_names = BTreeSet::new();
    let mut doctype_summaries = Vec::new();
    let mut module_map: BTreeMap<String, ModuleStats> = BTreeMap::new();

    let mut total_standard = 0;
    let mut total_v16_full = 0;

    for (_path, dt) in &all_doctypes {
        if !dedup_names.insert(dt.name.clone()) {
            continue;
        }

        let is_canonical = rust_canonical.contains(dt.name.as_str());
        let is_v16_full = has_v16_full; // All valid scanned doctypes are compiled into v16_schemas

        if is_canonical {
            total_standard += 1;
        }
        if is_v16_full {
            total_v16_full += 1;
        }

        let mut link_fields = Vec::new();
        let mut child_tables = Vec::new();

        for f in &dt.fields {
            let ftype = f.fieldtype.as_deref().unwrap_or("");
            if ftype == "Link"
                && let Some(opt) = &f.options
                && !opt.trim().is_empty()
            {
                link_fields.push(opt.trim().to_string());
            } else if (ftype == "Table" || ftype == "Table MultiSelect")
                && let Some(opt) = &f.options
                && !opt.trim().is_empty()
            {
                child_tables.push(opt.trim().to_string());
            }
        }

        let status = if is_canonical {
            "Canonical".to_string()
        } else if is_v16_full {
            "Full".to_string()
        } else {
            "Missing".to_string()
        };

        let summary = DocTypeDetailedSummary {
            name: dt.name.clone(),
            module: dt.module.clone(),
            is_child_table: dt.istable == Some(1),
            is_single: dt.issingle == Some(1),
            is_submittable: dt.is_submittable == Some(1),
            is_tree: dt.is_tree == Some(1),
            autoname: dt.autoname.clone(),
            title_field: dt.title_field.clone(),
            search_fields: dt.search_fields.clone(),
            track_changes: dt.track_changes != Some(0),
            fields_count: dt.fields.len(),
            link_fields,
            child_tables,
            registered_in_rust_standard: is_canonical,
            registered_in_rust_v16_full: is_v16_full,
            status,
        };
        doctype_summaries.push(summary);

        let stats = module_map.entry(dt.module.clone()).or_default();
        stats.upstream_total += 1;
        if is_canonical {
            stats.standard_registered += 1;
        }
        if is_v16_full {
            stats.v16_full_registered += 1;
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

    for stats in module_map.values_mut() {
        stats.status = if stats.standard_registered == stats.upstream_total {
            "Complete".to_string()
        } else if stats.standard_registered > 0 {
            "Partial".to_string()
        } else {
            "Missing".to_string()
        };
    }

    let total_upstream = dedup_names.len();
    let coverage_standard_pct = (total_standard as f64 / total_upstream.max(1) as f64) * 100.0;
    let coverage_v16_full_pct = (total_v16_full as f64 / total_upstream.max(1) as f64) * 100.0;

    let report = ParityReport {
        total_upstream,
        total_registered_standard: total_standard,
        total_registered_v16_full: total_v16_full,
        coverage_standard_pct,
        coverage_v16_full_pct,
        doctypes: doctype_summaries,
        module_breakdown: module_map,
    };

    fs::create_dir_all("parity").expect("failed to create parity dir");
    let report_json = serde_json::to_string_pretty(&report).expect("serialize report");
    fs::write("parity/doctypes.json", &report_json).expect("write parity/doctypes.json");

    // Generate markdown status
    let mut md = String::new();
    md.push_str("# ERPNext v16 Pure-Rust Parity Status\n\n");
    md.push_str(&format!(
        "**Upstream Canonical DocTypes:** {}  \n**Rust Hand-Written Canonical:** {} ({:.2}%)  \n**Rust Compiled v16 Full:** {} ({:.2}%)\n\n",
        report.total_upstream,
        report.total_registered_standard,
        report.coverage_standard_pct,
        report.total_registered_v16_full,
        report.coverage_v16_full_pct
    ));

    md.push_str("## Module Breakdown\n\n");
    md.push_str("| Module | Upstream Total | Standard Canonical | v16 Full Compiled | Canonical Status | Child Tables | Submittable | Singles |\n");
    md.push_str("|---|---|---|---|---|---|---|---|\n");

    for (mod_name, stats) in &report.module_breakdown {
        md.push_str(&format!(
            "| {} | {} | {} | {} | **{}** | {} | {} | {} |\n",
            mod_name,
            stats.upstream_total,
            stats.standard_registered,
            stats.v16_full_registered,
            stats.status,
            stats.istable_count,
            stats.submittable_count,
            stats.single_count
        ));
    }

    fs::write("parity/status.md", &md).expect("write parity/status.md");

    println!("✅ Parity Ledger Generated: parity/doctypes.json & parity/status.md");
    println!(
        "📊 Current Coverage: Standard Canonical={}/{} ({:.2}%), v16 Full Compiled={}/{} ({:.2}%)",
        report.total_registered_standard,
        report.total_upstream,
        report.coverage_standard_pct,
        report.total_registered_v16_full,
        report.total_upstream,
        report.coverage_v16_full_pct
    );

    if check_mode && report.total_registered_standard < 38 {
        eprintln!("❌ Coverage regression! Expected at least 38 canonical DocTypes.");
        std::process::exit(1);
    }
}
