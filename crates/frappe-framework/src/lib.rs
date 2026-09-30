pub mod ai_tools;
pub mod bpmn;
pub mod dmn;
pub mod export_engine;
pub mod jalali;
pub mod lifecycle;
pub mod localization;
pub mod notification;
pub mod print_format;
pub mod report_engine;
pub mod saga;
pub mod scripting;
pub mod typestate;
pub mod wasmtime_sandbox;
pub mod webhook;
pub mod workflow_approval;

pub use export_engine::{DocumentExporter, ExportColumn, ExportError, ExportFormat};
pub use notification::{
    NotificationChannel, NotificationDispatcher, NotificationError, NotificationInboxRegistry,
    NotificationMessage, NotificationPriority, UserNotificationPreferences,
};
pub use print_format::{InvoicePrintContext, PrintEngine, PrintLineItem, ReceiptPrintContext};
pub use report_engine::{
    PivotAggregate, PivotTableResult, ReportColumn, ReportEngine, ReportError, ReportResult,
    ReportSummaryCard,
};
pub use workflow_approval::{
    ApprovalWorkflow, DocumentVersionRecord, DocumentVersioningEngine, WorkflowApprovalLog,
    WorkflowError, WorkflowStateNode, WorkflowTransitionOutcome, WorkflowTransitionRule,
};

pub use ai_tools::{
    ErpToolCall, ErpToolDispatcher, QuotationItemDto, ToolExecutionError, ToolExecutionResult,
};
pub use bpmn::{
    ActivityType, BpmnEngine, BpmnProcessDefinition, FlowNode, ProcessInstance, SequenceFlow,
};
pub use dmn::{ConditionOp, DecisionRule, DecisionTable, HitPolicy};
pub use jalali::JalaliDate;
pub use lifecycle::{Document, DocumentController, DocumentError};
pub use localization::{
    ChineseFapiaoType, JapanInvoiceEngine, JapanTaxBreakdown, PersianNormalizer, RtlEngine,
};
pub use saga::{SagaAction, SagaCoordinator, SagaTransaction};
pub use scripting::{LifecycleEvent, RhaiHookEngine, ScriptError};
pub use typestate::{
    CancelledState, Doc, DocumentLifecycle, DocumentState, DraftState, SecurityContext,
    SubmittedState,
};
pub use wasmtime_sandbox::RealSandbox;
pub use webhook::{WebhookDispatcher, WebhookError, WebhookPayload, WebhookSubscription};

#[cfg(test)]
mod tests {
    use super::*;
    use frappe_meta::{DocFieldSchema, DocPermSchema, DocTypeSchema, FieldType};

    fn make_test_schema() -> DocTypeSchema {
        DocTypeSchema {
            name: "Sales Invoice".to_string(),
            module: "Accounts".to_string(),
            is_single: false,
            is_submittable: true,
            is_child_table: false,
            is_tree: false,
            track_changes: true,
            quick_entry: false,
            allow_rename: false,
            allow_import: true,
            allow_auto_repeat: false,
            naming_rule: Some("ACC-INV-.YYYY.-.#####".to_string()),
            naming_rule_spec: None,
            virtual_child_tables: false,
            lazy_materialization: false,
            extends_class: None,
            fields: vec![
                DocFieldSchema {
                    fieldname: "customer".to_string(),
                    fieldtype: FieldType::Data,
                    label: "Customer".to_string(),
                    reqd: true,
                    unique: false,
                    read_only: false,
                    hidden: false,
                    in_list_view: true,
                    mask: false,
                    options: None,
                    default_value: None,
                    permlevel: 0,
                },
                DocFieldSchema {
                    fieldname: "grand_total".to_string(),
                    fieldtype: FieldType::Currency,
                    label: "Grand Total".to_string(),
                    reqd: true,
                    unique: false,
                    read_only: false,
                    hidden: false,
                    in_list_view: true,
                    mask: false,
                    options: None,
                    default_value: Some(serde_json::json!(0.0)),
                    permlevel: 0,
                },
            ],
            permissions: vec![DocPermSchema {
                role: "Accounts User".to_string(),
                read: true,
                write: true,
                create: true,
                delete: false,
                submit: true,
                cancel: true,
                amend: true,
                report: true,
                export: true,
                import: false,
                permlevel: 0,
                accounting_period_exempt: false,
            }],
        }
    }

    #[test]
    fn test_rhai_operation_limit_exceeded() {
        let engine = RhaiHookEngine::new();
        let infinite_loop_script = "let i = 0; while true { i += 1; }";
        let mut doc = serde_json::json!({ "value": 10 });

        let res = engine.dispatch_hook(LifecycleEvent::Validate, &mut doc, infinite_loop_script);

        assert_eq!(res, Err(ScriptError::OperationLimitExceeded));
    }

    #[test]
    fn test_document_lifecycle_state_machine() {
        let controller = DocumentController::new();
        let schema = make_test_schema();

        let mut doc = Document::new(
            "Sales Invoice",
            serde_json::json!({
                "customer": "Acme Corp",
                "grand_total": 500.00
            }),
        );

        // 1. Insert -> Draft
        controller
            .insert(&mut doc, &schema, None, 2026, 42)
            .expect("Insert failed");
        assert_eq!(doc.name, "ACC-INV-2026-00042");
        assert_eq!(doc.docstatus, 0);
        assert!(doc.is_draft());

        // 2. Submit -> Submitted
        controller
            .submit(&mut doc, &schema, None)
            .expect("Submit failed");
        assert_eq!(doc.docstatus, 1);
        assert!(doc.is_submitted());

        // 3. Edit submitted document -> Rejected
        let update_res = controller.update(
            &mut doc,
            &schema,
            serde_json::json!({ "grand_total": 600.00 }),
            None,
        );
        assert_eq!(update_res, Err(DocumentError::CannotEditSubmittedDocument));

        // 4. Cancel -> Cancelled
        controller
            .cancel(&mut doc, &schema, None)
            .expect("Cancel failed");
        assert_eq!(doc.docstatus, 2);
        assert!(doc.is_cancelled());
    }
}
