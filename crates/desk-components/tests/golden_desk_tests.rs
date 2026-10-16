//! Upstream ERPNext Desk UI, Form Engine & Reports Golden Tests (`desk-components`).
//!
//! Validates Phase 7 requirements:
//! 1. Desk UI widgets (Combobox, Date/Time Pickers, TreeView, ContextMenu, Calendar, Dashboard, MobileShell).
//! 2. Form view layout engine driven by DocType JSON schema (Tabs, Sections, Columns, Child tables, Alt-key field inspector, Timeline).
//! 3. Declarative Script Report execution (typed filtering, pagination, and grand totals calculation).

use desk_components::{
    CalendarEvent, CalendarViewModel, CalendarViewMode, ChartCardModel, ComboboxModel,
    ComboboxOption, ContextMenuItem, ContextMenuModel, DashboardViewModel, DatePickerModel,
    DeclarativeReport, FilterOperator, FormLayoutEngine, FormLayoutWidget, FormTimelineEvent,
    MobileNavItem, MobileShellModel, NumberCardModel, ReportFilter, ReportPagination,
    ScriptReportColumn, TimePickerModel, TreeNode, TreeViewModel,
};
use frappe_meta::{DocFieldSchema, DocTypeSchema, FieldType};
use rust_decimal_macros::dec;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_golden_desk_widgets_suite() {
    // 1. Combobox with filtering
    let options = vec![
        ComboboxOption {
            label: "EUR - Euro".into(),
            value: "EUR".into(),
            description: Some("European Union Currency".into()),
        },
        ComboboxOption {
            label: "USD - US Dollar".into(),
            value: "USD".into(),
            description: Some("United States Currency".into()),
        },
        ComboboxOption {
            label: "IRR - Iranian Rial".into(),
            value: "IRR".into(),
            description: Some("Iranian National Currency".into()),
        },
    ];
    let mut combobox = ComboboxModel::new("Select currency...", options);
    combobox.query = "eur".into();
    let filtered = combobox.filtered_options();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].value.as_str(), "EUR");

    // 2. Date and Time Pickers
    let date_picker = DatePickerModel::new("Posting Date", Some("2026-10-16"));
    assert_eq!(date_picker.value.as_deref(), Some("2026-10-16"));

    let time_picker = TimePickerModel {
        label: "Shift Start".into(),
        value: Some("09:00:00".into()),
        step_minutes: 15,
        is_required: true,
    };
    assert_eq!(time_picker.step_minutes, 15);

    // 3. Tree View (Chart of Accounts / Department tree)
    let child1 = TreeNode::new("1110", "Cash in Hand", Some("1100"), false);
    let child2 = TreeNode::new("1120", "Bank Accounts", Some("1100"), true);
    let mut parent = TreeNode::new("1100", "Current Assets", None::<&str>, true);
    parent.children.push(child1);
    parent.children.push(child2);

    let tree = TreeViewModel::new(vec![parent]);
    assert_eq!(tree.root_nodes.len(), 1);
    assert_eq!(tree.root_nodes[0].children.len(), 2);

    // 4. Context Menu
    let context_menu = ContextMenuModel {
        x: 250.0,
        y: 180.0,
        items: vec![
            ContextMenuItem {
                id: "edit".into(),
                label: "Edit Row".into(),
                icon: Some("pencil".into()),
                is_disabled: false,
                is_danger: false,
                shortcut: Some("E".into()),
            },
            ContextMenuItem {
                id: "delete".into(),
                label: "Delete Row".into(),
                icon: Some("trash".into()),
                is_disabled: false,
                is_danger: true,
                shortcut: Some("Del".into()),
            },
        ],
        is_open: true,
    };
    assert_eq!(context_menu.items.len(), 2);
    assert!(context_menu.items[1].is_danger);

    // 5. Calendar View
    let calendar = CalendarViewModel {
        doctype: "Event".into(),
        mode: CalendarViewMode::Month,
        current_date_iso: "2026-10-16".into(),
        events: vec![CalendarEvent {
            id: "EVT-001".into(),
            title: "Q4 Strategy Review".into(),
            start_iso: "2026-10-16T10:00:00Z".into(),
            end_iso: "2026-10-16T12:00:00Z".into(),
            color: "blue".into(),
            all_day: false,
        }],
    };
    assert_eq!(calendar.events.len(), 1);

    // 6. Dashboard & Number Cards
    let dashboard = DashboardViewModel {
        dashboard_name: "Executive Overview".into(),
        number_cards: vec![NumberCardModel {
            title: "Total Revenue".into(),
            value_formatted: "$1,450,000".into(),
            delta_percentage: Some(18.5),
            is_positive_delta: true,
            color: "green".into(),
        }],
        chart_cards: vec![ChartCardModel {
            title: "Monthly Sales Trend".into(),
            chart_type: "line".into(),
            labels: vec!["Jul".into(), "Aug".into(), "Sep".into(), "Oct".into()],
            datasets: vec![vec![120.0, 150.0, 180.0, 210.0]],
        }],
    };
    assert_eq!(dashboard.number_cards.len(), 1);
    assert_eq!(dashboard.chart_cards.len(), 1);

    // 7. Mobile Shell
    let mobile = MobileShellModel {
        active_route: "/app/sales-invoice".into(),
        nav_items: vec![
            MobileNavItem {
                route: "/app/home".into(),
                label: "Home".into(),
                icon: "home".into(),
                badge: None,
            },
            MobileNavItem {
                route: "/app/sales-invoice".into(),
                label: "Invoices".into(),
                icon: "file-text".into(),
                badge: Some(3),
            },
        ],
        bottom_bar_visible: true,
        offline_indicator: false,
    };
    assert_eq!(mobile.nav_items.len(), 2);
    assert_eq!(mobile.nav_items[1].badge, Some(3));
}

