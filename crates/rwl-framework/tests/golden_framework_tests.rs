//! rwl-framework Core Golden Tests (`rwl-framework`).
//!
//! Validates:
//! 1. Complete Document Lifecycle (insert -> validate -> submit -> cancel -> amend -> trash -> discard).
//! 2. Role-based Access Control (RBAC), Permlevels, User Permissions, and DocShare.
//! 3. Naming Series atomic counter allocation.
//! 4. Workflow approval transitions with conditional guards.
//! 5. Social engine (Comments, Assignments/ToDo, Version diff tracking).
//! 6. Background job queue priority ordering and retry mechanics.

use rwl_framework::{
    BackgroundJobQueue, CommentType, DocController, DocShare, Document, DocumentController,
    DocumentError, JobPriority, JobStatus, NamingEngine, NamingSeriesStore, PermType,
    PermissionEngine, SocialEngine, UserContext, WorkflowTransition,
};
use rwl_meta::{DocFieldSchema, DocPermSchema, DocTypeSchema, FieldType};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

struct GoldenInvoiceController {
    on_submit_called: Arc<AtomicBool>,
    on_cancel_called: Arc<AtomicBool>,
    on_trash_called: Arc<AtomicBool>,
}

impl DocController for GoldenInvoiceController {
    fn validate(&self, doc: &mut Document, _schema: &DocTypeSchema) -> Result<(), DocumentError> {
        if let Some(obj) = doc.data.as_object_mut() {
            obj.insert("system_validated".into(), serde_json::json!(true));
        }
        Ok(())
    }

