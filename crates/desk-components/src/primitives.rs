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