#[test]
fn test_golden_doctype_driven_form_layout_engine() {
    // Construct a realistic Sales Invoice DocTypeSchema with Tabs, Sections, Columns, and Child Table
    let fields = vec![
        // Tab 1: Details
        DocFieldSchema {
            fieldname: "tab_details".into(),
            label: "Invoice Details".into(),
            fieldtype: FieldType::TabBreak,
            reqd: false,
            read_only: false,
            hidden: false,
            default_value: None,
            options: None,
            depends_on: None,
            description: None,
            in_list_view: false,
            in_standard_filter: false,
            collapsible: false,
        },
        // Section 1: Customer Information
        DocFieldSchema {
            fieldname: "sec_customer".into(),
            label: "Customer Info".into(),
            fieldtype: FieldType::SectionBreak,
            reqd: false,
            read_only: false,
            hidden: false,
            default_value: None,
            options: None,
            depends_on: None,
            description: None,
            in_list_view: false,
            in_standard_filter: false,
            collapsible: false,
        },
        // Column 1: Customer Link
        DocFieldSchema {
            fieldname: "customer".into(),
            label: "Customer".into(),
            fieldtype: FieldType::Link {
                target_doctype: "Customer".into(),
            },
            reqd: true,
            read_only: false,
            hidden: false,
            default_value: None,
            options: None,
            depends_on: None,
            description: Some("Select the purchasing customer".into()),
            in_list_view: true,
            in_standard_filter: true,
            collapsible: false,
        },
        // Column 2: Date
        DocFieldSchema {
            fieldname: "col_break_1".into(),
            label: "".into(),
            fieldtype: FieldType::ColumnBreak,
            reqd: false,
            read_only: false,
            hidden: false,
            default_value: None,
            options: None,
            depends_on: None,
            description: None,
            in_list_view: false,
            in_standard_filter: false,
            collapsible: false,
        },
        DocFieldSchema {
            fieldname: "posting_date".into(),
            label: "Posting Date".into(),
            fieldtype: FieldType::Date,
            reqd: true,
            read_only: false,
            hidden: false,
            default_value: None,
            options: None,
            depends_on: None,
            description: None,
            in_list_view: true,
            in_standard_filter: false,
            collapsible: false,
        },
        // Section 2: Items Table
        DocFieldSchema {
            fieldname: "sec_items".into(),
            label: "Line Items".into(),
            fieldtype: FieldType::SectionBreak,
            reqd: false,
            read_only: false,
            hidden: false,
            default_value: None,
            options: None,
            depends_on: None,
            description: None,
            in_list_view: false,
            in_standard_filter: false,
            collapsible: false,
        },
        DocFieldSchema {
            fieldname: "items".into(),
            label: "Items".into(),
            fieldtype: FieldType::Table {
                child_doctype: "Sales Invoice Item".into(),
            },
            reqd: true,
            read_only: false,
            hidden: false,
            default_value: None,
            options: None,
            depends_on: None,
            description: None,
            in_list_view: false,
            in_standard_filter: false,
            collapsible: false,
        },
        // Tab 2: Accounting Dimensions
        DocFieldSchema {
            fieldname: "tab_accounting".into(),
            label: "Accounting".into(),
            fieldtype: FieldType::TabBreak,
            reqd: false,
            read_only: false,
            hidden: false,
            default_value: None,
            options: None,
            depends_on: None,
            description: None,
            in_list_view: false,
            in_standard_filter: false,
            collapsible: false,
        },
        DocFieldSchema {
            fieldname: "cost_center".into(),
            label: "Cost Center".into(),
            fieldtype: FieldType::Link {
                target_doctype: "Cost Center".into(),
            },
            reqd: false,
            read_only: false,
            hidden: false,
            default_value: None,
            options: None,
            depends_on: None,
            description: None,
            in_list_view: false,
            in_standard_filter: false,
            collapsible: false,
        },
    ];

    let schema = DocTypeSchema {
        name: "Sales Invoice".into(),
        module: "Accounts".into(),
        is_single: false,
        is_submittable: true,
        is_child_table: false,
        is_tree: false,
        track_changes: true,
        quick_entry: false,
        allow_rename: false,
        allow_import: true,
        allow_auto_repeat: true,
        fields,
        permissions: Vec::new(),
    };

    // Compile layout using FormLayoutEngine
    let mut layout = FormLayoutEngine::compile_layout(&schema);

    assert_eq!(layout.doctype_name.as_str(), "Sales Invoice");
    assert!(layout.is_submittable);
    assert!(layout.alt_inspect_enabled);

    // Verify 2 tabs compiled
    assert_eq!(layout.tabs.len(), 2);
    assert_eq!(layout.tabs[0].label.as_str(), "Invoice Details");
    assert_eq!(layout.tabs[1].label.as_str(), "Accounting");

    // Verify Tab 1 has 2 sections: "Customer Info" and "Line Items"
    let tab1 = &layout.tabs[0];
    assert_eq!(tab1.sections.len(), 2);
    assert_eq!(tab1.sections[0].title.as_deref(), Some("Customer Info"));
    assert_eq!(tab1.sections[1].title.as_deref(), Some("Line Items"));

    // Verify Section 1 has 2 columns: Column 0 (customer), Column 1 (posting_date)
    assert_eq!(tab1.sections[0].columns.len(), 2);

    let col0_widget = &tab1.sections[0].columns[0].widgets[0];
    if let FormLayoutWidget::Field {
        fieldname,
        reqd,
        inspector,
        ..
    } = col0_widget
    {
        assert_eq!(fieldname.as_str(), "customer");
        assert!(reqd);
        assert_eq!(
            inspector.description.as_deref(),
            Some("Select the purchasing customer")
        );
    } else {
        panic!("Expected field widget in col 0");
    }

    // Verify Section 2 has ChildTable widget
    let col_table_widget = &tab1.sections[1].columns[0].widgets[0];
    if let FormLayoutWidget::ChildTable(table) = col_table_widget {
        assert_eq!(table.fieldname.as_str(), "items");
        assert_eq!(table.child_doctype.as_str(), "Sales Invoice Item");
        assert!(table.allow_bulk_add);
    } else {
        panic!("Expected ChildTable widget");
    }

    // Verify Form Timeline event appending
    layout.timeline.push_event(FormTimelineEvent {
        id: "EVT-01".into(),
        event_type: "Audit".into(),
        user: "auditor@company.com".into(),
        timestamp_iso: "2026-10-16T12:00:00Z".into(),
        message: "Invoice created via POS terminal".into(),
    });

    assert_eq!(layout.timeline.events.len(), 1);
    assert_eq!(
        layout.timeline.events[0].message.as_str(),
        "Invoice created via POS terminal"
    );
}

