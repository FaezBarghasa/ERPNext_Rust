pub mod forms;
pub mod grid;
pub mod signals;
pub mod video_hud;

pub use forms::{eval_depends_on, DynamicFormModel, FormFieldWidget};
pub use grid::visible_slice;
pub use video_hud::{ChapterMarker, VideoHudState};

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
            track_changes: true,
            naming_rule: None,
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
                    options: None,
                    default_value: None,
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
                    options: None,
                    default_value: Some(serde_json::json!(0.0)),
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
                    options: None,
                    default_value: None,
                },
            ],
            permissions: vec![DocPermSchema::default()],
        };

        let model = DynamicFormModel::from_schema(&schema);
        assert_eq!(model.doctype_name, "Customer");
        assert_eq!(model.widgets.len(), 3);
        assert!(matches!(model.widgets[0], FormFieldWidget::TextInput { .. }));
        assert!(matches!(model.widgets[1], FormFieldWidget::CurrencyInput { .. }));
        assert!(matches!(model.widgets[2], FormFieldWidget::LinkDropdown { .. }));
    }

    #[test]
    fn test_depends_on_evaluator() {
        let mut values = HashMap::new();
        values.insert("status".into(), "Open".into());
        values.insert("workflow_state".into(), "Pending".into());

        assert!(eval_depends_on("status==Open", &values));
        assert!(!eval_depends_on("status==Closed", &values));
        assert!(eval_depends_on("status==Open && workflow_state==Pending", &values));
        assert!(!eval_depends_on("status==Open && workflow_state==Approved", &values));
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
