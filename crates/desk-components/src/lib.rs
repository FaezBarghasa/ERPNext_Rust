pub mod cpq_view;
pub mod desk_shell;
pub mod form_engine;
pub mod forms;
pub mod gantt;
pub mod grid;
pub mod primitives;
pub mod report_query;
pub mod resources;
pub mod signals;
pub mod spc_view;
pub mod views;
pub mod visual_builder;
pub mod wms_view;

pub use visual_builder::{
    AccessibilityAuditWcag, ApplicationMode, BoxModelProperties, ComponentCategory,
    CoreWebVitalsBudget, DevicePreset, FluidTypography, GridSystemConfig, GridType,
    LaunchReadinessGate, MotionTimeline, OklchColor, PaletteItem, SnapSettings, SssRankState,
    StudioThemeMode, VisualStudioState, W3cDesignTokens, get_component_palette,
    render_visual_builder_html,
};

pub use report_query::{
    DeclarativeReport, FilterOperator, ReportDataChunk, ReportFilter, ReportPagination,
    ScriptReportColumn,
};

pub use desk_shell::{
    AwesomeBar, DeskShell, LanguageDirection, MenuItem, MenuTree, MenuTreeLocation, ThemeConfig,
    ThemeMode, WorkspaceModule, format_currency, get_desk_workspaces, is_rtl,
    render_desk_shell_component, render_desk_shell_html, render_form_view_html,
    render_list_view_html,
};

pub use cpq_view::{CpqConfiguratorModel, OptionCard};
pub use form_engine::{
    ChildGridColumn, ChildTableGridModel, ClientScriptEngine, ClientScriptError,
    CompiledFormLayout, DynamicFieldState, DynamicFormEvaluator, FieldInspectorInfo, FormColumn,
    FormLayoutEngine, FormLayoutWidget, FormSection, FormTab, FormTimelineEvent, FormTimelineModel,
};
pub use forms::{
    ChildTableRow, ChildTableState, ConditionGroup, ConditionOperator, ConditionRule, DocTypeForm,
    DynamicFormModel, FieldValidationRule, FormDraft, FormFieldWidget, FormStateTracker,
    FormValidator, LogicalCombinator, ValidationReport, eval_depends_on,
    render_doctype_form_component,
};
pub use gantt::{GanttDependencyLink, GanttTaskRow, GanttViewModel};
pub use grid::{
    CellCoordinate, CellSelectionRange, EditableChildTableGrid, GridNavDirection,
    VirtualizedDataGrid, VirtualizedGridState, render_virtualized_grid_component, visible_slice,
};
pub use primitives::{
    AlertModel, AlertVariant, AttachedFile, AutocompleteModel, AutocompleteOption, AvatarModel,
    AvatarSize, AwesomeBarItem, AwesomeBarKeyboardAction, AwesomeBarModel, BadgeModel,
    BottomSheetModel, ButtonModel, ButtonVariant, CalendarEvent, CalendarViewMode,
    CalendarViewModel, CardModel, ChartCardModel, ComboboxModel, ComboboxOption,
    CommandPaletteItem, CommandPaletteModel, ContextMenuItem, ContextMenuModel, DashboardViewModel,
    DatePickerModel, DateTimePickerModel, DialogModel, FileUploaderModel, FilterCondition,
    FilterGroupModel, FilterLogicalOperator, KeyboardShortcutsDialogModel, MobileNavItem,
    MobileShellModel, MultiSelectModel, NumberCardModel, RatingModel, ShortcutGuideItem,
    SliderModel, TagInputModel, TimePickerModel, TimelineEventType, TimelineItem, TimelineModel,
    ToastModel, TooltipModel, TreeNode, TreeViewModel, WorkspaceDesktopModel, WorkspaceShortcut,
};
pub use resources::{
    ColorScheme, DocumentResourceState, KeyboardShortcutConfig, ListResourceState,
    PageMetaComposable, ResourceState,
};
pub use spc_view::{SpcChartViewModel, SpcPointView};
pub use views::{
    BatchActionBarModel, BatchActionType, FilterCriterion, FilterDrawerModel, FormTimelineEntry,
    FormViewModel, KanbanColumn, KanbanViewModel, ListViewColumn, ListViewModel, ReportType,
    ReportViewModel,
};
pub use wms_view::{AmrMarkerViewModel, BinViewModel, Warehouse3DViewModel};

#[cfg(test)]
mod tests {
    use super::*;
    use rwl_meta::{DocFieldSchema, DocPermSchema, DocTypeSchema, FieldType};
    use std::collections::HashMap;

    #[test]
    fn test_dynamic_form_compiler() {
        let schema = DocTypeSchema {
            name: "Customer".into(),
            module: "Selling".into(),
            is_single: false,
            is_submittable: false,
            is_child_table: false,
            is_tree: false,
            track_changes: true,
            quick_entry: false,
            allow_rename: false,
            allow_import: true,
            allow_auto_repeat: false,
            naming_rule: None,
            naming_rule_spec: None,
            virtual_child_tables: false,
            lazy_materialization: false,
            extends_class: None,
            title_field: Some("customer_name".into()),
            search_fields: Some("customer_name".into()),
            image_field: None,
            fields: vec![
                DocFieldSchema {
                    fieldname: "customer_name".into(),
                    fieldtype: FieldType::Data,
                    label: "Customer Name".into(),
                    reqd: true,
                    unique: true,
                    read_only: false,
                    hidden: false,
                    in_list_view: true,
                    mask: false,
                    options: None,
                    default_value: None,
                    permlevel: 0,
                    depends_on: None,
                    mandatory_depends_on: None,
                    read_only_depends_on: None,
                    fetch_from: None,
                    precision: None,
                },
                DocFieldSchema {
                    fieldname: "credit_limit".into(),
                    fieldtype: FieldType::Currency,
                    label: "Credit Limit".into(),
                    reqd: false,
                    unique: false,
                    read_only: false,
                    hidden: false,
                    in_list_view: false,
                    mask: false,
                    options: None,
                    default_value: Some(serde_json::json!(0.0)),
                    permlevel: 0,
                    depends_on: None,
                    mandatory_depends_on: None,
                    read_only_depends_on: None,
                    fetch_from: None,
                    precision: None,
                },
                DocFieldSchema {
                    fieldname: "default_currency".into(),
                    fieldtype: FieldType::Link {
                        target_doctype: "Currency".into(),
                    },
                    label: "Currency".into(),
                    reqd: false,
                    unique: false,
                    read_only: false,
                    hidden: false,
                    in_list_view: false,
                    mask: false,
                    options: None,
                    default_value: None,
                    permlevel: 0,
                    depends_on: None,
                    mandatory_depends_on: None,
                    read_only_depends_on: None,
                    fetch_from: None,
                    precision: None,
                },
            ],
            permissions: vec![DocPermSchema::default()],
        };

        let model = DynamicFormModel::from_schema(&schema);
        assert_eq!(model.doctype_name, "Customer");
        assert_eq!(model.widgets.len(), 3);
        assert!(matches!(
            model.widgets[0],
            FormFieldWidget::TextInput { .. }
        ));
        assert!(matches!(
            model.widgets[1],
            FormFieldWidget::CurrencyInput { .. }
        ));
        assert!(matches!(
            model.widgets[2],
            FormFieldWidget::LinkDropdown { .. }
        ));
    }

    #[test]
    fn test_depends_on_evaluator() {
        let mut values = HashMap::new();
        values.insert("status".into(), "Open".into());
        values.insert("workflow_state".into(), "Pending".into());

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
    }
}
