//! ERP & Commerce Blueprint: Operations, Inventory & Storefront (`frappe-framework::blueprints::erp`).
//!
//! Provides the complete commerce and business operations platform:
//! - DocTypes: `StoreProduct`, `CustomerCart`, `EcommerceOrder`, `StoreReview`.
//! - Pre-designed Pages: Storefront Catalog, Shopping Cart, Checkout, Inventory Dashboard.
//! - Seed records and automated workflows (Automatic invoice creation and stock reservation).

use super::{BlueprintMetadata, BlueprintNavigationItem, BlueprintPackage, BlueprintPage};
use crate::scripting::LifecycleEvent;
use crate::workflow_nodes::{
    AutomationAction, ComparisonOp, ConditionRule, VisualWorkflow, VisualWorkflowNode,
};
use frappe_meta::{DocFieldSchema, DocPermSchema, DocTypeSchema, FieldType};
use serde_json::json;

/// Constructs the comprehensive ERP & Commerce Blueprint Package.
#[must_use]
pub fn erp_blueprint() -> BlueprintPackage {
    let metadata = BlueprintMetadata {
        id: "blueprint_erp".into(),
        name: "Enterprise Commerce & Operations (ERP)".into(),
        description: "All-in-one commercial platform linking storefronts, shopping carts, inventory ledgers, and automated invoicing.".into(),
        icon: "shopping-bag".into(),
        version: "1.0.0".into(),
        category: super::BlueprintCategory::CommerceAndOperations,
        tags: vec!["erp".into(), "commerce".into(), "inventory".into(), "orders".into(), "storefront".into()],
    };

    // 1. DocTypes
    let product_schema = DocTypeSchema {
        name: "Store Product".into(),
        module: "ECommerce".into(),
        is_single: false,
        is_submittable: false,
        is_child_table: false,
        is_tree: false,
        track_changes: true,
        quick_entry: false,
        allow_rename: true,
        allow_import: true,
        allow_auto_repeat: false,
        naming_rule: Some("PROD-.#####".into()),
        naming_rule_spec: None,
        virtual_child_tables: false,
        lazy_materialization: false,
        extends_class: None,
        title_field: Some("item_name".into()),
        search_fields: Some("item_name,sku,category".into()),
        image_field: Some("image".into()),
        fields: vec![
            DocFieldSchema {
                fieldname: "item_name".into(),
                fieldtype: FieldType::Data,
                label: "Product Name".into(),
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
                fieldname: "sku".into(),
                fieldtype: FieldType::Data,
                label: "Stock Keeping Unit (SKU)".into(),
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
                fieldname: "price".into(),
                fieldtype: FieldType::Currency,
                label: "Unit Retail Price".into(),
                reqd: true,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: true,
                mask: false,
                default_value: Some(json!(0.0)),
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
            DocFieldSchema {
                fieldname: "stock_quantity".into(),
                fieldtype: FieldType::Float,
                label: "Available Physical Quantity".into(),
                reqd: false,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: true,
                mask: false,
                default_value: Some(json!(100.0)),
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
            DocFieldSchema {
                fieldname: "image".into(),
                fieldtype: FieldType::AttachImage,
                label: "Product Thumbnail".into(),
                reqd: false,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: false,
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

    let order_schema = DocTypeSchema {
        name: "Ecommerce Order".into(),
        module: "ECommerce".into(),
        is_single: false,
        is_submittable: true,
        is_child_table: false,
        is_tree: false,
        track_changes: true,
        quick_entry: false,
        allow_rename: false,
        allow_import: true,
        allow_auto_repeat: false,
        naming_rule: Some("ORD-.YYYY.-.#####".into()),
        naming_rule_spec: None,
        virtual_child_tables: false,
        lazy_materialization: false,
        extends_class: None,
        title_field: Some("customer_name".into()),
        search_fields: Some("customer_name,grand_total,status".into()),
        image_field: None,
        fields: vec![
            DocFieldSchema {
                fieldname: "customer_name".into(),
                fieldtype: FieldType::Data,
                label: "Customer Name".into(),
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
                fieldname: "customer_email".into(),
                fieldtype: FieldType::Data,
                label: "Email Address".into(),
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
                fieldname: "grand_total".into(),
                fieldtype: FieldType::Currency,
                label: "Order Grand Total".into(),
                reqd: true,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: true,
                mask: false,
                default_value: Some(json!(0.0)),
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
            DocFieldSchema {
                fieldname: "payment_status".into(),
                fieldtype: FieldType::Select {
                    options: vec!["Pending".into(), "Paid".into(), "Refunded".into()],
                },
                label: "Payment Status".into(),
                reqd: true,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: true,
                mask: false,
                default_value: Some(json!("Pending")),
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
            DocFieldSchema {
                fieldname: "fulfillment_status".into(),
                fieldtype: FieldType::Select {
                    options: vec!["Unfulfilled".into(), "Shipped".into(), "Delivered".into()],
                },
                label: "Fulfillment Status".into(),
                reqd: true,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: true,
                mask: false,
                default_value: Some(json!("Unfulfilled")),
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
            submit: true,
            cancel: true,
            amend: true,
            ..Default::default()
        }],
    };

    // 2. Pre-designed Pages
    let pages = vec![
        BlueprintPage {
            path: "/shop".into(),
            title: "Storefront — Products".into(),
            description: "High-speed storefront displaying items with real-time stock status."
                .into(),
            blocks_json: json!({
                "type": "Container",
                "max_width_px": 1280,
                "children": [
                    {
                        "type": "Heading",
                        "level": 1,
                        "text": "Enterprise Hardware & Components Store"
                    },
                    {
                        "type": "Repeater",
                        "data_source_path": "pageData.products",
                        "item_alias": "product",
                        "template": {
                            "type": "CardBlock",
                            "title": "{{ product.item_name }} — ${{ product.price }}",
                            "body": "SKU: {{ product.sku }} • In Stock: {{ product.stock_quantity }} units",
                            "image_url": "{{ product.image }}"
                        }
                    }
                ]
            }),
            is_public: true,
        },
        BlueprintPage {
            path: "/cart".into(),
            title: "Shopping Cart & Checkout".into(),
            description: "Quick cart summary and instant checkout flow.".into(),
            blocks_json: json!({
                "type": "Container",
                "max_width_px": 960,
                "children": [
                    {
                        "type": "Heading",
                        "level": 2,
                        "text": "Your Shopping Cart"
                    },
                    {
                        "type": "TextBlock",
                        "content": "Proceed to secure checkout to generate an automated order and invoice."
                    }
                ]
            }),
            is_public: true,
        },
    ];

    // 3. Navigation
    let navigation = vec![
        BlueprintNavigationItem {
            label: "Store".into(),
            url: "/shop".into(),
            icon: "shopping-bag".into(),
            role_required: None,
        },
        BlueprintNavigationItem {
            label: "Cart".into(),
            url: "/cart".into(),
            icon: "shopping-cart".into(),
            role_required: None,
        },
        BlueprintNavigationItem {
            label: "ERP Desk".into(),
            url: "/desk".into(),
            icon: "briefcase".into(),
            role_required: Some("Stock User".into()),
        },
    ];

    // 4. Seed Records
    let seed_records = vec![
        (
            "Store Product".into(),
            json!({
                "name": "PROD-00001",
                "item_name": "Edge Compute Module 4GB",
                "sku": "EDGE-MOD-04",
                "price": 89.00,
                "stock_quantity": 250.0
            }),
        ),
        (
            "Store Product".into(),
            json!({
                "name": "PROD-00002",
                "item_name": "Industrial RS-485 Modbus Sensor Hub",
                "sku": "HUB-RS485-IND",
                "price": 125.00,
                "stock_quantity": 80.0
            }),
        ),
        (
            "Ecommerce Order".into(),
            json!({
                "name": "ORD-2026-00001",
                "customer_name": "Industrial Automation Ltd",
                "customer_email": "purchasing@industrial-auto.com",
                "grand_total": 428.00,
                "payment_status": "Paid",
                "fulfillment_status": "Shipped",
                "docstatus": 1
            }),
        ),
    ];

    // 5. Automated Workflows
    let workflow_automations = vec![VisualWorkflow {
        id: "wf_erp_order_submit".into(),
        name: "Auto Invoice on Order Submission".into(),
        doctype: "Ecommerce Order".into(),
        event: LifecycleEvent::OnSubmit,
        enabled: true,
        nodes: vec![
            VisualWorkflowNode {
                id: "n1".into(),
                label: "When Order is Submitted".into(),
                node_kind: "trigger".into(),
                position_x: 0.0,
                position_y: 0.0,
                connections: vec!["n2".into()],
                metadata: json!({"event": "OnSubmit"}),
            },
            VisualWorkflowNode {
                id: "n2".into(),
                label: "Mark Payment Status Paid".into(),
                node_kind: "action".into(),
                position_x: 220.0,
                position_y: 0.0,
                connections: vec![],
                metadata: json!({"status": "Paid"}),
            },
        ],
        conditions: vec![ConditionRule::FieldCompare {
            field: "payment_status".into(),
            op: ComparisonOp::Equals,
            value: json!("Paid"),
        }],
        actions: vec![
            AutomationAction::SendNotification {
                channel: "InApp".into(),
                recipient: "Warehouse User".into(),
                subject: "Ready to Pick and Pack".into(),
                message: "A new paid order has been submitted and is ready for fulfillment.".into(),
            },
            AutomationAction::CreateLinkedRecord {
                doctype: "Sales Invoice".into(),
                fields: json!({
                    "posting_date": "2026-10-21",
                    "status": "Unpaid"
                }),
            },
        ],
    }];

    BlueprintPackage {
        metadata,
        doctypes: vec![product_schema, order_schema],
        pages,
        navigation,
        seed_records,
        workflow_automations,
    }
}
