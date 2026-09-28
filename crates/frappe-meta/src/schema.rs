use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Schema validation and compilation errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SchemaError {
    /// Invalid field name violating naming conventions.
    #[error("Invalid fieldname '{0}': must match ^[a-z_][a-z0-9_]{{0,62}}$")]
    InvalidFieldName(String),
    /// Field name is a reserved keyword in SurrealQL / SQL.
    #[error("Fieldname '{0}' is a reserved keyword")]
    ReservedKeyword(String),
    /// Invalid link target DocType.
    #[error("Link field '{0}' specifies empty target doctype")]
    InvalidLinkTarget(String),
    /// Invalid child table DocType.
    #[error("Table field '{0}' specifies empty child doctype")]
    InvalidChildTable(String),
    /// Duplicate field name in DocType schema.
    #[error("Duplicate fieldname '{0}' in doctype '{1}'")]
    DuplicateFieldName(String, String),
    /// Cyclic class inheritance detected.
    #[error("Cyclic inheritance detected in DocType '{0}': extends '{1}'")]
    CyclicInheritance(String, String),
    /// Base class not found for inheritance.
    #[error("Base DocType class '{0}' not found for '{1}'")]
    BaseClassNotFound(String, String),
    /// Compilation error.
    #[error("Compilation failed: {0}")]
    CompilationFailed(String),
}

/// Specialized presets for `Data` fields.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DataPreset {
    Email,
    Name,
    Phone,
    Url,
    Barcode,
    Iban,
}

/// Naming generation rules supported by the DocType engine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "value")]
pub enum NamingRule {
    Autoincrement,
    SetByUser,
    ByField(String),
    Series(String),
    Expression(String),
}

impl Default for NamingRule {
    fn default() -> Self {
        Self::SetByUser
    }
}

/// Universal Field Types matching all 40+ Frappe Framework field types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "options")]
pub enum FieldType {
    // Text & String Primitives
    Data,
    DataWithPreset { preset: DataPreset },
    LongText,
    SmallText, // Bounded 250 characters
    Text,
    ReadOnly,
    Password, // Argon2id envelope encrypted at rest

    // Relational Primitives
    Link { target_doctype: String },
    DynamicLink { link_field: String },
    Table { child_doctype: String },
    TableMultiSelect { child_doctype: String },

    // Numeric & Financial Primitives
    Int,
    Float,
    Currency,
    Percent,

    // Temporal Primitives
    Date,
    Datetime,
    Time,
    Duration,
    SmartDuration,

    // Code & Structured Markup
    Code { language: Option<String> },
    TextEditor,
    HtmlEditor,
    MarkdownEditor,
    Json,

    // Media & Physical Assets
    Attach,
    AttachImage,
    AttachmentGallery,
    Image,
    Barcode,
    Signature,
    Color,
    Geolocation,
    GeoPoint,

    // Form Layout Primitives
    SectionBreak { collapsible: bool },
    ColumnBreak,
    TabBreak,

    // Behavioral Controls
    Check,
    Select { options: Vec<String> },
    Button,
    Rating,
}

impl FieldType {
    /// Returns the corresponding SurrealQL field type string.
    #[must_use]
    pub fn surreal_type(&self) -> String {
        match self {
            Self::Data
            | Self::DataWithPreset { .. }
            | Self::Text
            | Self::LongText
            | Self::SmallText
            | Self::ReadOnly
            | Self::Password
            | Self::Barcode
            | Self::TextEditor
            | Self::HtmlEditor
            | Self::MarkdownEditor
            | Self::Signature
            | Self::Color => "string".into(),
            Self::Code { .. } => "string".into(),
            Self::Int => "int".into(),
            Self::Float | Self::Percent => "float".into(),
            Self::Currency => "decimal".into(),
            Self::Check => "bool".into(),
            Self::Date | Self::Datetime => "datetime".into(),
            Self::Time => "string".into(),
            Self::Duration | Self::SmartDuration => "duration".into(),
            Self::GeoPoint | Self::Geolocation => "geometry::point".into(),
            Self::Attach | Self::AttachImage | Self::Image => "record<tab_file>".into(),
            Self::AttachmentGallery => "array<record<tab_file>>".into(),
            Self::Json => "object".into(),
            Self::Rating => "int".into(),
            Self::Link { target_doctype } => {
                let sanitized = target_doctype.to_lowercase().replace(' ', "_");
                format!("record<{sanitized}>")
            }
            Self::DynamicLink { .. } => "record".into(),
            Self::Table { child_doctype } | Self::TableMultiSelect { child_doctype } => {
                let sanitized = child_doctype.to_lowercase().replace(' ', "_");
                format!("array<record<{sanitized}>>")
            }
            Self::Select { .. } => "string".into(),
            Self::SectionBreak { .. } | Self::ColumnBreak | Self::TabBreak | Self::Button => "none".into(),
        }
    }

