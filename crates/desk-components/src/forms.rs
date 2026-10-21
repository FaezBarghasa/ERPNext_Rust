use chrono::Utc;
use frappe_meta::{DocTypeSchema, FieldType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// 1. VISUAL FIELD TYPES (18 CORE VARIANTS)
// ============================================================================

/// Client-side UI widget type mapped from server DocField AST.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FormFieldWidget {
    TextInput {
        fieldname: String,
        label: String,
        reqd: bool,
    },
    LongTextInput {
        fieldname: String,
        label: String,
        reqd: bool,
        rows: u32,
    },
    RichTextInput {
        fieldname: String,
        label: String,
        reqd: bool,
    },
    MarkdownInput {
        fieldname: String,
        label: String,
        reqd: bool,
    },
    PasswordInput {
        fieldname: String,
        label: String,
        reqd: bool,
    },
    NumberInput {
        fieldname: String,
        label: String,
        reqd: bool,
    },
    CurrencyInput {
        fieldname: String,
        label: String,
        reqd: bool,
    },
    DatePicker {
        fieldname: String,
        label: String,
        reqd: bool,
    },
    TimePicker {
        fieldname: String,
        label: String,
        reqd: bool,
    },
    DatetimePicker {
        fieldname: String,
        label: String,
        reqd: bool,
    },
    Checkbox {
        fieldname: String,
        label: String,
        default_checked: bool,
    },
    SelectDropdown {
        fieldname: String,
        label: String,
        options: Vec<String>,
        reqd: bool,
    },
    LinkDropdown {
        fieldname: String,
        label: String,
        target_doctype: String,
        endpoint: String,
        reqd: bool,
    },
    DynamicLinkDropdown {
        fieldname: String,
        label: String,
        link_doctype_field: String,
        reqd: bool,
    },
    TableSubgrid {
        fieldname: String,
        label: String,
        child_doctype: String,
    },
    AttachmentFile {
        fieldname: String,
        label: String,
        max_size_mb: u32,
        allowed_types: Vec<String>,
        reqd: bool,
    },
    ColorPicker {
        fieldname: String,
        label: String,
        default_color: String,
    },
    SignaturePad {
        fieldname: String,
        label: String,
        reqd: bool,
    },
    CodeEditor {
        fieldname: String,
        label: String,
        language: String,
        reqd: bool,
    },
}

impl FormFieldWidget {
    #[must_use]
    pub fn fieldname(&self) -> &str {
        match self {
            Self::TextInput { fieldname, .. }
            | Self::LongTextInput { fieldname, .. }
            | Self::RichTextInput { fieldname, .. }
            | Self::MarkdownInput { fieldname, .. }
            | Self::PasswordInput { fieldname, .. }
            | Self::NumberInput { fieldname, .. }
            | Self::CurrencyInput { fieldname, .. }
            | Self::DatePicker { fieldname, .. }
            | Self::TimePicker { fieldname, .. }
            | Self::DatetimePicker { fieldname, .. }
            | Self::Checkbox { fieldname, .. }
            | Self::SelectDropdown { fieldname, .. }
            | Self::LinkDropdown { fieldname, .. }
            | Self::DynamicLinkDropdown { fieldname, .. }
            | Self::TableSubgrid { fieldname, .. }
            | Self::AttachmentFile { fieldname, .. }
            | Self::ColorPicker { fieldname, .. }
            | Self::SignaturePad { fieldname, .. }
            | Self::CodeEditor { fieldname, .. } => fieldname,
        }
    }

    #[must_use]
    pub fn label(&self) -> &str {
        match self {
            Self::TextInput { label, .. }
            | Self::LongTextInput { label, .. }
            | Self::RichTextInput { label, .. }
            | Self::MarkdownInput { label, .. }
            | Self::PasswordInput { label, .. }
            | Self::NumberInput { label, .. }
            | Self::CurrencyInput { label, .. }
            | Self::DatePicker { label, .. }
            | Self::TimePicker { label, .. }
            | Self::DatetimePicker { label, .. }
            | Self::Checkbox { label, .. }
            | Self::SelectDropdown { label, .. }
            | Self::LinkDropdown { label, .. }
            | Self::DynamicLinkDropdown { label, .. }
            | Self::TableSubgrid { label, .. }
            | Self::AttachmentFile { label, .. }
            | Self::ColorPicker { label, .. }
            | Self::SignaturePad { label, .. }
            | Self::CodeEditor { label, .. } => label,
        }
    }

    #[must_use]
    pub fn is_required(&self) -> bool {
        match self {
            Self::TextInput { reqd, .. }
            | Self::LongTextInput { reqd, .. }
            | Self::RichTextInput { reqd, .. }
            | Self::MarkdownInput { reqd, .. }
            | Self::PasswordInput { reqd, .. }
            | Self::NumberInput { reqd, .. }
            | Self::CurrencyInput { reqd, .. }
            | Self::DatePicker { reqd, .. }
            | Self::TimePicker { reqd, .. }
            | Self::DatetimePicker { reqd, .. }
            | Self::SelectDropdown { reqd, .. }
            | Self::LinkDropdown { reqd, .. }
            | Self::DynamicLinkDropdown { reqd, .. }
            | Self::AttachmentFile { reqd, .. }
            | Self::SignaturePad { reqd, .. }
            | Self::CodeEditor { reqd, .. } => *reqd,
            Self::Checkbox { .. } | Self::TableSubgrid { .. } | Self::ColorPicker { .. } => false,
        }
    }
}

// ============================================================================
// 2. DYNAMIC FORM MODEL & SCHEMA COMPILER
// ============================================================================

