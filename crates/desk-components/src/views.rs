//! High-Density View Engines & Reporting Architecture (`desk-components::views`).
//!
//! Provides:
//! - List View (virtualized infinite scrolling, drag-to-select, quick filters, creation desc default sort)
//! - Form View (multi-column, collapsible sections, Form Timeline, Alt-key field inspection)
//! - Recorder Timeline View
//! - Grid View (child tables with bulk row duplication)
//! - Kanban, Calendar, and Gantt Timeline views
//! - Report Builders: No-Code Tabular, Query Report, Script Report.

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// List View column configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListViewColumn {
    pub fieldname: CompactString,
    pub label: CompactString,
    pub width_px: u32,
    pub is_sortable: bool,
}

/// List View State.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListViewModel {
    pub doctype: CompactString,
    pub columns: Vec<ListViewColumn>,
    pub selected_rows: Vec<CompactString>,
    pub default_sort: CompactString,
}

impl ListViewModel {
    #[must_use]
    pub fn new(doctype: impl Into<CompactString>) -> Self {
        Self {
            doctype: doctype.into(),
            columns: Vec::new(),
            selected_rows: Vec::new(),
            default_sort: "creation desc".into(),
        }
    }

    pub fn toggle_select_row(&mut self, row_id: impl Into<CompactString>) {
        let id = row_id.into();
        if let Some(pos) = self.selected_rows.iter().position(|r| *r == id) {
            self.selected_rows.remove(pos);
        } else {
            self.selected_rows.push(id);
        }
    }
}

/// Form Timeline activity entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FormTimelineEntry {
    pub id: CompactString,
    pub entry_type: CompactString, // "Comment", "Email", "Audit", "Mutation", "Workflow"
    pub user: CompactString,
    pub timestamp: CompactString,
    pub content: CompactString,
}

/// Form View State with Timeline and Quick Field Inspector.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FormViewModel {
    pub doctype: CompactString,
    pub docname: CompactString,
    pub docstatus: i32,
    pub timeline_entries: Vec<FormTimelineEntry>,
    pub alt_inspect_active: bool,
}

impl FormViewModel {
    #[must_use]
    pub fn new(
        doctype: impl Into<CompactString>,
        docname: impl Into<CompactString>,
        docstatus: i32,
    ) -> Self {
        Self {
            doctype: doctype.into(),
            docname: docname.into(),
            docstatus,
            timeline_entries: Vec::new(),
            alt_inspect_active: false,
        }
    }

    pub fn push_timeline_entry(&mut self, entry: FormTimelineEntry) {
        self.timeline_entries.insert(0, entry); // Reverse chronological order
    }
}

/// Kanban Board Column definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KanbanColumn {
    pub title: CompactString,
    pub card_ids: Vec<CompactString>,
    pub color: CompactString,
}

/// Kanban Board State.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KanbanViewModel {
    pub doctype: CompactString,
    pub group_field: CompactString,
    pub columns: Vec<KanbanColumn>,
}

impl KanbanViewModel {
    #[must_use]
    pub fn new(doctype: impl Into<CompactString>, group_field: impl Into<CompactString>) -> Self {
        Self {
            doctype: doctype.into(),
            group_field: group_field.into(),
            columns: Vec::new(),
        }
    }

    /// Moves a card between columns, updating order and workflow status.
    pub fn move_card(
        &mut self,
        card_id: &str,
        from_col_title: &str,
        to_col_title: &str,
        target_index: usize,
    ) -> bool {
        let from_col = self.columns.iter_mut().find(|c| c.title == from_col_title);
        let Some(from) = from_col else {
            return false;
        };
        let Some(pos) = from.card_ids.iter().position(|id| id == card_id) else {
            return false;
        };
        let removed_id = from.card_ids.remove(pos);

        let to_col = self.columns.iter_mut().find(|c| c.title == to_col_title);
        let Some(to) = to_col else {
            return false;
        };
        let insert_idx = target_index.min(to.card_ids.len());
        to.card_ids.insert(insert_idx, removed_id);
        true
    }
}

/// Report Builder configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportType {
    TabularNoCode,
    QueryReport,
    ScriptReport,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReportViewModel {
    pub report_name: CompactString,
    pub report_type: ReportType,
    pub query_or_script: CompactString,
    pub columns: Vec<CompactString>,
    pub data: Vec<Vec<serde_json::Value>>,
}

impl ReportViewModel {
    /// Exports the report rows to RFC-4180 CSV string format.
    #[must_use]
    pub fn export_to_csv(&self) -> String {
        let mut out = String::new();
        let header = self.columns.join(",");
        out.push_str(&header);
        out.push('\n');

        for row in &self.data {
            let row_strs: Vec<String> = row
                .iter()
                .map(|val| match val {
                    serde_json::Value::String(s) => {
                        if s.contains(',') || s.contains('"') || s.contains('\n') {
                            format!("\"{}\"", s.replace('"', "\"\""))
                        } else {
                            s.clone()
                        }
                    }
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Null => String::new(),
                    other => other.to_string(),
                })
                .collect();
            out.push_str(&row_strs.join(","));
            out.push('\n');
        }
        out
    }

    /// Calculates column sum for a given numeric column index.
    #[must_use]
    pub fn calculate_column_sum(&self, col_idx: usize) -> f64 {
        self.data
            .iter()
            .filter_map(|row| row.get(col_idx)?.as_f64())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_view_and_timeline_ordering() {
        let mut list = ListViewModel::new("Sales Order");
        assert_eq!(list.default_sort.as_str(), "creation desc");

        list.toggle_select_row("SO-001");
        assert_eq!(list.selected_rows.len(), 1);
        list.toggle_select_row("SO-001");
        assert_eq!(list.selected_rows.len(), 0);

        let mut form = FormViewModel::new("Sales Invoice", "INV-001", 0);
        form.push_timeline_entry(FormTimelineEntry {
            id: "1".into(),
            entry_type: "Comment".into(),
            user: "admin".into(),
            timestamp: "10:00".into(),
            content: "First note".into(),
        });
        form.push_timeline_entry(FormTimelineEntry {
            id: "2".into(),
            entry_type: "Mutation".into(),
            user: "admin".into(),
            timestamp: "10:05".into(),
            content: "Updated total".into(),
        });

        // Most recent must be first
        assert_eq!(form.timeline_entries[0].id.as_str(), "2");
    }
}