    /// Returns the corresponding SQLite column type string.
    #[must_use]
    pub fn sqlite_type(&self) -> &'static str {
        match self {
            Self::Int | Self::Check | Self::Rating => "INTEGER",
            Self::Float | Self::Percent => "REAL",
            Self::Currency => "TEXT", // Fixed-point decimal string representation
            Self::Date | Self::Datetime | Self::Time | Self::Duration | Self::SmartDuration => "TEXT",
            Self::SectionBreak { .. } | Self::ColumnBreak | Self::TabBreak | Self::Button => "NONE",
            _ => "TEXT",
        }
    }

    /// Returns true if this is a layout primitive that doesn't persist columns in the database.
    #[must_use]
    pub fn is_layout_field(&self) -> bool {
        matches!(
            self,
            Self::SectionBreak { .. } | Self::ColumnBreak | Self::TabBreak | Self::Button
        )
    }
}

/// Role permission specification for DocType access control.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DocPermSchema {
    /// Role identifier permitted.
    pub role: String,
    /// Read permission flag.
    #[serde(default)]
    pub read: bool,
    /// Write / Edit permission flag.
    #[serde(default)]
    pub write: bool,
    /// Create new document flag.
    #[serde(default)]
    pub create: bool,
    /// Delete document flag.
    #[serde(default)]
    pub delete: bool,
    /// Submit document flag.
    #[serde(default)]
    pub submit: bool,
    /// Cancel document flag.
    #[serde(default)]
    pub cancel: bool,
    /// Amend cancelled document flag.
    #[serde(default)]
    pub amend: bool,
    /// View reports flag.
    #[serde(default)]
    pub report: bool,
    /// Export data flag.
    #[serde(default)]
    pub export: bool,
    /// Import data flag.
    #[serde(default)]
    pub import: bool,
    /// Permission level ceiling (0 = document level, 1+ = field level).
    #[serde(default)]
    pub permlevel: u8,
    /// Exemption flag allowing bypass of closed accounting periods.
    #[serde(default)]
    pub accounting_period_exempt: bool,
}

/// Structural schema definition for an individual DocField.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocFieldSchema {
    /// Programmatic field identifier.
    pub fieldname: String,
    /// Logical field data type.
    pub fieldtype: FieldType,
    /// Human readable UI label.
    pub label: String,
    /// Is this field mandatory for document submission/saving?
    #[serde(default)]
    pub reqd: bool,
    /// Must this field be unique across all records?
    #[serde(default)]
    pub unique: bool,
    /// Is this field read-only in the UI and API?
    #[serde(default)]
    pub read_only: bool,
    /// Is this field hidden from standard view?
    #[serde(default)]
    pub hidden: bool,
    /// Should this field appear in the default list view?
    #[serde(default)]
    pub in_list_view: bool,
    /// Role-based masking attribute (displays as masked unless explicit role granted).
    #[serde(default)]
    pub mask: bool,
    /// Dynamic options or select values.
    #[serde(default)]
    pub options: Option<String>,
    /// Default fallback value.
    #[serde(default)]
    pub default_value: Option<serde_json::Value>,
    /// Permlevel for field-level access control (0-9).
    #[serde(default)]
    pub permlevel: u8,
}