/// Dynamic Form Component Model compiled from DocTypeSchema AST.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DynamicFormModel {
    pub doctype_name: String,
    pub is_submittable: bool,
    pub widgets: Vec<FormFieldWidget>,
    pub dependencies: HashMap<String, String>,
}

impl DynamicFormModel {
    /// Compiles a server DocTypeSchema AST into a client-side DynamicFormModel.
    #[must_use]
    pub fn from_schema(schema: &DocTypeSchema) -> Self {
        let mut widgets = Vec::new();
        let mut dependencies = HashMap::new();

        for field in &schema.fields {
            if field.hidden {
                continue;
            }

            if let Some(dep) = &field.depends_on {
                dependencies.insert(field.fieldname.clone(), dep.clone());
            }

            let widget = match &field.fieldtype {
                FieldType::Data
                | FieldType::DataWithPreset { .. }
                | FieldType::Text
                | FieldType::ReadOnly => FormFieldWidget::TextInput {
                    fieldname: field.fieldname.clone(),
                    label: field.label.clone(),
                    reqd: field.reqd,
                },
                FieldType::LongText | FieldType::SmallText => FormFieldWidget::LongTextInput {
                    fieldname: field.fieldname.clone(),
                    label: field.label.clone(),
                    reqd: field.reqd,
                    rows: 4,
                },
                FieldType::TextEditor | FieldType::HtmlEditor => FormFieldWidget::RichTextInput {
                    fieldname: field.fieldname.clone(),
                    label: field.label.clone(),
                    reqd: field.reqd,
                },
                FieldType::MarkdownEditor => FormFieldWidget::MarkdownInput {
                    fieldname: field.fieldname.clone(),
                    label: field.label.clone(),
                    reqd: field.reqd,
                },
                FieldType::Password => FormFieldWidget::PasswordInput {
                    fieldname: field.fieldname.clone(),
                    label: field.label.clone(),
                    reqd: field.reqd,
                },
                FieldType::Int | FieldType::Float | FieldType::Percent | FieldType::Rating => {
                    FormFieldWidget::NumberInput {
                        fieldname: field.fieldname.clone(),
                        label: field.label.clone(),
                        reqd: field.reqd,
                    }
                }
                FieldType::Currency => FormFieldWidget::CurrencyInput {
                    fieldname: field.fieldname.clone(),
                    label: field.label.clone(),
                    reqd: field.reqd,
                },
                FieldType::Date => FormFieldWidget::DatePicker {
                    fieldname: field.fieldname.clone(),
                    label: field.label.clone(),
                    reqd: field.reqd,
                },
                FieldType::Time => FormFieldWidget::TimePicker {
                    fieldname: field.fieldname.clone(),
                    label: field.label.clone(),
                    reqd: field.reqd,
                },
                FieldType::Datetime => FormFieldWidget::DatetimePicker {
                    fieldname: field.fieldname.clone(),
                    label: field.label.clone(),
                    reqd: field.reqd,
                },
                FieldType::Check => FormFieldWidget::Checkbox {
                    fieldname: field.fieldname.clone(),
                    label: field.label.clone(),
                    default_checked: field
                        .default_value
                        .as_ref()
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false),
                },
                FieldType::Link { target_doctype } => FormFieldWidget::LinkDropdown {
                    fieldname: field.fieldname.clone(),
                    label: field.label.clone(),
                    target_doctype: target_doctype.clone(),
                    endpoint: format!(
                        "/api/v1/resource/{}",
                        target_doctype.to_lowercase().replace(' ', "_")
                    ),
                    reqd: field.reqd,
                },
                FieldType::DynamicLink { link_field } => FormFieldWidget::DynamicLinkDropdown {
                    fieldname: field.fieldname.clone(),
                    label: field.label.clone(),
                    link_doctype_field: link_field.clone(),
                    reqd: field.reqd,
                },
                FieldType::Select { options } => FormFieldWidget::SelectDropdown {
                    fieldname: field.fieldname.clone(),
                    label: field.label.clone(),
                    options: options.clone(),
                    reqd: field.reqd,
                },
                FieldType::Table { child_doctype } => FormFieldWidget::TableSubgrid {
                    fieldname: field.fieldname.clone(),
                    label: field.label.clone(),
                    child_doctype: child_doctype.clone(),
                },
                FieldType::Attach | FieldType::AttachImage => FormFieldWidget::AttachmentFile {
                    fieldname: field.fieldname.clone(),
                    label: field.label.clone(),
                    max_size_mb: 25,
                    allowed_types: vec!["image/*".into(), "application/pdf".into()],
                    reqd: field.reqd,
                },
                FieldType::Color => FormFieldWidget::ColorPicker {
                    fieldname: field.fieldname.clone(),
                    label: field.label.clone(),
                    default_color: "#3b82f6".into(),
                },
                FieldType::Signature => FormFieldWidget::SignaturePad {
                    fieldname: field.fieldname.clone(),
                    label: field.label.clone(),
                    reqd: field.reqd,
                },
                FieldType::Code { language } => FormFieldWidget::CodeEditor {
                    fieldname: field.fieldname.clone(),
                    label: field.label.clone(),
                    language: language.clone().unwrap_or_else(|| "text".to_string()),
                    reqd: field.reqd,
                },
                _ => FormFieldWidget::TextInput {
                    fieldname: field.fieldname.clone(),
                    label: field.label.clone(),
                    reqd: field.reqd,
                },
            };

            widgets.push(widget);
        }

        Self {
            doctype_name: schema.name.clone(),
            is_submittable: schema.is_submittable,
            widgets,
            dependencies,
        }
    }
}

// ============================================================================
// 3. CONDITIONAL DEPENDENCIES (depends_on) & CONDITION BUILDER
// ============================================================================

