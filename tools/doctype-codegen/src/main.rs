use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
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
    pub depends_on: Option<String>,
    pub mandatory_depends_on: Option<String>,
    pub read_only_depends_on: Option<String>,
    pub fetch_from: Option<String>,
    pub precision: Option<serde_json::Value>,
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
    pub image_field: Option<String>,
    #[serde(default)]
    pub track_changes: Option<i64>,
    #[serde(default)]
    pub quick_entry: Option<i64>,
    #[serde(default)]
    pub allow_rename: Option<i64>,
    #[serde(default)]
    pub allow_import: Option<i64>,
    #[serde(default)]
    pub allow_auto_repeat: Option<i64>,
    #[serde(default)]
    pub fields: Vec<UpstreamDocField>,
}

fn scan_repo(repo_path: &Path) -> Vec<(PathBuf, UpstreamDocType)> {
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

fn map_field_type(ft: &str, options: Option<&str>) -> String {
    match ft {
        "Data" => "FieldType::Data".to_string(),
        "Link" => {
            let target = options.unwrap_or("").replace('"', "\\\"");
            format!("FieldType::Link {{ target_doctype: \"{target}\".into() }}")
        }
        "Dynamic Link" => {
            let link = options.unwrap_or("").replace('"', "\\\"");
            format!("FieldType::DynamicLink {{ link_field: \"{link}\".into() }}")
        }
        "Table" => {
            let child = options.unwrap_or("").replace('"', "\\\"");
            format!("FieldType::Table {{ child_doctype: \"{child}\".into() }}")
        }
        "Table MultiSelect" => {
            let child = options.unwrap_or("").replace('"', "\\\"");
            format!("FieldType::TableMultiSelect {{ child_doctype: \"{child}\".into() }}")
        }
        "Select" => {
            let opts = options.unwrap_or("");
            let variants: Vec<String> = opts
                .lines()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| format!("\"{}\".into()", s.replace('"', "\\\"")))
                .collect();
            format!(
                "FieldType::Select {{ options: vec![{}] }}",
                variants.join(", ")
            )
        }
        "Int" => "FieldType::Int".to_string(),
        "Float" => "FieldType::Float".to_string(),
        "Currency" => "FieldType::Currency".to_string(),
        "Percent" => "FieldType::Percent".to_string(),
        "Check" => "FieldType::Check".to_string(),
        "Date" => "FieldType::Date".to_string(),
        "Datetime" => "FieldType::Datetime".to_string(),
        "Time" => "FieldType::Time".to_string(),
        "Duration" => "FieldType::Duration".to_string(),
        "Small Text" => "FieldType::SmallText".to_string(),
        "Text" => "FieldType::Text".to_string(),
        "Long Text" => "FieldType::LongText".to_string(),
        "Code" => "FieldType::Code { language: None }".to_string(),
        "Text Editor" => "FieldType::TextEditor".to_string(),
        "HTML Editor" => "FieldType::HtmlEditor".to_string(),
        "Markdown Editor" => "FieldType::MarkdownEditor".to_string(),
        "JSON" | "Json" => "FieldType::Json".to_string(),
        "Attach" => "FieldType::Attach".to_string(),
        "Attach Image" => "FieldType::AttachImage".to_string(),
        "Rating" => "FieldType::Rating".to_string(),
        "Password" => "FieldType::Password".to_string(),
        "Read Only" => "FieldType::ReadOnly".to_string(),
        "Section Break" => "FieldType::SectionBreak { collapsible: false }".to_string(),
        "Column Break" => "FieldType::ColumnBreak".to_string(),
        "Tab Break" => "FieldType::TabBreak".to_string(),
        "Button" => "FieldType::Button".to_string(),
        "Color" => "FieldType::Color".to_string(),
        "Geolocation" => "FieldType::Geolocation".to_string(),
        _ => "FieldType::Data".to_string(),
    }
}

fn sanitize_ident(s: &str) -> String {
    let lower = s.to_lowercase().replace([' ', '-', '.'], "_");
    let mut ident = String::new();
    for c in lower.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            ident.push(c);
        }
    }
    if ident.is_empty() || ident.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        ident = format!("dt_{ident}");
    }
    ident
}

fn format_opt_str(val: &Option<String>) -> String {
    match val {
        Some(s) => format!(
            "Some(\"{}\".into())",
            s.replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
        ),
        None => "None".to_string(),
    }
}