#[test]
fn test_golden_declarative_script_report_engine() {
    let columns = vec![
        ScriptReportColumn::new("invoice", "Invoice ID", "Data", 150),
        ScriptReportColumn::new("customer", "Customer Name", "Data", 200),
        ScriptReportColumn::new("status", "Status", "Data", 100),
        ScriptReportColumn::new("net_amount", "Net Amount", "Currency", 120),
        ScriptReportColumn::new("tax_amount", "Tax Amount", "Currency", 100),
        ScriptReportColumn::new("grand_total", "Grand Total", "Currency", 140),
    ];

    let report = DeclarativeReport::new("General Sales Register", "Sales Invoice", columns);

    // Mock dataset with 4 invoices
    let mut row1 = HashMap::new();
    row1.insert("invoice".into(), json!("INV-001"));
    row1.insert("customer".into(), json!("Acme Corp"));
    row1.insert("status".into(), json!("Paid"));
    row1.insert("net_amount".into(), json!(1000.0));
    row1.insert("tax_amount".into(), json!(100.0));
    row1.insert("grand_total".into(), json!(1100.0));

    let mut row2 = HashMap::new();
    row2.insert("invoice".into(), json!("INV-002"));
    row2.insert("customer".into(), json!("Globex LLC"));
    row2.insert("status".into(), json!("Unpaid"));
    row2.insert("net_amount".into(), json!(2000.0));
    row2.insert("tax_amount".into(), json!(200.0));
    row2.insert("grand_total".into(), json!(2200.0));

    let mut row3 = HashMap::new();
    row3.insert("invoice".into(), json!("INV-003"));
    row3.insert("customer".into(), json!("Acme Corp"));
    row3.insert("status".into(), json!("Paid"));
    row3.insert("net_amount".into(), json!(3000.0));
    row3.insert("tax_amount".into(), json!(300.0));
    row3.insert("grand_total".into(), json!(3300.0));

    let mut row4 = HashMap::new();
    row4.insert("invoice".into(), json!("INV-004"));
    row4.insert("customer".into(), json!("Initech Inc"));
    row4.insert("status".into(), json!("Cancelled"));
    row4.insert("net_amount".into(), json!(500.0));
    row4.insert("tax_amount".into(), json!(50.0));
    row4.insert("grand_total".into(), json!(550.0));

    let all_rows = vec![row1, row2, row3, row4];

    // Filter by: status == "Paid"
    let filters = vec![ReportFilter {
        fieldname: "status".into(),
        label: "Invoice Status".into(),
        operator: FilterOperator::Equals,
        value: json!("Paid"),
        is_mandatory: false,
    }];

    let pagination = ReportPagination {
        page_size: 10,
        page_index: 0,
    };

    let (paged_rows, grand_totals, total_count) =
        report.execute_with_pagination(&all_rows, &filters, pagination);

    assert_eq!(total_count, 2); // INV-001 and INV-003
    assert_eq!(paged_rows.len(), 2);

    // Verify grand totals for filtered rows
    // net_amount = 1000 + 3000 = 4000
    assert_eq!(grand_totals.get("net_amount"), Some(&dec!(4000.0)));
    // tax_amount = 100 + 300 = 400
    assert_eq!(grand_totals.get("tax_amount"), Some(&dec!(400.0)));
    // grand_total = 1100 + 3300 = 4400
    assert_eq!(grand_totals.get("grand_total"), Some(&dec!(4400.0)));
}
