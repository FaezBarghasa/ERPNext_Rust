use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Schema validation and compilation errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SchemaError {
    /// Invalid field name violating naming conventions.
    #[error("Invalid fieldname '{0}': must match ^[a-z_][a-z0-9_]{{0,62}}$")]
    InvalidFieldName(String),
    /// Field name is a reserved keyword in SurrealQL.
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
    /// Compilation error.
    #[error("SurrealQL compilation failed: {0}")]
    CompilationFailed(String),
}

/// Primitives for DocType field types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "options")]
pub enum FieldType {
    /// Standard single-line text string.
    Data,
    /// 64-bit integer.
    Int,
    /// 64-bit floating point number.
    Float,
    /// Fixed-point financial currency.
    Currency,
    /// Boolean flag.
    Check,
    /// Foreign key reference to another DocType.
    Link { target_doctype: String },
    /// Dynamic foreign key referencing another field for DocType name.
    DynamicLink { link_field: String },
    /// Nested child table association.
    Table { child_doctype: String },
    /// Multi-option select list.
    Select { options: Vec<String> },
    /// Multi-line text field.
    Text,
    /// Unbounded long text field.
    LongText,
    /// Code or script block.
    Code,
    /// Calendar date (YYYY-MM-DD).
    Date,
    /// Timestamp (ISO 8601).
    Datetime,
    /// Percentage value.
    Percent,
    /// Numeric rating score.
    Rating,
    /// Time duration.
    Duration,
    /// Geographic coordinates.
    GeoPoint,
    /// Encrypted password string.
    Password,
    /// File attachment record.
    Attach,
    /// Arbitrary JSON object.
    Json,
    /// Barcode or QR string.
    Barcode,
}

impl FieldType {
    /// Returns the corresponding SurrealQL field type string.
    #[must_use]
    pub fn surreal_type(&self) -> String {
        match self {
            Self::Data
            | Self::Text
            | Self::LongText
            | Self::Code
            | Self::Password
            | Self::Barcode => "string".into(),
            Self::Int => "int".into(),
            Self::Float | Self::Percent => "float".into(),
            Self::Currency => "decimal".into(),
            Self::Check => "bool".into(),
            Self::Date | Self::Datetime => "datetime".into(),
            Self::Duration => "duration".into(),
            Self::GeoPoint => "geometry::point".into(),
            Self::Attach => "record<tab_file>".into(),
            Self::Json => "object".into(),
            Self::Rating => "int".into(),
            Self::Link { target_doctype } => {
                let sanitized = target_doctype.to_lowercase().replace(' ', "_");
                format!("record<{sanitized}>")
            }
            Self::DynamicLink { .. } => "record".into(),
            Self::Table { child_doctype } => {
                let sanitized = child_doctype.to_lowercase().replace(' ', "_");
                format!("array<record<{sanitized}>>")
            }
            Self::Select { .. } => "string".into(),
        }
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
    /// Permission level ceiling (0 = document level, 1+ = field level).
    #[serde(default)]
    pub permlevel: u8,
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
    /// Dynamic options or select values.
    #[serde(default)]
    pub options: Option<String>,
    /// Default fallback value.
    #[serde(default)]
    pub default_value: Option<serde_json::Value>,
}

/// Abstract Syntax Tree (AST) definition for an entire DocType schema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocTypeSchema {
    /// DocType name (e.g. "Sales Invoice", "Customer").
    pub name: String,
    /// Application module namespace (e.g. "Accounts", "Stock").
    pub module: String,
    /// Is this a singleton configuration document?
    #[serde(default)]
    pub is_single: bool,
    /// Can documents of this type be submitted and posted to ledgers?
    #[serde(default)]
    pub is_submittable: bool,
    /// Track historical version changes?
    #[serde(default)]
    pub track_changes: bool,
    /// Naming rule template (e.g. "ACC-INV-.YYYY.-.#####").
    #[serde(default)]
    pub naming_rule: Option<String>,
    /// Field definitions.
    pub fields: Vec<DocFieldSchema>,
    /// Security permission definitions.
    #[serde(default)]
    pub permissions: Vec<DocPermSchema>,
}

const RESERVED_KEYWORDS: &[&str] = &[
    "table",
    "select",
    "delete",
    "record",
    "type",
    "id",
    "from",
    "where",
    "insert",
    "update",
    "remove",
    "alter",
    "create",
    "drop",
    "define",
    "begin",
    "commit",
    "cancel",
    "transaction",
    "return",
    "let",
    "if",
    "else",
    "then",
    "end",
];

impl DocTypeSchema {
    /// Validates the DocType schema AST according to strict architectural invariants.
    pub fn validate(&self) -> Result<(), SchemaError> {
        let field_regex = Regex::new(r"^[a-z_][a-z0-9_]{0,62}$")
            .map_err(|e| SchemaError::CompilationFailed(e.to_string()))?;

        let mut seen_fields = std::collections::HashSet::new();

        for field in &self.fields {
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
                FieldType::Table { child_doctype } if child_doctype.trim().is_empty() => {
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
}