fn generate_doctype_code(dt: &UpstreamDocType) -> String {
    let fn_name = format!("schema_{}", sanitize_ident(&dt.name));
    let dt_name = dt.name.replace('"', "\\\"");
    let module = dt.module.replace('"', "\\\"");

    let mut fields_code = String::new();
    let mut seen_fields = BTreeSet::new();

    for f in &dt.fields {
        let raw_fname = f.fieldname.as_deref().unwrap_or("").trim();
        if raw_fname.is_empty() || !seen_fields.insert(raw_fname.to_string()) {
            continue;
        }

        let fname_sanitized = raw_fname.to_lowercase().replace(['-', ' ', '.'], "_");
        if fname_sanitized.is_empty() {
            continue;
        }

        let ftype = map_field_type(
            f.fieldtype.as_deref().unwrap_or("Data"),
            f.options.as_deref(),
        );

        let label = f.label.as_deref().unwrap_or(raw_fname).replace('"', "\\\"");
        let reqd = f.reqd == Some(1);
        let hidden = f.hidden == Some(1);
        let read_only = f.read_only == Some(1);
        let unique = f.unique == Some(1);
        let in_list_view = f.in_list_view == Some(1);
        let permlevel = f.permlevel.unwrap_or(0).clamp(0, 9);

        let options_str = format_opt_str(&f.options);
        let depends_on_str = format_opt_str(&f.depends_on);
        let mandatory_depends_on_str = format_opt_str(&f.mandatory_depends_on);
        let read_only_depends_on_str = format_opt_str(&f.read_only_depends_on);
        let fetch_from_str = format_opt_str(&f.fetch_from);

        let precision_str = match &f.precision {
            Some(serde_json::Value::Number(n)) => format!("Some({})", n.as_u64().unwrap_or(2)),
            Some(serde_json::Value::String(s)) => {
                if let Ok(n) = s.parse::<u8>() {
                    format!("Some({n})")
                } else {
                    "None".to_string()
                }
            }
            _ => "None".to_string(),
        };

        fields_code.push_str(&format!(
            "            DocFieldSchema {{\n                fieldname: \"{fname_sanitized}\".into(),\n                fieldtype: {ftype},\n                label: \"{label}\".into(),\n                reqd: {reqd},\n                unique: {unique},\n                read_only: {read_only},\n                hidden: {hidden},\n                in_list_view: {in_list_view},\n                mask: false,\n                options: {options_str},\n                default_value: None,\n                permlevel: {permlevel},\n                depends_on: {depends_on_str},\n                mandatory_depends_on: {mandatory_depends_on_str},\n                read_only_depends_on: {read_only_depends_on_str},\n                fetch_from: {fetch_from_str},\n                precision: {precision_str},\n            }},\n"
        ));
    }

    let is_table = dt.istable == Some(1);
    let is_submittable = dt.is_submittable == Some(1);
    let is_single = dt.issingle == Some(1);
    let is_tree = dt.is_tree == Some(1);
    let track_changes = dt.track_changes != Some(0);
    let quick_entry = dt.quick_entry == Some(1);
    let allow_rename = dt.allow_rename == Some(1);
    let allow_import = dt.allow_import != Some(0);
    let allow_auto_repeat = dt.allow_auto_repeat == Some(1);

    let autoname_str = format_opt_str(&dt.autoname);
    let title_field_str = format_opt_str(&dt.title_field);
    let search_fields_str = format_opt_str(&dt.search_fields);
    let image_field_str = format_opt_str(&dt.image_field);

    format!(
        r#"/// Auto-generated schema for `{dt_name}`
pub fn {fn_name}() -> DocTypeSchema {{
    DocTypeSchema {{
        name: "{dt_name}".into(),
        module: "{module}".into(),
        is_single: {is_single},
        is_submittable: {is_submittable},
        is_child_table: {is_table},
        is_tree: {is_tree},
        track_changes: {track_changes},
        quick_entry: {quick_entry},
        allow_rename: {allow_rename},
        allow_import: {allow_import},
        allow_auto_repeat: {allow_auto_repeat},
        naming_rule: {autoname_str},
        naming_rule_spec: None,
        virtual_child_tables: false,
        lazy_materialization: false,
        extends_class: None,
        title_field: {title_field_str},
        search_fields: {search_fields_str},
        image_field: {image_field_str},
        fields: vec![
{fields_code}        ],
        permissions: Vec::new(),
    }}
}}
"#
    )
}

fn main() {
    let erpnext_path = Path::new("third_party/erpnext/erpnext");
    let frappe_path = Path::new("third_party/frappe/frappe");

    println!("🚀 Starting ERPNext v16 DocType Code Generator...");

    let mut doctypes = scan_repo(erpnext_path);
    doctypes.extend(scan_repo(frappe_path));
    doctypes.sort_by(|a, b| a.1.name.cmp(&b.1.name));

    let mut dedup_names = BTreeSet::new();
    let mut generated_functions = Vec::new();
    let mut schema_call_list = Vec::new();

    let out_dir = Path::new("crates/frappe-meta/src/generated");
    fs::create_dir_all(out_dir).expect("create generated dir");

    let mut total_generated = 0;

    for (_path, dt) in &doctypes {
        if !dedup_names.insert(dt.name.clone()) {
            continue;
        }

        let code = generate_doctype_code(dt);
        generated_functions.push(code);

        let fn_name = format!("schema_{}", sanitize_ident(&dt.name));
        schema_call_list.push(format!("        {fn_name}(),"));
        total_generated += 1;
    }

    let mut out_file_content = String::new();
    out_file_content
        .push_str("//! Auto-generated canonical ERPNext v16 & Frappe v16 DocType schemas.\n");
    out_file_content
        .push_str("//! Generated by `tools/doctype-codegen`. Do not edit manually.\n\n");
    out_file_content.push_str("use crate::schema::{DocFieldSchema, DocTypeSchema, FieldType};\n\n");

    out_file_content
        .push_str("/// Returns all 800+ canonical compiled schemas for ERPNext & Frappe v16.\n");
    out_file_content.push_str("pub fn all_v16_schemas() -> Vec<DocTypeSchema> {\n");
    out_file_content.push_str("    vec![\n");
    for call in &schema_call_list {
        out_file_content.push_str(call);
        out_file_content.push('\n');
    }
    out_file_content.push_str("    ]\n}\n\n");

    for f_code in &generated_functions {
        out_file_content.push_str(f_code);
        out_file_content.push('\n');
    }

    let target_path = out_dir.join("v16_schemas.rs");
    fs::write(&target_path, &out_file_content).expect("write v16_schemas.rs");

    println!(
        "✅ Successfully generated {} DocTypes into {}",
        total_generated,
        target_path.display()
    );
}