/// Abstract Syntax Tree (AST) definition for an entire DocType schema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocTypeSchema {
    /// DocType name (e.g. "Sales Invoice", "Customer", "Account").
    pub name: String,
    /// Application module namespace (e.g. "Accounts", "Stock").
    pub module: String,
    /// Is this a singleton configuration document?
    #[serde(default)]
    pub is_single: bool,
    /// Can documents of this type be submitted and posted to ledgers?
    #[serde(default)]
    pub is_submittable: bool,
    /// Is this a child table embedded in parent documents?
    #[serde(default)]
    pub is_child_table: bool,
    /// Is this a hierarchical Nested Set tree (managed with `lft` and `rgt`)?
    #[serde(default)]
    pub is_tree: bool,
    /// Track historical bitemporal version changes?
    #[serde(default = "default_true")]
    pub track_changes: bool,
    /// Enable quick entry modal dialog?
    #[serde(default)]
    pub quick_entry: bool,
    /// Allow renaming document primary keys?
    #[serde(default)]
    pub allow_rename: bool,
    /// Allow bulk importing data?
    #[serde(default = "default_true")]
    pub allow_import: bool,
    /// Allow recurring auto-repeat execution?
    #[serde(default)]
    pub allow_auto_repeat: bool,
    /// Naming rule generation specification.
    #[serde(default)]
    pub naming_rule: Option<String>,
    /// Structured naming rule enum.
    #[serde(default)]
    pub naming_rule_spec: Option<NamingRule>,
    /// Enable virtual child tables (lazy in-memory views)?
    #[serde(default)]
    pub virtual_child_tables: bool,
    /// Enable lazy document materialization (`get_lazy_doc()`)?
    #[serde(default)]
    pub lazy_materialization: bool,
    /// Parent class inheritance name (for class overrides).
    #[serde(default)]
    pub extends_class: Option<String>,
    /// Field definitions.
    pub fields: Vec<DocFieldSchema>,
    /// Security permission definitions.
    #[serde(default)]
    pub permissions: Vec<DocPermSchema>,
}

fn default_true() -> bool {
    true
}

const RESERVED_KEYWORDS: &[&str] = &[
    "table", "select", "delete", "record", "type", "id", "from", "where", "insert", "update",
    "remove", "alter", "create", "drop", "define", "begin", "commit", "cancel", "transaction",
    "return", "let", "if", "else", "then", "end",
];

impl DocTypeSchema {
    /// Validates the DocType schema AST according to strict architectural invariants.
    pub fn validate(&self) -> Result<(), SchemaError> {
        let field_regex = Regex::new(r"^[a-z_][a-z0-9_]{0,62}$")
            .map_err(|e| SchemaError::CompilationFailed(e.to_string()))?;

        let mut seen_fields = std::collections::HashSet::new();

        for field in &self.fields {
            if field.fieldtype.is_layout_field() {
                continue;
            }

            if !field_regex.is_match(&field.fieldname) {
                return Err(SchemaError::InvalidFieldName(field.fieldname.clone()));
            }

            if RESERVED_KEYWORDS.contains(&field.fieldname.to_lowercase().as_str()) {
                return Err(SchemaError::ReservedKeyword(field.fieldname.clone()));
            }

            if !seen_fields.insert(&field.fieldname) {
                return Err(SchemaError::DuplicateFieldName(
                    field.fieldname.clone(),
                    self.name.clone(),
                ));
            }

            match &field.fieldtype {
                FieldType::Link { target_doctype } if target_doctype.trim().is_empty() => {
                    return Err(SchemaError::InvalidLinkTarget(field.fieldname.clone()));
                }
                FieldType::Table { child_doctype }
                | FieldType::TableMultiSelect { child_doctype }
                    if child_doctype.trim().is_empty() =>
                {
                    return Err(SchemaError::InvalidChildTable(field.fieldname.clone()));
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Returns the database table name derived from the DocType name.
    #[must_use]
    pub fn table_name(&self) -> String {
        self.name.to_lowercase().replace(' ', "_")
    }

    /// Returns true if this DocType contains tree traversal fields `lft` and `rgt`.
    #[must_use]
    pub fn is_tree_structure(&self) -> bool {
        self.is_tree
    }
}