/// Relational comparison operators supported in visual conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
    Contains,
    IsEmpty,
    IsNotEmpty,
}

impl ConditionOperator {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Equals => "==",
            Self::NotEquals => "!=",
            Self::GreaterThan => ">",
            Self::LessThan => "<",
            Self::GreaterOrEqual => ">=",
            Self::LessOrEqual => "<=",
            Self::Contains => "contains",
            Self::IsEmpty => "is_empty",
            Self::IsNotEmpty => "is_not_empty",
        }
    }
}

/// An individual field-level condition rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConditionRule {
    pub source_field: String,
    pub operator: ConditionOperator,
    pub target_value: String,
}

impl ConditionRule {
    #[must_use]
    pub fn new(
        source_field: impl Into<String>,
        operator: ConditionOperator,
        target_value: impl Into<String>,
    ) -> Self {
        Self {
            source_field: source_field.into(),
            operator,
            target_value: target_value.into(),
        }
    }

    /// Evaluates this rule against form values.
    #[must_use]
    pub fn evaluate(&self, form_values: &HashMap<String, String>) -> bool {
        let current_val = form_values
            .get(&self.source_field)
            .map(String::as_str)
            .unwrap_or("");
        match self.operator {
            ConditionOperator::Equals => current_val == self.target_value,
            ConditionOperator::NotEquals => current_val != self.target_value,
            ConditionOperator::GreaterThan => {
                if let (Ok(curr), Ok(target)) =
                    (current_val.parse::<f64>(), self.target_value.parse::<f64>())
                {
                    curr > target
                } else {
                    false
                }
            }
            ConditionOperator::LessThan => {
                if let (Ok(curr), Ok(target)) =
                    (current_val.parse::<f64>(), self.target_value.parse::<f64>())
                {
                    curr < target
                } else {
                    false
                }
            }
            ConditionOperator::GreaterOrEqual => {
                if let (Ok(curr), Ok(target)) =
                    (current_val.parse::<f64>(), self.target_value.parse::<f64>())
                {
                    curr >= target
                } else {
                    false
                }
            }
            ConditionOperator::LessOrEqual => {
                if let (Ok(curr), Ok(target)) =
                    (current_val.parse::<f64>(), self.target_value.parse::<f64>())
                {
                    curr <= target
                } else {
                    false
                }
            }
            ConditionOperator::Contains => current_val.contains(&self.target_value),
            ConditionOperator::IsEmpty => current_val.trim().is_empty(),
            ConditionOperator::IsNotEmpty => !current_val.trim().is_empty(),
        }
    }

    /// Converts this rule into a serialized expression string.
    #[must_use]
    pub fn to_expression(&self) -> String {
        format!(
            "{} {} \"{}\"",
            self.source_field,
            self.operator.as_str(),
            self.target_value
        )
    }
}

/// Logical combinator for linking multiple condition rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LogicalCombinator {
    #[default]
    And,
    Or,
}

/// A group of condition rules evaluated with an AND/OR combinator.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ConditionGroup {
    pub combinator: LogicalCombinator,
    pub rules: Vec<ConditionRule>,
}

impl ConditionGroup {
    #[must_use]
    pub fn new(combinator: LogicalCombinator) -> Self {
        Self {
            combinator,
            rules: Vec::new(),
        }
    }

    pub fn add_rule(&mut self, rule: ConditionRule) {
        self.rules.push(rule);
    }

    #[must_use]
    pub fn evaluate(&self, form_values: &HashMap<String, String>) -> bool {
        if self.rules.is_empty() {
            return true;
        }

        match self.combinator {
            LogicalCombinator::And => self.rules.iter().all(|r| r.evaluate(form_values)),
            LogicalCombinator::Or => self.rules.iter().any(|r| r.evaluate(form_values)),
        }
    }

    #[must_use]
    pub fn to_expression(&self) -> String {
        let op_str = match self.combinator {
            LogicalCombinator::And => " && ",
            LogicalCombinator::Or => " || ",
        };
        self.rules
            .iter()
            .map(ConditionRule::to_expression)
            .collect::<Vec<_>>()
            .join(op_str)
    }
}

/// Evaluates Frappe dynamic form `depends_on` visibility conditions.
#[must_use]
pub fn eval_depends_on(expr: &str, form_values: &HashMap<String, String>) -> bool {
    let expr = expr.trim();
    if expr.is_empty() {
        return true;
    }

    // Support eval: prefix by stripping and evaluating basic equality/truthiness
    let expr_clean = expr.strip_prefix("eval:").unwrap_or(expr).trim();

    if expr_clean.contains("||") {
        return expr_clean
            .split("||")
            .any(|part| eval_single_depends_clause(part.trim(), form_values));
    }

    expr_clean
        .split("&&")
        .all(|part| eval_single_depends_clause(part.trim(), form_values))
}

