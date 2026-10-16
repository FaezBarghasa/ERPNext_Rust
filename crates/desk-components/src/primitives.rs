//! Frappe UI Primitive Suite (`desk-components::primitives`).
//!
//! Provides native, zero-runtime WASM and Rust desktop UI primitives matching
//! all props, events, and styling slots of `frappe-ui`.

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

// -------------------------------------------------------------------------
// Display Primitives
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertVariant {
    Info,
    Success,
    Warning,
    Danger,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AlertModel {
    pub variant: AlertVariant,
    pub title: CompactString,
    pub message: CompactString,
    pub dismissible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AvatarSize {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    Xxl,
    Xxxl,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AvatarModel {
    pub label: CompactString,
    pub image_url: Option<CompactString>,
    pub size: AvatarSize,
    pub is_round: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BadgeModel {
    pub label: CompactString,
    pub theme: CompactString, // "gray", "blue", "green", "red", "orange", "purple"
    pub size: CompactString,  // "sm", "md", "lg"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ButtonVariant {
    Solid,
    Subtle,
    Outline,
    Ghost,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ButtonModel {
    pub label: CompactString,
    pub variant: ButtonVariant,
    pub theme: CompactString,
    pub loading: bool,
    pub disabled: bool,
    pub icon_prefix: Option<CompactString>,
    pub icon_suffix: Option<CompactString>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CardModel {
    pub title: CompactString,
    pub subtitle: Option<CompactString>,
    pub is_loading: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RatingModel {
    pub score: f32, // 1.0 to 5.0
    pub max: u8,
    pub is_readonly: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TooltipModel {
    pub text: CompactString,
    pub position: CompactString, // "top", "bottom", "left", "right"
}

// -------------------------------------------------------------------------
// Form Primitives
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutocompleteOption {
    pub label: CompactString,
    pub value: CompactString,
    pub group: Option<CompactString>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutocompleteModel {
    pub placeholder: CompactString,
    pub options: Vec<AutocompleteOption>,
    pub selected_value: Option<CompactString>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MultiSelectModel {
    pub tags: Vec<CompactString>,
    pub max_tags: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SliderModel {
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub current_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileUploaderModel {
    pub allowed_extensions: Vec<CompactString>,
    pub max_size_mb: u32,
    pub upload_progress_pct: u8,
    pub is_uploading: bool,
}

// -------------------------------------------------------------------------
// Navigation & Command Primitives
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DialogModel {
    pub title: CompactString,
    pub message: CompactString,
    pub size: CompactString, // "xs", "sm", "md", "lg", "xl", "2xl", "full"
    pub is_open: bool,
    pub is_danger: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToastModel {
    pub id: CompactString,
    pub text: CompactString,
    pub variant: AlertVariant,
    pub timeout_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandPaletteItem {
    pub id: CompactString,
    pub title: CompactString,
    pub group: CompactString, // "Routes", "DocTypes", "Actions"
    pub shortcut: Option<CompactString>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandPaletteModel {
    pub query: CompactString,
    pub items: Vec<CommandPaletteItem>,
    pub is_open: bool,
}

impl CommandPaletteModel {
    #[must_use]
    pub fn new() -> Self {
        Self {
            query: CompactString::default(),
            items: Vec::new(),
            is_open: false,
        }
    }

    /// Fuzzy searches items based on query.
    #[must_use]
    pub fn search_results(&self) -> Vec<&CommandPaletteItem> {
        let q = self.query.to_lowercase();
        if q.is_empty() {
            return self.items.iter().collect();
        }

        let q_str = q.as_str();
        self.items
            .iter()
            .filter(|item| {
                item.title.to_lowercase().contains(q_str)
                    || item.group.to_lowercase().contains(q_str)
            })
            .collect()
    }
}

impl Default for CommandPaletteModel {
    fn default() -> Self {
        Self::new()
    }
}

// -------------------------------------------------------------------------
// Combobox & Advanced Select
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComboboxOption {
    pub label: CompactString,
    pub value: CompactString,
    pub description: Option<CompactString>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComboboxModel {
    pub placeholder: CompactString,
    pub query: CompactString,
    pub options: Vec<ComboboxOption>,
    pub selected_value: Option<CompactString>,
    pub is_open: bool,
    pub allow_custom: bool,
}

impl ComboboxModel {
    #[must_use]
    pub fn new(placeholder: impl Into<CompactString>, options: Vec<ComboboxOption>) -> Self {
        Self {
            placeholder: placeholder.into(),
            query: CompactString::default(),
            options,
            selected_value: None,
            is_open: false,
            allow_custom: false,
        }
    }

    pub fn filtered_options(&self) -> Vec<&ComboboxOption> {
        let q = self.query.to_lowercase();
        if q.is_empty() {
            return self.options.iter().collect();
        }
        let q_str = q.as_str();
        self.options
            .iter()
            .filter(|opt| {
                opt.label.to_lowercase().contains(q_str)
                    || opt.value.to_lowercase().contains(q_str)
                    || opt
                        .description
                        .as_ref()
                        .is_some_and(|d| d.to_lowercase().contains(q_str))
            })
            .collect()
    }
}

// -------------------------------------------------------------------------
// Date & Time Pickers
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatePickerModel {
    pub label: CompactString,
    pub value: Option<CompactString>, // ISO Date "YYYY-MM-DD"
    pub min_date: Option<CompactString>,
    pub max_date: Option<CompactString>,
    pub is_required: bool,
    pub is_readonly: bool,
}

impl DatePickerModel {
    #[must_use]
    pub fn new(label: impl Into<CompactString>, value: Option<impl Into<CompactString>>) -> Self {
        Self {
            label: label.into(),
            value: value.map(Into::into),
            min_date: None,
            max_date: None,
            is_required: false,
            is_readonly: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimePickerModel {
    pub label: CompactString,
    pub value: Option<CompactString>, // Time "HH:MM:SS"
    pub step_minutes: u32,
    pub is_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DateTimePickerModel {
    pub label: CompactString,
    pub value: Option<CompactString>, // ISO-8601 Timestamp
    pub is_required: bool,
}

// -------------------------------------------------------------------------
// Tree View (Hierarchical Organizational / Chart of Accounts)
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TreeNode {
    pub id: CompactString,
    pub label: CompactString,
    pub parent_id: Option<CompactString>,
    pub is_group: bool,
    pub is_expanded: bool,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    #[must_use]
    pub fn new(
        id: impl Into<CompactString>,
        label: impl Into<CompactString>,
        parent_id: Option<impl Into<CompactString>>,
        is_group: bool,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            parent_id: parent_id.map(Into::into),
            is_group,
            is_expanded: false,
            children: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TreeViewModel {
    pub root_nodes: Vec<TreeNode>,
    pub selected_node_id: Option<CompactString>,
}

impl TreeViewModel {
    #[must_use]
    pub fn new(root_nodes: Vec<TreeNode>) -> Self {
        Self {
            root_nodes,
            selected_node_id: None,
        }
    }
}

// -------------------------------------------------------------------------
// Context Menu
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextMenuItem {
    pub id: CompactString,
    pub label: CompactString,
    pub icon: Option<CompactString>,
    pub is_disabled: bool,
    pub is_danger: bool,
    pub shortcut: Option<CompactString>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ContextMenuModel {
    pub x: f64,
    pub y: f64,
    pub items: Vec<ContextMenuItem>,
    pub is_open: bool,
}

// -------------------------------------------------------------------------
// Calendar View
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CalendarViewMode {
    #[default]
    Month,
    Week,
    Day,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CalendarEvent {
    pub id: CompactString,
    pub title: CompactString,
    pub start_iso: CompactString,
    pub end_iso: CompactString,
    pub color: CompactString,
    pub all_day: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CalendarViewModel {
    pub doctype: CompactString,
    pub mode: CalendarViewMode,
    pub current_date_iso: CompactString,
    pub events: Vec<CalendarEvent>,
}

// -------------------------------------------------------------------------
// Dashboard & Analytics
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NumberCardModel {
    pub title: CompactString,
    pub value_formatted: CompactString,
    pub delta_percentage: Option<f64>,
    pub is_positive_delta: bool,
    pub color: CompactString,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChartCardModel {
    pub title: CompactString,
    pub chart_type: CompactString, // "line", "bar", "donut", "heatmap"
    pub labels: Vec<CompactString>,
    pub datasets: Vec<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DashboardViewModel {
    pub dashboard_name: CompactString,
    pub number_cards: Vec<NumberCardModel>,
    pub chart_cards: Vec<ChartCardModel>,
}

// -------------------------------------------------------------------------
// Mobile Shell & PWA
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MobileNavItem {
    pub route: CompactString,
    pub label: CompactString,
    pub icon: CompactString,
    pub badge: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MobileShellModel {
    pub active_route: CompactString,
    pub nav_items: Vec<MobileNavItem>,
    pub bottom_bar_visible: bool,
    pub offline_indicator: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_palette_filtering() {
        let mut palette = CommandPaletteModel::new();
        palette.items.push(CommandPaletteItem {
            id: "1".into(),
            title: "Sales Invoice List".into(),
            group: "DocTypes".into(),
            shortcut: Some("Cmd+Shift+I".into()),
        });
        palette.items.push(CommandPaletteItem {
            id: "2".into(),
            title: "General Ledger Report".into(),
            group: "Reports".into(),
            shortcut: None,
        });

        palette.query = "sales".into();
        let results = palette.search_results();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title.as_str(), "Sales Invoice List");
    }
}
