//! SaaS Blueprint: Multi-Tenant Subscription SaaS Portal (`rwl-framework::blueprints::saas`).
//!
//! Provides the complete subscription business platform:
//! - DocTypes: `SubscriptionPlan`, `CustomerAccount`, `ApiKey`, `UsageRecord`.
//! - Pre-designed Pages: Pricing Table, Signup Flow, Customer Billing Portal.
//! - Seed records and automated workflows (Welcome onboarding and API key generation).

use super::{BlueprintMetadata, BlueprintNavigationItem, BlueprintPackage, BlueprintPage};
use crate::scripting::LifecycleEvent;
use crate::workflow_nodes::{
    AutomationAction, ComparisonOp, ConditionRule, VisualWorkflow, VisualWorkflowNode,
};
use rwl_meta::{DocFieldSchema, DocPermSchema, DocTypeSchema, FieldType};
use serde_json::json;

/// Constructs the comprehensive SaaS Blueprint Package.
#[must_use]
pub fn saas_blueprint() -> BlueprintPackage {
    let metadata = BlueprintMetadata {
        id: "blueprint_saas".into(),
        name: "Subscription SaaS & API Portal".into(),
        description: "Turnkey multi-tier subscription platform with customer accounts, API keys, usage metering, and billing portal.".into(),
        icon: "credit-card".into(),
        version: "1.0.0".into(),
        category: super::BlueprintCategory::SubscriptionSaaS,
        tags: vec!["saas".into(), "subscription".into(), "billing".into(), "api-keys".into(), "portal".into()],
    };

    // 1. DocTypes
    let plan_schema = DocTypeSchema {
        name: "Subscription Plan".into(),
        module: "Subscription".into(),
        is_single: false,
        is_submittable: false,
        is_child_table: false,
        is_tree: false,
        track_changes: true,
        quick_entry: true,
        allow_rename: true,
        allow_import: true,
        allow_auto_repeat: false,
        naming_rule: None,
        naming_rule_spec: None,
        virtual_child_tables: false,
        lazy_materialization: false,
        extends_class: None,
        title_field: Some("plan_name".into()),
        search_fields: Some("plan_name,tier".into()),
        image_field: None,
        fields: vec![
            DocFieldSchema {
                fieldname: "plan_name".into(),
                fieldtype: FieldType::Data,
                label: "Plan Name".into(),
                reqd: true,
                unique: true,
                read_only: false,
                hidden: false,
                in_list_view: true,
                mask: false,
                default_value: None,
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
            DocFieldSchema {
                fieldname: "tier".into(),
                fieldtype: FieldType::Select {
                    options: vec!["Starter".into(), "Pro".into(), "Enterprise".into()],
                },
                label: "Tier".into(),
                reqd: true,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: true,
                mask: false,
                default_value: Some(json!("Starter")),
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
            DocFieldSchema {
                fieldname: "monthly_price".into(),
                fieldtype: FieldType::Currency,
                label: "Monthly Price (USD)".into(),
                reqd: true,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: true,
                mask: false,
                default_value: Some(json!(29.0)),
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
            DocFieldSchema {
                fieldname: "annual_price".into(),
                fieldtype: FieldType::Currency,
                label: "Annual Price (USD)".into(),
                reqd: true,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: true,
                mask: false,
                default_value: Some(json!(290.0)),
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
            DocFieldSchema {
                fieldname: "monthly_api_calls".into(),
                fieldtype: FieldType::Int,
                label: "Monthly Included API Calls".into(),
                reqd: true,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: true,
                mask: false,
                default_value: Some(json!(100_000)),
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
        ],
        permissions: vec![DocPermSchema {
            role: "All".into(),
            permlevel: 0,
            read: true,
            write: true,
            create: true,
            delete: true,
            submit: false,
            cancel: false,
            amend: false,
            ..Default::default()
        }],
    };

    let account_schema = DocTypeSchema {
        name: "Customer Account".into(),
        module: "Subscription".into(),
        is_single: false,
        is_submittable: false,
        is_child_table: false,
        is_tree: false,
        track_changes: true,
        quick_entry: false,
        allow_rename: true,
        allow_import: true,
        allow_auto_repeat: false,
        naming_rule: Some("ACC-.#####".into()),
        naming_rule_spec: None,
        virtual_child_tables: false,
        lazy_materialization: false,
        extends_class: None,
        title_field: Some("company_name".into()),
        search_fields: Some("company_name,billing_email,status".into()),
        image_field: None,
        fields: vec![
            DocFieldSchema {
                fieldname: "company_name".into(),
                fieldtype: FieldType::Data,
                label: "Company / Team Name".into(),
                reqd: true,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: true,
                mask: false,
                default_value: None,
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
            DocFieldSchema {
                fieldname: "billing_email".into(),
                fieldtype: FieldType::Data,
                label: "Billing Email Address".into(),
                reqd: true,
                unique: true,
                read_only: false,
                hidden: false,
                in_list_view: true,
                mask: false,
                default_value: None,
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
            DocFieldSchema {
                fieldname: "plan".into(),
                fieldtype: FieldType::Link {
                    target_doctype: "Subscription Plan".into(),
                },
                label: "Active Plan".into(),
                reqd: true,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: true,
                mask: false,
                default_value: None,
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
            DocFieldSchema {
                fieldname: "billing_cycle".into(),
                fieldtype: FieldType::Select {
                    options: vec!["Monthly".into(), "Annual".into()],
                },
                label: "Billing Cadence".into(),
                reqd: true,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: true,
                mask: false,
                default_value: Some(json!("Monthly")),
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
            DocFieldSchema {
                fieldname: "status".into(),
                fieldtype: FieldType::Select {
                    options: vec!["Active".into(), "PastDue".into(), "Cancelled".into()],
                },
                label: "Account Status".into(),
                reqd: true,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: true,
                mask: false,
                default_value: Some(json!("Active")),
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
            DocFieldSchema {
                fieldname: "api_key".into(),
                fieldtype: FieldType::Password,
                label: "Primary Production API Key".into(),
                reqd: false,
                unique: false,
                read_only: true,
                hidden: false,
                in_list_view: false,
                mask: true,
                default_value: None,
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
        ],
        permissions: vec![DocPermSchema {
            role: "All".into(),
            permlevel: 0,
            read: true,
            write: true,
            create: true,
            delete: true,
            submit: false,
            cancel: false,
            amend: false,
            ..Default::default()
        }],
    };

    // 2. Pre-designed Pages
    let pages = vec![
        BlueprintPage {
            path: "/pricing".into(),
            title: "Pricing & Plans — Scale Confidently".into(),
            description:
                "Transparent pricing table with monthly/annual toggle and instant checkout.".into(),
            blocks_json: json!({
                "type": "Container",
                "max_width_px": 1280,
                "children": [
                    {
                        "type": "Heading",
                        "level": 1,
                        "text": "Simple, Predictable Infrastructure Pricing"
                    },
                    {
                        "type": "TextBlock",
                        "content": "No hidden metered charges. Everything you need to scale on a budget."
                    },
                    {
                        "type": "Repeater",
                        "data_source_path": "pageData.plans",
                        "item_alias": "plan",
                        "template": {
                            "type": "CardBlock",
                            "title": "{{ plan.plan_name }} — ${{ plan.monthly_price }}/mo",
                            "body": "Tier: {{ plan.tier }} • Includes {{ plan.monthly_api_calls }} API requests/mo",
                            "image_url": null
                        }
                    }
                ]
            }),
            is_public: true,
        },
        BlueprintPage {
            path: "/account/billing".into(),
            title: "Customer Billing Portal".into(),
            description: "Manage subscription plan, invoices, and API credentials.".into(),
            blocks_json: json!({
                "type": "Container",
                "max_width_px": 960,
                "children": [
                    {
                        "type": "Heading",
                        "level": 2,
                        "text": "Subscription & Usage Overview"
                    },
                    {
                        "type": "TextBlock",
                        "content": "Active Plan: {{ account.plan }} (Billing: {{ account.billing_cycle }})"
                    }
                ]
            }),
            is_public: false,
        },
    ];

    // 3. Navigation
    let navigation = vec![
        BlueprintNavigationItem {
            label: "Pricing".into(),
            url: "/pricing".into(),
            icon: "tag".into(),
            role_required: None,
        },
        BlueprintNavigationItem {
            label: "Billing".into(),
            url: "/account/billing".into(),
            icon: "credit-card".into(),
            role_required: Some("Customer".into()),
        },
        BlueprintNavigationItem {
            label: "SaaS Admin".into(),
            url: "/desk".into(),
            icon: "shield-check".into(),
            role_required: Some("System Manager".into()),
        },
    ];

    // 4. Seed Records
    let seed_records = vec![
        (
            "Subscription Plan".into(),
            json!({
                "name": "Starter",
                "plan_name": "Starter",
                "tier": "Starter",
                "monthly_price": 29.00,
                "annual_price": 290.00,
                "monthly_api_calls": 50_000
            }),
        ),
        (
            "Subscription Plan".into(),
            json!({
                "name": "Pro",
                "plan_name": "Pro",
                "tier": "Pro",
                "monthly_price": 79.00,
                "annual_price": 790.00,
                "monthly_api_calls": 250_000
            }),
        ),
        (
            "Subscription Plan".into(),
            json!({
                "name": "Enterprise",
                "plan_name": "Enterprise",
                "tier": "Enterprise",
                "monthly_price": 299.00,
                "annual_price": 2990.00,
                "monthly_api_calls": 2_000_000
            }),
        ),
        (
            "Customer Account".into(),
            json!({
                "name": "ACC-00001",
                "company_name": "Apex Cloud Systems",
                "billing_email": "ops@apexcloud.io",
                "plan": "Pro",
                "billing_cycle": "Monthly",
                "status": "Active",
                "api_key": "rwl_live_8fbc29a01e44"
            }),
        ),
    ];

    // 5. Automated Workflows
    let workflow_automations = vec![VisualWorkflow {
        id: "wf_saas_welcome_account".into(),
        name: "Onboard New Customer Account".into(),
        doctype: "Customer Account".into(),
        event: LifecycleEvent::Validate,
        enabled: true,
        nodes: vec![
            VisualWorkflowNode {
                id: "n1".into(),
                label: "Generate API Key if Missing".into(),
                node_kind: "condition".into(),
                position_x: 0.0,
                position_y: 0.0,
                connections: vec!["n2".into()],
                metadata: json!({"field": "status", "val": "Active"}),
            },
            VisualWorkflowNode {
                id: "n2".into(),
                label: "Send Welcome Onboarding Email".into(),
                node_kind: "action".into(),
                position_x: 200.0,
                position_y: 0.0,
                connections: vec![],
                metadata: json!({"channel": "Email"}),
            },
        ],
        conditions: vec![ConditionRule::FieldCompare {
            field: "status".into(),
            op: ComparisonOp::Equals,
            value: json!("Active"),
        }],
        actions: vec![AutomationAction::SendNotification {
            channel: "Email".into(),
            recipient: "billing@customer.com".into(),
            subject: "Welcome to rust-web-land Platform".into(),
            message: "Your account is activated and your API credentials are ready.".into(),
        }],
    }];

    BlueprintPackage {
        metadata,
        doctypes: vec![plan_schema, account_schema],
        pages,
        navigation,
        seed_records,
        workflow_automations,
    }
}