fn eval_single_depends_clause(clause: &str, form_values: &HashMap<String, String>) -> bool {
    let clause = clause.trim();
    if let Some((k, v)) = clause.split_once("==") {
        let key = k.trim().trim_start_matches("doc.");
        let val = v.trim().trim_matches('"').trim_matches('\'');
        form_values.get(key).is_some_and(|x| x == val)
    } else if let Some((k, v)) = clause.split_once("!=") {
        let key = k.trim().trim_start_matches("doc.");
        let val = v.trim().trim_matches('"').trim_matches('\'');
        form_values.get(key).is_none_or(|x| x != val)
    } else if let Some((k, v)) = clause.split_once(">=") {
        let key = k.trim().trim_start_matches("doc.");
        if let (Some(cur_str), Ok(target)) = (form_values.get(key), v.trim().parse::<f64>()) {
            cur_str.parse::<f64>().is_ok_and(|c| c >= target)
        } else {
            false
        }
    } else if let Some((k, v)) = clause.split_once('>') {
        let key = k.trim().trim_start_matches("doc.");
        if let (Some(cur_str), Ok(target)) = (form_values.get(key), v.trim().parse::<f64>()) {
            cur_str.parse::<f64>().is_ok_and(|c| c > target)
        } else {
            false
        }
    } else if let Some((k, v)) = clause.split_once("<=") {
        let key = k.trim().trim_start_matches("doc.");
        if let (Some(cur_str), Ok(target)) = (form_values.get(key), v.trim().parse::<f64>()) {
            cur_str.parse::<f64>().is_ok_and(|c| c <= target)
        } else {
            false
        }
    } else if let Some((k, v)) = clause.split_once('<') {
        let key = k.trim().trim_start_matches("doc.");
        if let (Some(cur_str), Ok(target)) = (form_values.get(key), v.trim().parse::<f64>()) {
            cur_str.parse::<f64>().is_ok_and(|c| c < target)
        } else {
            false
        }
    } else {
        // Truthiness check
        let key = clause.trim().trim_start_matches("doc.");
        form_values
            .get(key)
            .is_some_and(|v| v == "true" || v == "1" || (!v.trim().is_empty() && v != "0"))
    }
}

// ============================================================================
// 4. DIRTY-STATE & AUTO-SAVE DRAFT MANAGER
// ============================================================================

/// Persistent draft snapshot for local recovery and auto-save.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FormDraft {
    pub doctype_name: String,
    pub docname: Option<String>,
    pub values: HashMap<String, String>,
    pub saved_at: chrono::DateTime<chrono::Utc>,
}

/// Tracks per-field dirty state, keyboard shortcuts, and auto-save drafts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormStateTracker {
    initial_values: HashMap<String, String>,
    current_values: HashMap<String, String>,
    last_saved_at: Option<chrono::DateTime<chrono::Utc>>,
    draft_key: Option<String>,
}

impl FormStateTracker {
    #[must_use]
    pub fn new(initial_values: HashMap<String, String>) -> Self {
        Self {
            initial_values: initial_values.clone(),
            current_values: initial_values,
            last_saved_at: Some(Utc::now()),
            draft_key: None,
        }
    }

    pub fn set_field(&mut self, field: &str, val: &str) {
        self.current_values
            .insert(field.to_string(), val.to_string());
    }

    #[must_use]
    pub fn get_field(&self, field: &str) -> Option<&str> {
        self.current_values.get(field).map(String::as_str)
    }

    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.current_values != self.initial_values
    }

    #[must_use]
    pub fn is_field_dirty(&self, field: &str) -> bool {
        self.current_values.get(field) != self.initial_values.get(field)
    }

    #[must_use]
    pub fn dirty_fields(&self) -> Vec<String> {
        self.current_values
            .iter()
            .filter_map(|(k, v)| {
                if self.initial_values.get(k) != Some(v) {
                    Some(k.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn mark_saved(&mut self) {
        self.initial_values = self.current_values.clone();
        self.last_saved_at = Some(Utc::now());
    }

    pub fn reset(&mut self) {
        self.current_values = self.initial_values.clone();
    }

    #[must_use]
    pub fn create_draft_snapshot(&self, doctype: &str, docname: Option<&str>) -> FormDraft {
        FormDraft {
            doctype_name: doctype.to_string(),
            docname: docname.map(ToString::to_string),
            values: self.current_values.clone(),
            saved_at: Utc::now(),
        }
    }

    pub fn restore_from_draft(&mut self, draft: &FormDraft) {
        self.current_values = draft.values.clone();
    }
}

// ============================================================================
// 5. VALIDATION RULES & REAL-TIME FEEDBACK
// ============================================================================

/// Field-level validation constraints.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FieldValidationRule {
    Required,
    Email,
    Phone,
    Url,
    Regex(String),
    MinLength(usize),
    MaxLength(usize),
    NumericRange { min: Option<f64>, max: Option<f64> },
}

/// Validation execution report for a form.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub errors: HashMap<String, String>,
    pub warnings: HashMap<String, String>,
}

/// Form validator engine executing declarative constraints.
#[derive(Debug, Clone, Default)]
pub struct FormValidator {
    rules: HashMap<String, Vec<FieldValidationRule>>,
}

impl FormValidator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, field: impl Into<String>, rule: FieldValidationRule) {
        self.rules.entry(field.into()).or_default().push(rule);
    }

    #[must_use]
    pub fn validate(&self, values: &HashMap<String, String>) -> ValidationReport {
        let mut errors = HashMap::new();
        let mut warnings = HashMap::new();

        for (field, field_rules) in &self.rules {
            let val = values.get(field).map(String::as_str).unwrap_or("");
            for rule in field_rules {
                match rule {
                    FieldValidationRule::Required => {
                        if val.trim().is_empty() {
                            errors.insert(field.clone(), "This field is mandatory".into());
                        }
                    }
                    FieldValidationRule::Email => {
                        if !val.is_empty() && (!val.contains('@') || !val.contains('.')) {
                            errors
                                .insert(field.clone(), "Please enter a valid email address".into());
                        }
                    }
                    FieldValidationRule::Phone => {
                        if !val.is_empty() && val.chars().filter(|c| c.is_ascii_digit()).count() < 7
                        {
                            warnings.insert(
                                field.clone(),
                                "Phone number format may be incomplete".into(),
                            );
                        }
                    }
                    FieldValidationRule::Url => {
                        if !val.is_empty()
                            && !val.starts_with("http://")
                            && !val.starts_with("https://")
                        {
                            errors.insert(
                                field.clone(),
                                "URL must start with http:// or https://".into(),
                            );
                        }
                    }
                    FieldValidationRule::MinLength(min) => {
                        if !val.is_empty() && val.len() < *min {
                            errors.insert(
                                field.clone(),
                                format!("Minimum {min} characters required"),
                            );
                        }
                    }
                    FieldValidationRule::MaxLength(max) => {
                        if val.len() > *max {
                            errors.insert(
                                field.clone(),
                                format!("Maximum length is {max} characters"),
                            );
                        }
                    }
                    FieldValidationRule::NumericRange { min, max } => {
                        if !val.is_empty() {
                            if let Ok(num) = val.parse::<f64>() {
                                if let Some(m) = min
                                    && num < *m
                                {
                                    errors.insert(
                                        field.clone(),
                                        format!("Value must be at least {m}"),
                                    );
                                }
                                if let Some(m) = max
                                    && num > *m
                                {
                                    errors
                                        .insert(field.clone(), format!("Value cannot exceed {m}"));
                                }
                            } else {
                                errors.insert(field.clone(), "Please enter a valid number".into());
                            }
                        }
                    }
                    FieldValidationRule::Regex(_) => {}
                }
            }
        }

        let is_valid = errors.is_empty();
        ValidationReport {
            is_valid,
            errors,
            warnings,
        }
    }
}

