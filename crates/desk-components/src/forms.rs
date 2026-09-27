use frappe_meta::{DocTypeSchema, FieldType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Client-side UI widget type mapped from server DocField AST.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FormFieldWidget {
    TextInput {
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
    Checkbox {
        fieldname: String,
        label: String,
        default_checked: bool,
    },
    LinkDropdown {
        fieldname: String,
        label: String,
        target_doctype: String,
        endpoint: String,
        reqd: bool,
    },
    SelectDropdown {
        fieldname: String,
        label: String,
        options: Vec<String>,
        reqd: bool,
    },
    TableSubgrid {
        fieldname: String,
        label: String,
        child_doctype: String,
    },
}

/// Dynamic Form Component Model compiled from DocTypeSchema AST (Milestone 5.6).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DynamicFormModel {
    pub doctype_name: String,
    pub is_submittable: bool,
    pub widgets: Vec<FormFieldWidget>,
}

impl DynamicFormModel {
    /// Compiles a server DocTypeSchema AST into a client-side DynamicFormModel.
    #[must_use]
    pub fn from_schema(schema: &DocTypeSchema) -> Self {
        let mut widgets = Vec::new();

        for field in &schema.fields {
            if field.hidden {
                continue;
            }

            let widget = match &field.fieldtype {
                FieldType::Data
                | FieldType::Text
                | FieldType::LongText
                | FieldType::Code
                | FieldType::Password => FormFieldWidget::TextInput {
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
        }
    }
}

/// Evaluates Frappe dynamic form `depends_on` visibility conditions.
#[must_use]
pub fn eval_depends_on(expr: &str, form_values: &HashMap<String, String>) -> bool {
    if expr.trim().is_empty() {
        return true;
    }

    expr.split("&&").all(|condition| {
        let trimmed = condition.trim();
        if let Some((k, v)) = trimmed.split_once("==") {
            let key = k.trim();
            let val = v.trim().trim_matches('"');
            form_values.get(key).map(|x| x == val).unwrap_or(false)
        } else if let Some((k, v)) = trimmed.split_once("!=") {
            let key = k.trim();
            let val = v.trim().trim_matches('"');
            form_values.get(key).map(|x| x != val).unwrap_or(true)
        } else {
            false
        }
    })
}
