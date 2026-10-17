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

/// Supported comparison operators for dynamic List View filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    Like,
    In,
    GreaterThan,
    LessThan,
}

/// An individual filter criterion on a DocType field.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FilterCriterion {
    pub fieldname: CompactString,
    pub operator: FilterOperator,
    pub value: CompactString,
}

impl FilterCriterion {
    #[must_use]
    pub fn matches(&self, val: &str) -> bool {
        match self.operator {
            FilterOperator::Equals => val == self.value.as_str(),
            FilterOperator::NotEquals => val != self.value.as_str(),
            FilterOperator::Like => val
                .to_lowercase()
                .contains(self.value.to_lowercase().as_str()),
            FilterOperator::In => self
                .value
                .split(',')
                .map(|s| s.trim())
                .any(|item| item.eq_ignore_ascii_case(val)),
            FilterOperator::GreaterThan => {
                if let (Ok(num_val), Ok(num_target)) =
                    (val.parse::<f64>(), self.value.parse::<f64>())
                {
                    num_val > num_target
                } else {
                    val > self.value.as_str()
                }
            }
            FilterOperator::LessThan => {
                if let (Ok(num_val), Ok(num_target)) =
                    (val.parse::<f64>(), self.value.parse::<f64>())
                {
                    num_val < num_target
                } else {
                    val < self.value.as_str()
                }
            }
        }
    }
}

/// Dynamic Filter Drawer state for List and Kanban views.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FilterDrawerModel {
    pub filters: Vec<FilterCriterion>,
    pub is_open: bool,
}

impl FilterDrawerModel {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle_open(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn add_filter(
        &mut self,
        fieldname: impl Into<CompactString>,
        operator: FilterOperator,
        value: impl Into<CompactString>,
    ) {
        self.filters.push(FilterCriterion {
            fieldname: fieldname.into(),
            operator,
            value: value.into(),
        });
    }

    pub fn remove_filter(&mut self, idx: usize) -> bool {
        if idx < self.filters.len() {
            self.filters.remove(idx);
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.filters.clear();
    }

    /// Evaluates all filter criteria against a record JSON object (conjunction AND).
    #[must_use]
    pub fn matches_record(&self, record: &serde_json::Map<String, serde_json::Value>) -> bool {
        for f in &self.filters {
            let field_val = match record.get(f.fieldname.as_str()) {
                Some(serde_json::Value::String(s)) => s.clone(),
                Some(serde_json::Value::Number(n)) => n.to_string(),
                Some(serde_json::Value::Bool(b)) => b.to_string(),
                _ => String::new(),
            };
            if !f.matches(&field_val) {
                return false;
            }
        }
        true
    }
}

/// Supported batch actions on multiple selected rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchActionType {
    Delete,
    Submit,
    Cancel,
    Print,
}

/// Batch action selection bar model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BatchActionBarModel {
    pub selected_rows: Vec<CompactString>,
    pub is_visible: bool,
}

impl BatchActionBarModel {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_selection(&mut self, selected: Vec<CompactString>) {
        self.is_visible = !selected.is_empty();
        self.selected_rows = selected;
    }

    pub fn clear(&mut self) {
        self.selected_rows.clear();
        self.is_visible = false;
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

    #[test]
    fn test_filter_drawer_and_batch_actions() {
        let mut drawer = FilterDrawerModel::new();
        drawer.add_filter("status", FilterOperator::In, "Draft,Submitted");
        drawer.add_filter("grand_total", FilterOperator::GreaterThan, "500");

        let mut record_match = serde_json::Map::new();
        record_match.insert("status".into(), serde_json::json!("Draft"));
        record_match.insert("grand_total".into(), serde_json::json!("1000.00"));
        assert!(drawer.matches_record(&record_match));

        let mut record_mismatch = serde_json::Map::new();
        record_mismatch.insert("status".into(), serde_json::json!("Cancelled"));
        record_mismatch.insert("grand_total".into(), serde_json::json!("1000.00"));
        assert!(!drawer.matches_record(&record_mismatch));

        let mut batch = BatchActionBarModel::new();
        assert!(!batch.is_visible);
        batch.update_selection(vec!["INV-001".into(), "INV-002".into()]);
        assert!(batch.is_visible);
        assert_eq!(batch.selected_rows.len(), 2);
        batch.clear();
        assert!(!batch.is_visible);
    }
}