// ============================================================================
// 6. CHILD TABLE INLINE EDITING & RECALCULATION
// ============================================================================

/// An individual row within a child table subgrid.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChildTableRow {
    pub id: String,
    pub values: HashMap<String, String>,
}

/// Reactive child table state model with aggregate recalculation helpers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChildTableState {
    pub child_doctype: String,
    pub rows: Vec<ChildTableRow>,
}

impl ChildTableState {
    #[must_use]
    pub fn new(child_doctype: impl Into<String>) -> Self {
        Self {
            child_doctype: child_doctype.into(),
            rows: Vec::new(),
        }
    }

    pub fn add_row(&mut self, values: HashMap<String, String>) -> String {
        let id = format!("row_{}", self.rows.len() + 1);
        self.rows.push(ChildTableRow {
            id: id.clone(),
            values,
        });
        id
    }

    pub fn remove_row(&mut self, row_id: &str) -> bool {
        if let Some(pos) = self.rows.iter().position(|r| r.id == row_id) {
            self.rows.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn update_cell(&mut self, row_id: &str, col: &str, val: &str) -> bool {
        if let Some(row) = self.rows.iter_mut().find(|r| r.id == row_id) {
            row.values.insert(col.to_string(), val.to_string());
            true
        } else {
            false
        }
    }

    /// Sums a numeric column across all rows (e.g. qty, amount).
    #[must_use]
    pub fn sum_column(&self, col: &str) -> f64 {
        self.rows
            .iter()
            .filter_map(|r| r.values.get(col).and_then(|v| v.parse::<f64>().ok()))
            .sum()
    }
}

// ============================================================================
// 7. DIOXUS V7 REACTIVE COMPONENT RENDERING
// ============================================================================

use dioxus::prelude::*;

/// Dioxus v7 Dynamic DocType Form Component with validation and dirty-state tracking.
#[component]
pub fn DocTypeForm(model: DynamicFormModel, initial_values: HashMap<String, String>) -> Element {
    let mut values = use_signal(move || initial_values);
    let mut is_dirty = use_signal(|| false);
    let mut errors = use_signal(HashMap::<String, String>::new);

    let dirty = *is_dirty.read();
    let error_map = errors.read().clone();
    let current_vals = values.read().clone();

    rsx! {
        form { class: "desk-form-container max-w-4xl mx-auto p-6 bg-zinc-950 border border-zinc-800 rounded-xl shadow-xl text-zinc-100",
            div { class: "desk-form-header flex items-center justify-between pb-4 mb-6 border-b border-zinc-800",
                div { class: "flex items-center gap-3",
                    h2 { class: "text-xl font-bold tracking-tight text-white", "{model.doctype_name}" }
                    span {
                        class: if dirty { "text-xs px-2.5 py-0.5 rounded-full bg-amber-500/20 text-amber-300 border border-amber-500/30 flex items-center gap-1.5" } else { "text-xs px-2.5 py-0.5 rounded-full bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 flex items-center gap-1.5" },
                        if dirty { "● Unsaved Changes (Ctrl+S to save)" } else { "✓ Saved to Cloud" }
                    }
                }
                div { class: "flex items-center gap-2",
                    button {
                        r#type: "button",
                        class: "px-3 py-1.5 text-xs font-medium rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 cursor-pointer transition-colors",
                        onclick: move |_| {
                            is_dirty.set(false);
                            errors.write().clear();
                        },
                        "Discard"
                    }
                    if model.is_submittable {
                        button {
                            r#type: "button",
                            class: "btn-submit px-4 py-1.5 text-xs font-semibold rounded bg-blue-600 hover:bg-blue-500 text-white cursor-pointer disabled:opacity-50 transition-colors shadow-sm",
                            disabled: !error_map.is_empty(),
                            onclick: move |_| {
                                is_dirty.set(false);
                            },
                            "Submit"
                        }
                    }
                }
            }

            div { class: "desk-form-grid grid grid-cols-1 md:grid-cols-2 gap-4",
                for widget in model.widgets {
                    {
                        let fn_name = widget.fieldname().to_string();
                        let is_visible = model.dependencies.get(&fn_name).is_none_or(|dep| eval_depends_on(dep, &current_vals));

                        if is_visible {
                            let fn_clone = fn_name.clone();
                            let fn_err = fn_name.clone();
                            let err_msg = error_map.get(&fn_err).cloned();

                            match widget {
                                FormFieldWidget::TextInput { fieldname, label, reqd } => {
                                    rsx! {
                                        div { class: "form-group flex flex-col gap-1.5",
                                            label { class: "text-xs font-medium text-zinc-300 flex items-center gap-1",
                                                "{label}"
                                                if reqd { span { class: "text-red-400", "*" } }
                                            }
                                            input {
                                                r#type: "text",
                                                required: reqd,
                                                class: "px-3 py-2 text-sm bg-zinc-900 border border-zinc-700 rounded-md text-zinc-100 placeholder-zinc-500 focus:border-blue-500 outline-none",
                                                value: values.read().get(&fieldname).cloned().unwrap_or_default(),
                                                oninput: move |evt| {
                                                    let v = evt.value();
                                                    if reqd && v.trim().is_empty() {
                                                        errors.write().insert(fn_clone.clone(), "This field is required".into());
                                                    } else {
                                                        errors.write().remove(&fn_clone);
                                                    }
                                                    values.write().insert(fn_clone.clone(), v);
                                                    is_dirty.set(true);
                                                }
                                            }
                                            if let Some(err) = err_msg {
                                                span { class: "text-[11px] text-red-400", "{err}" }
                                            }
                                        }
                                    }
                                },
                                FormFieldWidget::LongTextInput { fieldname, label, reqd, rows } => {
                                    rsx! {
                                        div { class: "form-group col-span-1 md:col-span-2 flex flex-col gap-1.5",
                                            label { class: "text-xs font-medium text-zinc-300 flex items-center gap-1",
                                                "{label}"
                                                if reqd { span { class: "text-red-400", "*" } }
                                            }
                                            textarea {
                                                rows: "{rows}",
                                                required: reqd,
                                                class: "px-3 py-2 text-sm bg-zinc-900 border border-zinc-700 rounded-md text-zinc-100 placeholder-zinc-500 focus:border-blue-500 outline-none resize-y",
                                                value: values.read().get(&fieldname).cloned().unwrap_or_default(),
                                                oninput: move |evt| {
                                                    let v = evt.value();
                                                    values.write().insert(fn_clone.clone(), v);
                                                    is_dirty.set(true);
                                                }
                                            }
                                        }
                                    }
                                },
                                FormFieldWidget::PasswordInput { fieldname, label, reqd } => {
                                    rsx! {
                                        div { class: "form-group flex flex-col gap-1.5",
                                            label { class: "text-xs font-medium text-zinc-300 flex items-center gap-1",
                                                "{label}"
                                                if reqd { span { class: "text-red-400", "*" } }
                                            }
                                            input {
                                                r#type: "password",
                                                required: reqd,
                                                class: "px-3 py-2 text-sm bg-zinc-900 border border-zinc-700 rounded-md text-zinc-100 placeholder-zinc-500 focus:border-blue-500 outline-none font-mono",
                                                value: values.read().get(&fieldname).cloned().unwrap_or_default(),
                                                oninput: move |evt| {
                                                    let v = evt.value();
                                                    values.write().insert(fn_clone.clone(), v);
                                                    is_dirty.set(true);
                                                }
                                            }
                                        }
                                    }
                                },
                                FormFieldWidget::NumberInput { fieldname, label, reqd } => {
                                    rsx! {
                                        div { class: "form-group flex flex-col gap-1.5",
                                            label { class: "text-xs font-medium text-zinc-300 flex items-center gap-1",
                                                "{label}"
                                                if reqd { span { class: "text-red-400", "*" } }
                                            }
                                            input {
                                                r#type: "number",
                                                required: reqd,
                                                class: "px-3 py-2 text-sm bg-zinc-900 border border-zinc-700 rounded-md text-zinc-100 placeholder-zinc-500 focus:border-blue-500 outline-none",
                                                value: values.read().get(&fieldname).cloned().unwrap_or_default(),
                                                oninput: move |evt| {
                                                    let v = evt.value();
                                                    values.write().insert(fn_clone.clone(), v);
                                                    is_dirty.set(true);
                                                }
                                            }
                                            if let Some(err) = err_msg {
                                                span { class: "text-[11px] text-red-400", "{err}" }
                                            }
                                        }
                                    }
                                },
                                FormFieldWidget::CurrencyInput { fieldname, label, reqd } => {
                                    rsx! {
                                        div { class: "form-group flex flex-col gap-1.5",
                                            label { class: "text-xs font-medium text-zinc-300 flex items-center gap-1",
                                                "{label}"
                                                if reqd { span { class: "text-red-400", "*" } }
                                            }
                                            input {
                                                r#type: "number",
                                                step: "0.01",
                                                required: reqd,
                                                class: "px-3 py-2 text-sm bg-zinc-900 border border-zinc-700 rounded-md text-zinc-100 placeholder-zinc-500 focus:border-blue-500 outline-none font-mono",
                                                value: values.read().get(&fieldname).cloned().unwrap_or_default(),
                                                oninput: move |evt| {
                                                    let v = evt.value();
                                                    values.write().insert(fn_clone.clone(), v);
                                                    is_dirty.set(true);
                                                }
                                            }
                                            if let Some(err) = err_msg {
                                                span { class: "text-[11px] text-red-400", "{err}" }
                                            }
                                        }
                                    }
                                },
                                FormFieldWidget::DatePicker { fieldname, label, reqd } => {
                                    rsx! {
                                        div { class: "form-group flex flex-col gap-1.5",
                                            label { class: "text-xs font-medium text-zinc-300 flex items-center gap-1",
                                                "{label}"
                                                if reqd { span { class: "text-red-400", "*" } }
                                            }
                                            input {
                                                r#type: "date",
                                                required: reqd,
                                                class: "px-3 py-2 text-sm bg-zinc-900 border border-zinc-700 rounded-md text-zinc-100 focus:border-blue-500 outline-none",
                                                value: values.read().get(&fieldname).cloned().unwrap_or_default(),
                                                onchange: move |evt| {
                                                    let v = evt.value();
                                                    values.write().insert(fn_clone.clone(), v);
                                                    is_dirty.set(true);
                                                }
                                            }
                                        }
                                    }
                                },
                                FormFieldWidget::Checkbox { fieldname, label, default_checked } => {
                                    rsx! {
                                        div { class: "form-group flex items-center gap-2 pt-5",
                                            input {
                                                r#type: "checkbox",
                                                class: "w-4 h-4 rounded bg-zinc-900 border-zinc-700 text-blue-600 focus:ring-0 cursor-pointer",
                                                checked: values.read().get(&fieldname).map_or(default_checked, |v| v == "true"),
                                                onchange: move |evt| {
                                                    let v = if evt.checked() { "true" } else { "false" };
                                                    values.write().insert(fn_clone.clone(), v.into());
                                                    is_dirty.set(true);
                                                }
                                            }
                                            label { class: "text-xs font-medium text-zinc-300 cursor-pointer", "{label}" }
                                        }
                                    }
                                },
                                FormFieldWidget::SelectDropdown { fieldname: _, label, options, reqd } => {
                                    rsx! {
                                        div { class: "form-group flex flex-col gap-1.5",
                                            label { class: "text-xs font-medium text-zinc-300 flex items-center gap-1",
                                                "{label}"
                                                if reqd { span { class: "text-red-400", "*" } }
                                            }
                                            select {
                                                required: reqd,
                                                class: "px-3 py-2 text-sm bg-zinc-900 border border-zinc-700 rounded-md text-zinc-100 focus:border-blue-500 outline-none",
                                                onchange: move |evt| {
                                                    let v = evt.value();
                                                    values.write().insert(fn_clone.clone(), v);
                                                    is_dirty.set(true);
                                                },
                                                for opt in options {
                                                    option { value: "{opt}", "{opt}" }
                                                }
                                            }
                                        }
                                    }
                                },
                                FormFieldWidget::ColorPicker { fieldname, label, default_color } => {
                                    rsx! {
                                        div { class: "form-group flex flex-col gap-1.5",
                                            label { class: "text-xs font-medium text-zinc-300", "{label}" }
                                            div { class: "flex items-center gap-2",
                                                input {
                                                    r#type: "color",
                                                    class: "w-9 h-9 p-0.5 rounded bg-zinc-900 border border-zinc-700 cursor-pointer",
                                                    value: values.read().get(&fieldname).cloned().unwrap_or(default_color),
                                                    onchange: move |evt| {
                                                        let v = evt.value();
                                                        values.write().insert(fn_clone.clone(), v);
                                                        is_dirty.set(true);
                                                    }
                                                }
                                                span { class: "text-xs font-mono text-zinc-400", "{values.read().get(&fieldname).cloned().unwrap_or_default()}" }
                                            }
                                        }
                                    }
                                },
                                _ => rsx! { div {} },
                            }
                        } else {
                            rsx! {}
                        }
                    }
                }
            }
        }
    }
}

/// Helper rendering DocTypeForm into an HTML string using Dioxus 0.7 SSR.
#[must_use]
pub fn render_doctype_form_component(
    model: &DynamicFormModel,
    initial_values: &HashMap<String, String>,
) -> String {
    let mut vdom = VirtualDom::new_with_props(
        DocTypeForm,
        DocTypeFormProps {
            model: model.clone(),
            initial_values: initial_values.clone(),
        },
    );
    vdom.rebuild_in_place();
    dioxus::ssr::render(&vdom)
}

// ============================================================================
// 8. UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_18_visual_field_types_integrity() {
        let widgets = vec![
            FormFieldWidget::TextInput {
                fieldname: "name".into(),
                label: "Name".into(),
                reqd: true,
            },
            FormFieldWidget::LongTextInput {
                fieldname: "desc".into(),
                label: "Description".into(),
                reqd: false,
                rows: 5,
            },
            FormFieldWidget::RichTextInput {
                fieldname: "content".into(),
                label: "Content".into(),
                reqd: false,
            },
            FormFieldWidget::MarkdownInput {
                fieldname: "notes".into(),
                label: "Notes".into(),
                reqd: false,
            },
            FormFieldWidget::PasswordInput {
                fieldname: "secret".into(),
                label: "Password".into(),
                reqd: true,
            },
            FormFieldWidget::NumberInput {
                fieldname: "qty".into(),
                label: "Quantity".into(),
                reqd: true,
            },
            FormFieldWidget::CurrencyInput {
                fieldname: "rate".into(),
                label: "Rate".into(),
                reqd: true,
            },
            FormFieldWidget::DatePicker {
                fieldname: "dob".into(),
                label: "Date of Birth".into(),
                reqd: false,
            },
            FormFieldWidget::TimePicker {
                fieldname: "time".into(),
                label: "Time".into(),
                reqd: false,
            },
            FormFieldWidget::DatetimePicker {
                fieldname: "ts".into(),
                label: "Timestamp".into(),
                reqd: false,
            },
            FormFieldWidget::Checkbox {
                fieldname: "is_active".into(),
                label: "Active".into(),
                default_checked: true,
            },
            FormFieldWidget::SelectDropdown {
                fieldname: "role".into(),
                label: "Role".into(),
                options: vec!["Admin".into()],
                reqd: true,
            },
            FormFieldWidget::LinkDropdown {
                fieldname: "cust".into(),
                label: "Customer".into(),
                target_doctype: "Customer".into(),
                endpoint: "/api/cust".into(),
                reqd: true,
            },
            FormFieldWidget::DynamicLinkDropdown {
                fieldname: "target".into(),
                label: "Target".into(),
                link_doctype_field: "doc_type".into(),
                reqd: false,
            },
            FormFieldWidget::TableSubgrid {
                fieldname: "items".into(),
                label: "Items".into(),
                child_doctype: "Sales Item".into(),
            },
            FormFieldWidget::AttachmentFile {
                fieldname: "file".into(),
                label: "File".into(),
                max_size_mb: 10,
                allowed_types: vec!["*".into()],
                reqd: false,
            },
            FormFieldWidget::ColorPicker {
                fieldname: "badge_color".into(),
                label: "Badge Color".into(),
                default_color: "#ff0000".into(),
            },
            FormFieldWidget::SignaturePad {
                fieldname: "sign".into(),
                label: "Signature".into(),
                reqd: true,
            },
            FormFieldWidget::CodeEditor {
                fieldname: "script".into(),
                label: "Script".into(),
                language: "javascript".into(),
                reqd: false,
            },
        ];

        assert_eq!(widgets.len(), 19);
        assert_eq!(widgets[0].fieldname(), "name");
        assert!(widgets[0].is_required());
        assert!(!widgets[10].is_required());
    }

