//! DocType JSON-driven Form View Engine (`desk-components::form_engine`).
//!
//! Compiles Frappe server DocTypeSchema AST into a full hierarchical client layout:
//! - Tabs (`TabBreak`)
//! - Sections (`SectionBreak`) with collapsible state, title, and columns
//! - Columns (`ColumnBreak`) for multi-column grid distribution
//! - Child Table Grids (`Table` / `Table MultiSelect`)
//! - Linked DocType Autocomplete & Dynamic Links
//! - Form Timeline & Activity Trail (Comments, Audit, Mutations)
//! - Alt-Key Quick Field Inspector telemetry.

pub mod client_script;

use compact_str::CompactString;
use rwl_meta::{DocTypeSchema, FieldType};
use serde::{Deserialize, Serialize};

pub use client_script::{
    ClientScriptEngine, ClientScriptError, DynamicFieldState, DynamicFormEvaluator,
};

/// Information exposed when Alt-key hovering over a field for live inspection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FieldInspectorInfo {
    pub fieldname: CompactString,
    pub label: CompactString,
    pub fieldtype: CompactString,
    pub reqd: bool,
    pub read_only: bool,
    pub depends_on: Option<CompactString>,
    pub default_value: Option<CompactString>,
}

/// Child Table Grid Column Schema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChildGridColumn {
    pub fieldname: CompactString,
    pub label: CompactString,
    pub fieldtype: CompactString,
    pub reqd: bool,
    pub width_px: u32,
}

/// Child Table Subgrid Layout Model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChildTableGridModel {
    pub fieldname: CompactString,
    pub label: CompactString,
    pub child_doctype: CompactString,
    pub columns: Vec<ChildGridColumn>,
    pub allow_bulk_add: bool,
    pub min_rows: usize,
    pub max_rows: Option<usize>,
}

/// Form Field Widget within a Column.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FormLayoutWidget {
    Field {
        fieldname: CompactString,
        label: CompactString,
        fieldtype: CompactString,
        reqd: bool,
        read_only: bool,
        depends_on: Option<CompactString>,
        inspector: FieldInspectorInfo,
    },
    ChildTable(ChildTableGridModel),
    CustomHtml {
        html_content: CompactString,
    },
}

/// Form Column within a Section.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FormColumn {
    pub column_index: usize,
    pub widgets: Vec<FormLayoutWidget>,
}

/// Form Section within a Tab.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FormSection {
    pub title: Option<CompactString>,
    pub is_collapsible: bool,
    pub is_collapsed: bool,
    pub depends_on: Option<CompactString>,
    pub columns: Vec<FormColumn>,
}

/// Form Tab within the Form View.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FormTab {
    pub label: CompactString,
    pub icon: Option<CompactString>,
    pub depends_on: Option<CompactString>,
    pub sections: Vec<FormSection>,
}

/// Timeline entry representing user actions, audit mutations, or workflow transitions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FormTimelineEvent {
    pub id: CompactString,
    pub event_type: CompactString, // "Comment", "Audit", "Workflow", "Email"
    pub user: CompactString,
    pub timestamp_iso: CompactString,
    pub message: CompactString,
}

/// Form Timeline Model attached to the bottom/sidebar of the Form View.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FormTimelineModel {
    pub events: Vec<FormTimelineEvent>,
}

impl FormTimelineModel {
    pub fn push_event(&mut self, event: FormTimelineEvent) {
        self.events.insert(0, event); // Reverse chronological
    }
}

/// Complete Compiled Form Layout driven by DocTypeSchema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompiledFormLayout {
    pub doctype_name: CompactString,
    pub module: CompactString,
    pub is_submittable: bool,
    pub is_tree: bool,
    pub tabs: Vec<FormTab>,
    pub timeline: FormTimelineModel,
    pub alt_inspect_enabled: bool,
}

/// Form Layout Compilation Engine.
pub struct FormLayoutEngine;

