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
pub struct AttachedFile {
    pub name: CompactString,
    pub size_bytes: usize,
    pub file_url: CompactString,
    pub mime_type: CompactString,
    pub is_image: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FileUploaderModel {
    pub allowed_extensions: Vec<CompactString>,
    pub max_size_mb: u32,
    pub upload_progress_pct: u8,
    pub is_uploading: bool,
    pub attached_files: Vec<AttachedFile>,
}

impl FileUploaderModel {
    #[must_use]
    pub fn new(allowed_extensions: Vec<CompactString>, max_size_mb: u32) -> Self {
        Self {
            allowed_extensions,
            max_size_mb,
            upload_progress_pct: 0,
            is_uploading: false,
            attached_files: Vec::new(),
        }
    }

    pub fn validate_file(&self, filename: &str, size_bytes: usize) -> Result<(), CompactString> {
        let max_bytes = (self.max_size_mb as usize) * 1024 * 1024;
        if self.max_size_mb > 0 && size_bytes > max_bytes {
            return Err(format!(
                "File size {size_bytes} bytes exceeds maximum permitted size of {} MB",
                self.max_size_mb
            )
            .into());
        }

        if !self.allowed_extensions.is_empty() {
            let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
            let matched = self
                .allowed_extensions
                .iter()
                .any(|e| e.trim_start_matches('.').to_lowercase() == ext);
            if !matched {
                return Err(format!("File extension '.{ext}' is not permitted").into());
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn detect_mime(filename: &str) -> (CompactString, bool) {
        let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
        match ext.as_str() {
            "png" => ("image/png".into(), true),
            "jpg" | "jpeg" => ("image/jpeg".into(), true),
            "gif" => ("image/gif".into(), true),
            "webp" => ("image/webp".into(), true),
            "svg" => ("image/svg+xml".into(), true),
            "pdf" => ("application/pdf".into(), false),
            "csv" => ("text/csv".into(), false),
            "json" => ("application/json".into(), false),
            "xlsx" => (
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".into(),
                false,
            ),
            _ => ("application/octet-stream".into(), false),
        }
    }

    pub fn attach_file(
        &mut self,
        filename: impl Into<CompactString>,
        size_bytes: usize,
        file_url: impl Into<CompactString>,
    ) -> Result<&AttachedFile, CompactString> {
        let fname = filename.into();
        self.validate_file(fname.as_str(), size_bytes)?;
        let (mime_type, is_image) = Self::detect_mime(fname.as_str());
        let file = AttachedFile {
            name: fname,
            size_bytes,
            file_url: file_url.into(),
            mime_type,
            is_image,
        };
        self.attached_files.push(file);
        Ok(self.attached_files.last().expect("just pushed"))
    }

    pub fn remove_file(&mut self, filename: &str) -> bool {
        if let Some(pos) = self.attached_files.iter().position(|f| f.name == filename) {
            self.attached_files.remove(pos);
            true
        } else {
            false
        }
    }
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
    pub group: CompactString, // "Routes", "DocTypes", "Actions", "Recent"
    pub route: Option<CompactString>,
    pub shortcut: Option<CompactString>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CommandPaletteModel {
    pub query: CompactString,
    pub items: Vec<CommandPaletteItem>,
    pub recent_ids: Vec<CompactString>,
    pub selected_index: usize,
    pub is_open: bool,
}

impl CommandPaletteModel {
    #[must_use]
    pub fn new() -> Self {
        Self {
            query: CompactString::default(),
            items: Vec::new(),
            recent_ids: Vec::new(),
            selected_index: 0,
            is_open: false,
        }
    }

    pub fn add_item(&mut self, item: CommandPaletteItem) {
        self.items.push(item);
    }

    pub fn add_route(
        &mut self,
        id: impl Into<CompactString>,
        title: impl Into<CompactString>,
        route: impl Into<CompactString>,
        shortcut: Option<impl Into<CompactString>>,
    ) {
        self.items.push(CommandPaletteItem {
            id: id.into(),
            title: title.into(),
            group: "Routes".into(),
            route: Some(route.into()),
            shortcut: shortcut.map(Into::into),
        });
    }

    pub fn add_doctype(&mut self, doctype: impl Into<CompactString>) {
        let dt = doctype.into();
        let title = format!("{dt} List");
        let route = format!("/app/{}", dt.to_lowercase().replace(' ', "-"));
        self.items.push(CommandPaletteItem {
            id: format!("doctype:{dt}").into(),
            title: title.into(),
            group: "DocTypes".into(),
            route: Some(route.into()),
            shortcut: None,
        });
    }

    pub fn add_action(
        &mut self,
        id: impl Into<CompactString>,
        title: impl Into<CompactString>,
        shortcut: Option<impl Into<CompactString>>,
    ) {
        self.items.push(CommandPaletteItem {
            id: id.into(),
            title: title.into(),
            group: "Actions".into(),
            route: None,
            shortcut: shortcut.map(Into::into),
        });
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.selected_index = 0;
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.query.clear();
        self.selected_index = 0;
    }

    /// Fuzzy/ranked searches items based on query.
    #[must_use]
    pub fn search_results(&self) -> Vec<&CommandPaletteItem> {
        let q = self.query.to_lowercase();
        if q.is_empty() {
            let mut results = Vec::new();
            for r_id in &self.recent_ids {
                if let Some(item) = self.items.iter().find(|i| i.id == *r_id) {
                    results.push(item);
                }
            }
            for item in &self.items {
                if !self.recent_ids.contains(&item.id) {
                    results.push(item);
                }
            }
            return results;
        }

        let q_str = q.as_str();
        let mut exact = Vec::new();
        let mut prefix = Vec::new();
        let mut contains = Vec::new();

        for item in &self.items {
            let title_lower = item.title.to_lowercase();
            let group_lower = item.group.to_lowercase();
            let route_match = item
                .route
                .as_ref()
                .is_some_and(|r| r.to_lowercase().contains(q_str));

            if title_lower == q_str {
                exact.push(item);
            } else if title_lower.starts_with(q_str) {
                prefix.push(item);
            } else if title_lower.contains(q_str) || group_lower.contains(q_str) || route_match {
                contains.push(item);
            }
        }

        exact.extend(prefix);
        exact.extend(contains);
        exact
    }

    pub fn select_next(&mut self) {
        let total = self.search_results().len();
        if total > 0 {
            self.selected_index = (self.selected_index + 1) % total;
        }
    }

    pub fn select_prev(&mut self) {
        let total = self.search_results().len();
        if total > 0 {
            if self.selected_index == 0 {
                self.selected_index = total - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    #[must_use]
    pub fn current_selection(&self) -> Option<&CommandPaletteItem> {
        let results = self.search_results();
        results.get(self.selected_index).copied()
    }

    pub fn record_selection(&mut self, id: impl Into<CompactString>) {
        let id_comp = id.into();
        self.recent_ids.retain(|r| *r != id_comp);
        self.recent_ids.insert(0, id_comp);
        if self.recent_ids.len() > 10 {
            self.recent_ids.pop();
        }
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceShortcut {
    pub label: CompactString,
    pub link_to: CompactString,
    pub doc_view: Option<CompactString>,
    pub icon: Option<CompactString>,
    pub color: Option<CompactString>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkspaceDesktopModel {
    pub title: CompactString,
    pub icon: Option<CompactString>,
    pub shortcuts: Vec<WorkspaceShortcut>,
    pub number_cards: Vec<NumberCardModel>,
    pub chart_cards: Vec<ChartCardModel>,
}

impl WorkspaceDesktopModel {
    #[must_use]
    pub fn new(title: impl Into<CompactString>) -> Self {
        Self {
            title: title.into(),
            icon: None,
            shortcuts: Vec::new(),
            number_cards: Vec::new(),
            chart_cards: Vec::new(),
        }
    }

    pub fn add_shortcut(&mut self, shortcut: WorkspaceShortcut) {
        self.shortcuts.push(shortcut);
    }

    pub fn add_number_card(&mut self, card: NumberCardModel) {
        self.number_cards.push(card);
    }

    pub fn add_chart_card(&mut self, card: ChartCardModel) {
        self.chart_cards.push(card);
    }
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
            route: Some("/app/sales-invoice".into()),
            shortcut: Some("Cmd+Shift+I".into()),
        });
        palette.items.push(CommandPaletteItem {
            id: "2".into(),
            title: "General Ledger Report".into(),
            group: "Reports".into(),
            route: Some("/app/query-report/General Ledger".into()),
            shortcut: None,
        });

        palette.query = "sales".into();
        let results = palette.search_results();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title.as_str(), "Sales Invoice List");

        palette.query.clear();
        palette.record_selection("2");
        let recents = palette.search_results();
        assert_eq!(recents[0].id.as_str(), "2");

        palette.select_next();
        assert_eq!(palette.selected_index, 1);
        palette.select_prev();
        assert_eq!(palette.selected_index, 0);
    }

    #[test]
    fn test_file_uploader_validation_and_attachment() {
        let mut uploader = FileUploaderModel::new(vec!["png".into(), "pdf".into()], 5);

        // Disallowed extension
        assert!(uploader.validate_file("malicious.exe", 1024).is_err());

        // File too large (> 5MB)
        assert!(uploader.validate_file("large.pdf", 6 * 1024 * 1024).is_err());

        // Valid image
        let attach_res = uploader.attach_file("invoice.png", 50000, "/files/invoice.png");
        assert!(attach_res.is_ok());
        let file = attach_res.unwrap();
        assert!(file.is_image);
        assert_eq!(file.mime_type.as_str(), "image/png");
        assert_eq!(uploader.attached_files.len(), 1);

        // Remove file
        assert!(uploader.remove_file("invoice.png"));
        assert_eq!(uploader.attached_files.len(), 0);
    }

    #[test]
    fn test_workspace_desktop_model() {
        let mut ws = WorkspaceDesktopModel::new("Accounting");
        ws.add_shortcut(WorkspaceShortcut {
            label: "General Ledger".into(),
            link_to: "/app/query-report/General Ledger".into(),
            doc_view: Some("Report".into()),
            icon: Some("book".into()),
            color: Some("blue".into()),
        });
        ws.add_number_card(NumberCardModel {
            title: "Total Receivables".into(),
            value_formatted: "$120,000".into(),
            delta_percentage: Some(4.5),
            is_positive_delta: true,
            color: "green".into(),
        });
        assert_eq!(ws.shortcuts.len(), 1);
        assert_eq!(ws.number_cards.len(), 1);
    }
}