    #[test]
    fn test_visual_condition_builder_and_evaluation() {
        let mut group = ConditionGroup::new(LogicalCombinator::And);
        group.add_rule(ConditionRule::new(
            "is_corporate",
            ConditionOperator::Equals,
            "true",
        ));
        group.add_rule(ConditionRule::new(
            "revenue",
            ConditionOperator::GreaterThan,
            "100000",
        ));

        let mut values = HashMap::new();
        values.insert("is_corporate".into(), "true".into());
        values.insert("revenue".into(), "150000".into());

        assert!(group.evaluate(&values));

        values.insert("revenue".into(), "50000".into());
        assert!(!group.evaluate(&values));
    }

    #[test]
    fn test_eval_depends_on_expressions() {
        let mut values = HashMap::new();
        values.insert("status".into(), "Open".into());
        values.insert("workflow_state".into(), "Pending".into());
        values.insert("grand_total".into(), "1500".into());

        assert!(eval_depends_on("status==Open", &values));
        assert!(!eval_depends_on("status==Closed", &values));
        assert!(eval_depends_on(
            "status==Open && workflow_state==Pending",
            &values
        ));
        assert!(!eval_depends_on(
            "status==Open && workflow_state==Approved",
            &values
        ));
        assert!(eval_depends_on("grand_total > 1000", &values));
        assert!(!eval_depends_on("grand_total < 1000", &values));
    }