impl FormLayoutEngine {
    /// Compiles a DocTypeSchema into a structured tabs -> sections -> columns -> widgets hierarchy.
    #[must_use]
    pub fn compile_layout(schema: &DocTypeSchema) -> CompiledFormLayout {
        let mut tabs = Vec::new();
        let mut current_tab = FormTab {
            label: "Details".into(),
            icon: None,
            depends_on: None,
            sections: Vec::new(),
        };

        let mut current_section = FormSection {
            title: None,
            is_collapsible: false,
            is_collapsed: false,
            depends_on: None,
            columns: vec![FormColumn {
                column_index: 0,
                widgets: Vec::new(),
            }],
        };

        for field in &schema.fields {
            if field.hidden {
                continue;
            }

            match &field.fieldtype {
                FieldType::TabBreak => {
                    // Flush current section to current tab
                    if !current_section.columns.iter().all(|c| c.widgets.is_empty()) {
                        current_tab.sections.push(current_section);
                    }
                    // Flush current tab to tabs list
                    if !current_tab.sections.is_empty() {
                        tabs.push(current_tab);
                    }
                    // Start new tab
                    current_tab = FormTab {
                        label: if field.label.is_empty() {
                            "Tab".into()
                        } else {
                            field.label.as_str().into()
                        },
                        icon: None,
                        depends_on: field.depends_on.as_ref().map(|d| d.as_str().into()),
                        sections: Vec::new(),
                    };
                    current_section = FormSection {
                        title: None,
                        is_collapsible: false,
                        is_collapsed: false,
                        depends_on: None,
                        columns: vec![FormColumn {
                            column_index: 0,
                            widgets: Vec::new(),
                        }],
                    };
                }
                FieldType::SectionBreak { collapsible } => {
                    // Flush current section if non-empty
                    if !current_section.columns.iter().all(|c| c.widgets.is_empty()) {
                        current_tab.sections.push(current_section);
                    }
                    // Start new section
                    current_section = FormSection {
                        title: if field.label.is_empty() {
                            None
                        } else {
                            Some(field.label.as_str().into())
                        },
                        is_collapsible: *collapsible,
                        is_collapsed: *collapsible,
                        depends_on: field.depends_on.as_ref().map(|d| d.as_str().into()),
                        columns: vec![FormColumn {
                            column_index: 0,
                            widgets: Vec::new(),
                        }],
                    };
                }
                FieldType::ColumnBreak => {
                    // Add new column to current section
                    let new_idx = current_section.columns.len();
                    current_section.columns.push(FormColumn {
                        column_index: new_idx,
                        widgets: Vec::new(),
                    });
                }
                FieldType::Table { child_doctype } => {
                    let col_idx = current_section.columns.len().saturating_sub(1);
                    current_section.columns[col_idx]
                        .widgets
                        .push(FormLayoutWidget::ChildTable(ChildTableGridModel {
                            fieldname: field.fieldname.as_str().into(),
                            label: field.label.as_str().into(),
                            child_doctype: child_doctype.as_str().into(),
                            columns: Vec::new(), // Populated by child doctype schema if loaded
                            allow_bulk_add: true,
                            min_rows: 0,
                            max_rows: None,
                        }));
                }
                ft => {
                    let fieldtype_str = format!("{ft:?}");
                    let inspector = FieldInspectorInfo {
                        fieldname: field.fieldname.as_str().into(),
                        label: field.label.as_str().into(),
                        fieldtype: fieldtype_str.as_str().into(),
                        reqd: field.reqd,
                        read_only: field.read_only,
                        depends_on: field.depends_on.as_ref().map(|d| d.as_str().into()),
                        default_value: field.default_value.as_ref().map(|v| v.to_string().into()),
                    };

                    let col_idx = current_section.columns.len().saturating_sub(1);
                    current_section.columns[col_idx]
                        .widgets
                        .push(FormLayoutWidget::Field {
                            fieldname: field.fieldname.as_str().into(),
                            label: field.label.as_str().into(),
                            fieldtype: fieldtype_str.into(),
                            reqd: field.reqd,
                            read_only: field.read_only,
                            depends_on: field.depends_on.as_ref().map(|d| d.as_str().into()),
                            inspector,
                        });
                }
            }
        }

        // Flush remaining section and tab
        if !current_section.columns.iter().all(|c| c.widgets.is_empty()) {
            current_tab.sections.push(current_section);
        }
        if !current_tab.sections.is_empty() {
            tabs.push(current_tab);
        }

        CompiledFormLayout {
            doctype_name: schema.name.as_str().into(),
            module: schema.module.as_str().into(),
            is_submittable: schema.is_submittable,
            is_tree: schema.is_tree,
            tabs,
            timeline: FormTimelineModel::default(),
            alt_inspect_enabled: true,
        }
    }
}
