pub mod cpq_view;
pub mod desk_shell;
pub mod forms;
pub mod gantt;
pub mod grid;
pub mod primitives;
pub mod resources;
pub mod signals;
pub mod spc_view;
pub mod video_hud;
pub mod views;
pub mod wms_view;

pub use desk_shell::{WorkspaceModule, get_desk_workspaces, render_desk_shell_html};

pub use cpq_view::{CpqConfiguratorModel, OptionCard};
pub use forms::{DynamicFormModel, FormFieldWidget, eval_depends_on};
pub use gantt::{GanttDependencyLink, GanttTaskRow, GanttViewModel};
pub use grid::{
    CellCoordinate, CellSelectionRange, GridNavDirection, VirtualizedGridState, visible_slice,
};
pub use primitives::{
    AlertModel, AlertVariant, AutocompleteModel, AutocompleteOption, AvatarModel, AvatarSize,
    BadgeModel, ButtonModel, ButtonVariant, CardModel, CommandPaletteItem, CommandPaletteModel,
    DialogModel, FileUploaderModel, MultiSelectModel, RatingModel, SliderModel, ToastModel,
    TooltipModel,
};
pub use resources::{
    ColorScheme, DocumentResourceState, KeyboardShortcutConfig, ListResourceState,
    PageMetaComposable, ResourceState,
};
pub use spc_view::{SpcChartViewModel, SpcPointView};
pub use video_hud::{ChapterMarker, VideoHudState};
pub use views::{
    FormTimelineEntry, FormViewModel, KanbanColumn, KanbanViewModel, ListViewColumn, ListViewModel,
    ReportType, ReportViewModel,
};
pub use wms_view::{AmrMarkerViewModel, BinViewModel, Warehouse3DViewModel};

#[cfg(test)]
mod tests {
    use super::*;
    use frappe_meta::{DocFieldSchema, DocPermSchema, DocTypeSchema, FieldType};
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

    #[test]
    fn test_video_hud_state() {
        let mut hud = VideoHudState::new(
            "VID-001",
            120.0,
            vec![
                ChapterMarker {
                    timestamp_secs: 0.0,
                    title: "Introduction".into(),
                },
                ChapterMarker {
                    timestamp_secs: 45.0,
                    title: "Core Concepts".into(),
                },
            ],
        );

        assert_eq!(hud.current_time_secs, 0.0);
        hud.toggle_play();
        assert!(hud.is_playing);

        hud.seek_to(50.0);
        assert_eq!(hud.current_time_secs, 50.0);

        hud.push_comment("Great explanation!".into());
        assert_eq!(hud.comments.len(), 1);
    }
}