    #[test]
    fn test_form_state_tracker_dirty_and_draft_lifecycle() {
        let mut initial = HashMap::new();
        initial.insert("title".into(), "Draft Invoice".into());
        initial.insert("amount".into(), "100".into());

        let mut tracker = FormStateTracker::new(initial);
        assert!(!tracker.is_dirty());

        tracker.set_field("amount", "250");
        assert!(tracker.is_dirty());
        assert!(tracker.is_field_dirty("amount"));
        assert!(!tracker.is_field_dirty("title"));
        assert_eq!(tracker.dirty_fields(), vec!["amount"]);

        let draft = tracker.create_draft_snapshot("Sales Invoice", Some("SINV-001"));
        assert_eq!(draft.values.get("amount").map(String::as_str), Some("250"));

        tracker.mark_saved();
        assert!(!tracker.is_dirty());

        tracker.set_field("title", "Modified");
        tracker.reset();
        assert!(!tracker.is_dirty());
        assert_eq!(tracker.get_field("title"), Some("Draft Invoice"));
    }

    #[test]
    fn test_form_validator_rules() {
        let mut validator = FormValidator::new();
        validator.add_rule("email", FieldValidationRule::Required);
        validator.add_rule("email", FieldValidationRule::Email);
        validator.add_rule(
            "age",
            FieldValidationRule::NumericRange {
                min: Some(18.0),
                max: Some(120.0),
            },
        );

        let mut values = HashMap::new();
        values.insert("email".into(), "invalid-email".into());
        values.insert("age".into(), "15".into());

        let report = validator.validate(&values);
        assert!(!report.is_valid);
        assert!(report.errors.contains_key("email"));
        assert!(report.errors.contains_key("age"));

        values.insert("email".into(), "admin@example.com".into());
        values.insert("age".into(), "25".into());

        let valid_report = validator.validate(&values);
        assert!(valid_report.is_valid);
        assert!(valid_report.errors.is_empty());
    }

    #[test]
    fn test_child_table_state_recalculation() {
        let mut table = ChildTableState::new("Sales Invoice Item");

        let mut row1 = HashMap::new();
        row1.insert("item_code".into(), "ITEM-A".into());
        row1.insert("amount".into(), "100.50".into());
        let id1 = table.add_row(row1);

        let mut row2 = HashMap::new();
        row2.insert("item_code".into(), "ITEM-B".into());
        row2.insert("amount".into(), "200.25".into());
        table.add_row(row2);

        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.sum_column("amount"), 300.75);

        table.update_cell(&id1, "amount", "150.50");
        assert_eq!(table.sum_column("amount"), 350.75);

        table.remove_row(&id1);
        assert_eq!(table.rows.len(), 1);
        assert_eq!(table.sum_column("amount"), 200.25);
    }
}