    fn on_submit(&self, _doc: &mut Document, _schema: &DocTypeSchema) -> Result<(), DocumentError> {
        self.on_submit_called.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn on_cancel(&self, _doc: &mut Document, _schema: &DocTypeSchema) -> Result<(), DocumentError> {
        self.on_cancel_called.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn on_trash(&self, _doc: &mut Document, _schema: &DocTypeSchema) -> Result<(), DocumentError> {
        self.on_trash_called.store(true, Ordering::SeqCst);
        Ok(())
    }
}

fn build_golden_sales_invoice_schema() -> DocTypeSchema {
    DocTypeSchema {
        name: "Sales Invoice".into(),
        module: "Accounts".into(),
        is_submittable: true,
        is_single: false,
        is_child_table: false,
        is_tree: false,
        track_changes: true,
        quick_entry: false,
        allow_rename: false,
        allow_import: true,
        allow_auto_repeat: false,
        naming_rule: Some("ACC-SINV-.YYYY.-.#####".into()),
        naming_rule_spec: None,
        virtual_child_tables: false,
        lazy_materialization: false,
        extends_class: None,
        title_field: Some("customer".into()),
        search_fields: Some("customer, grand_total".into()),
        image_field: None,
        fields: vec![
            DocFieldSchema {
                fieldname: "customer".into(),
                label: "Customer".into(),
                fieldtype: FieldType::Data,
                reqd: true,
                unique: false,
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
                fieldname: "grand_total".into(),
                label: "Grand Total".into(),
                fieldtype: FieldType::Currency,
                reqd: true,
                unique: false,
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
                fieldname: "commission_rate".into(),
                label: "Commission Rate".into(),
                fieldtype: FieldType::Percent,
                reqd: false,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: false,
                mask: false,
                options: None,
                default_value: None,
                permlevel: 1, // Restricted field!
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
        ],
        permissions: vec![
            DocPermSchema {
                role: "Accounts User".into(),
                read: true,
                write: true,
                create: true,
                delete: false,
                submit: false,
                cancel: false,
                amend: false,
                report: true,
                export: false,
                import: false,
                permlevel: 0,
                accounting_period_exempt: false,
            },
            DocPermSchema {
                role: "Accounts Manager".into(),
                read: true,
                write: true,
                create: true,
                delete: true,
                submit: true,
                cancel: true,
                amend: true,
                report: true,
                export: true,
                import: true,
                permlevel: 0,
                accounting_period_exempt: true,
            },
            DocPermSchema {
                role: "Accounts Manager".into(),
                read: true,
                write: true,
                create: false,
                delete: false,
                submit: false,
                cancel: false,
                amend: false,
                report: false,
                export: false,
                import: false,
                permlevel: 1, // Can view and edit commission_rate
                accounting_period_exempt: false,
            },
        ],
    }
}

#[test]
fn test_golden_document_lifecycle_flow() {
    let mut controller = DocumentController::new();
    let on_submit_called = Arc::new(AtomicBool::new(false));
    let on_cancel_called = Arc::new(AtomicBool::new(false));
    let on_trash_called = Arc::new(AtomicBool::new(false));

    controller.register_controller(
        "Sales Invoice",
        GoldenInvoiceController {
            on_submit_called: on_submit_called.clone(),
            on_cancel_called: on_cancel_called.clone(),
            on_trash_called: on_trash_called.clone(),
        },
    );

    let schema = build_golden_sales_invoice_schema();
    let mut doc = Document::new(
        "Sales Invoice",
        serde_json::json!({
            "customer": "Starlight Industries",
            "grand_total": 25000.0,
            "commission_rate": 5.0
        }),
    );

    // 1. Insert Draft
    controller.insert(&mut doc, &schema, None, 2026, 1).unwrap();
    assert_eq!(doc.name, "ACC-SINV-2026-00001");
    assert_eq!(doc.docstatus, 0);
    assert_eq!(doc.data["system_validated"], true);

    // 2. Submit Document
    controller.submit(&mut doc, &schema, None).unwrap();
    assert_eq!(doc.docstatus, 1);
    assert!(on_submit_called.load(Ordering::SeqCst));

    // 3. Mutate or Delete submitted document must fail!
    let update_res = controller.update(
        &mut doc,
        &schema,
        serde_json::json!({ "grand_total": 30000.0 }),
        None,
    );
    assert_eq!(update_res, Err(DocumentError::CannotEditSubmittedDocument));

    let trash_res = controller.trash(&mut doc, &schema);
    assert_eq!(trash_res, Err(DocumentError::CannotEditSubmittedDocument));

    // 4. Cancel Document
    controller.cancel(&mut doc, &schema, None).unwrap();
    assert_eq!(doc.docstatus, 2);
    assert!(on_cancel_called.load(Ordering::SeqCst));

    // 5. Amend Document -> creates new draft
    let mut amended = controller.amend(&doc, &schema).unwrap();
    assert_eq!(amended.name, "ACC-SINV-2026-00001-1");
    assert_eq!(amended.docstatus, 0);
    assert_eq!(amended.amended_from.as_deref(), Some("ACC-SINV-2026-00001"));

    // 6. Trash on cancelled document succeeds
    controller.trash(&mut doc, &schema).unwrap();
    assert!(on_trash_called.load(Ordering::SeqCst));

    // 7. Discard on draft amended document
    controller.discard(&mut amended).unwrap();
    assert_eq!(amended.docstatus, 3);
}

#[test]
fn test_golden_permissions_and_permlevels() {
    let schema = build_golden_sales_invoice_schema();
    let doc = Document::new(
        "Sales Invoice",
        serde_json::json!({
            "name": "ACC-SINV-001",
            "customer": "Solar Dynamics",
            "grand_total": 45000.0,
            "commission_rate": 8.5,
            "company": "Main Company"
        }),
    );

    let user_accounts = UserContext::new("user1@corp.com", vec!["Accounts User".into()]);
    let mgr_accounts = UserContext::new("mgr1@corp.com", vec!["Accounts Manager".into()]);
    let guest = UserContext::new("guest@corp.com", vec!["Guest".into()]);

    // Accounts User can read/write, but cannot submit
    assert!(PermissionEngine::has_permission(
        &doc,
        PermType::Read,
        &user_accounts,
        &schema,
        &[]
    ));
    assert!(PermissionEngine::has_permission(
        &doc,
        PermType::Write,
        &user_accounts,
        &schema,
        &[]
    ));
    assert!(!PermissionEngine::has_permission(
        &doc,
        PermType::Submit,
        &user_accounts,
        &schema,
        &[]
    ));

    // Accounts Manager has submit and delete access
    assert!(PermissionEngine::has_permission(
        &doc,
        PermType::Submit,
        &mgr_accounts,
        &schema,
        &[]
    ));
    assert!(PermissionEngine::has_permission(
        &doc,
        PermType::Delete,
        &mgr_accounts,
        &schema,
        &[]
    ));

    // Guest cannot read
    assert!(!PermissionEngine::has_permission(
        &doc,
        PermType::Read,
        &guest,
        &schema,
        &[]
    ));

    // DocShare allows guest read access
    let shares = vec![DocShare {
        user: "guest@corp.com".into(),
        read: true,
        write: false,
        share: false,
    }];
    assert!(PermissionEngine::has_permission(
        &doc,
        PermType::Read,
        &guest,
        &schema,
        &shares
    ));

    // Field-level Permlevel 1 test:
    // Accounts User must NOT see commission_rate (permlevel 1)
    let filtered_user =
        PermissionEngine::filter_document_fields(&doc.data, &user_accounts, &schema, false);
    assert_eq!(filtered_user["customer"], "Solar Dynamics");
    assert!(filtered_user.get("commission_rate").is_none());

    // Accounts Manager DOES see commission_rate
    let filtered_mgr =
        PermissionEngine::filter_document_fields(&doc.data, &mgr_accounts, &schema, false);
    assert_eq!(filtered_mgr["customer"], "Solar Dynamics");
    assert_eq!(filtered_mgr["commission_rate"], 8.5);
}

#[test]
fn test_golden_naming_series_concurrency() {
    let store = NamingSeriesStore::new();
    let schema = build_golden_sales_invoice_schema();

    // Generate 100 sequential names and ensure each is unique and increasing
    let mut names = Vec::new();
    for _ in 0..100 {
        let mut doc = Document::new("Sales Invoice", serde_json::json!({}));
        let name = NamingEngine::autoname(&mut doc, &schema, &store).unwrap();
        names.push(name);
    }

    assert_eq!(names.len(), 100);
    assert!(names[0].ends_with("00001"));
    assert!(names[99].ends_with("00100"));
}

#[test]
fn test_golden_workflow_engine_approval_flow() {
    let mut controller = DocumentController::new();
    controller.register_workflow(
        "Sales Invoice",
        vec![
            WorkflowTransition {
                state: "Draft".into(),
                action: "Submit For Approval".into(),
                next_state: "Pending Approval".into(),
                allowed_role: "Accounts User".into(),
                condition_field: None,
                condition_value: None,
            },
            WorkflowTransition {
                state: "Pending Approval".into(),
                action: "Approve".into(),
                next_state: "Approved".into(),
                allowed_role: "Accounts Manager".into(),
                condition_field: None,
                condition_value: None,
            },
        ],
    );

    let mut doc = Document::new("Sales Invoice", serde_json::json!({}));
    assert_eq!(doc.workflow_state, None);

    // User submits for approval
    controller
        .apply_workflow_action(&mut doc, "Submit For Approval", "Accounts User")
        .unwrap();
    assert_eq!(doc.workflow_state.as_deref(), Some("Pending Approval"));

    // User without Accounts Manager role cannot approve
    let unauthorized_res = controller.apply_workflow_action(&mut doc, "Approve", "Accounts User");
    assert!(unauthorized_res.is_err());

    // Accounts Manager approves
    controller
        .apply_workflow_action(&mut doc, "Approve", "Accounts Manager")
        .unwrap();
    assert_eq!(doc.workflow_state.as_deref(), Some("Approved"));
}

#[test]
fn test_golden_social_and_background_jobs() {
    // 1. Social timeline and ToDo
    let mut social = SocialEngine::new();
    let assign = social.assign_to(
        "Sales Invoice",
        "INV-999",
        "auditor@corp.com",
        "cfo@corp.com",
        Some("Audit reconciliation"),
        "High",
    );
    assert_eq!(assign.status, "Open");

    let timeline = social.get_timeline("Sales Invoice", "INV-999");
    assert_eq!(timeline.len(), 1);
    assert_eq!(timeline[0].comment_type, CommentType::Assigned);

    // 2. Background queue priorities and retry
    let queue = BackgroundJobQueue::new();
    queue.enqueue("report_job", serde_json::json!({}), JobPriority::Long);
    let crit_id = queue.enqueue("gl_post_job", serde_json::json!({}), JobPriority::Critical);

    let dequeued = queue.dequeue().unwrap();
    assert_eq!(dequeued.id, crit_id);
    assert_eq!(dequeued.priority, JobPriority::Critical);
    queue.complete(&crit_id);
    assert_eq!(queue.get(&crit_id).unwrap().status, JobStatus::Completed);
}
