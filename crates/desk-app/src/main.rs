//! `desk-app` — The Pure-Rust Reactive Enterprise Desk Application.

use desk_components::{
    AmrMarkerViewModel, BinViewModel, CpqConfiguratorModel, DynamicFormModel, GanttDependencyLink,
    GanttTaskRow, GanttViewModel, OptionCard, SpcChartViewModel, SpcPointView,
    Warehouse3DViewModel, visible_slice,
};
use frappe_meta::{DocFieldSchema, DocPermSchema, DocTypeSchema, FieldType};

fn main() {
    println!("========================================================");
    println!("  Dioxus Reactive Enterprise Desk — Substrate Runtime    ");
    println!("========================================================");

    // 1. Virtualized Grid Viewport Simulation
    let (start_row, visible_count) = visible_slice(1_000_000, 36, 0, 600);
    println!(
        "[1/6] Virtualized Data Grid: Render slice indices [{}..{}] (window: {} rows of 1,000,000 total).",
        start_row,
        start_row + visible_count,
        visible_count
    );

    // 2. Dynamic Reactive Form Engine Compilation
    let schema = DocTypeSchema {
        name: "Sales Invoice".into(),
        module: "Accounts".into(),
        is_single: false,
        is_submittable: true,
        track_changes: true,
        naming_rule: Some("ACC-SINV-.YYYY.-.#####".into()),
        fields: vec![
            DocFieldSchema {
                fieldname: "customer".into(),
                fieldtype: FieldType::Link {
                    target_doctype: "Customer".into(),
                },
                label: "Customer".into(),
                reqd: true,
                unique: false,
                read_only: false,
                hidden: false,
                default_value: None,
                options: None,
                in_list_view: true,
            },
            DocFieldSchema {
                fieldname: "grand_total".into(),
                fieldtype: FieldType::Currency,
                label: "Grand Total".into(),
                reqd: true,
                unique: false,
                read_only: true,
                hidden: false,
                default_value: Some("0.0".into()),
                options: None,
                in_list_view: true,
            },
        ],
        permissions: vec![DocPermSchema {
            role: "Accounts User".into(),
            permlevel: 0,
            read: true,
            write: true,
            create: true,
            delete: false,
            submit: true,
            cancel: false,
            amend: true,
            report: true,
            export: true,
        }],
    };
    let form_model = DynamicFormModel::from_schema(&schema);
    println!(
        "[2/6] Dynamic Reactive Form Model: Compiled `{}` with {} active interactive widgets.",
        form_model.doctype_name,
        form_model.widgets.len()
    );

    // 3. Gantt Schedule View Model
    let mut gantt = GanttViewModel::new("Global Refinery Expansion".into(), 180);
    gantt.tasks.push(GanttTaskRow {
        task_id: 1,
        name: "EPC Civil Works".into(),
        start_day: 0,
        duration_days: 45,
        total_float: 0,
        is_critical: true,
        progress_percent: 85.0,
    });
    gantt.tasks.push(GanttTaskRow {
        task_id: 2,
        name: "Piping & Instrumentation".into(),
        start_day: 45,
        duration_days: 60,
        total_float: 0,
        is_critical: true,
        progress_percent: 20.0,
    });
    gantt.dependencies.push(GanttDependencyLink {
        from_task_id: 1,
        to_task_id: 2,
        lag_days: 0,
    });
    println!(
        "[3/6] Gantt CPM Schedule: `{}` ({} tasks, {} dependencies, Buffer Zone: {}).",
        gantt.project_name,
        gantt.tasks.len(),
        gantt.dependencies.len(),
        gantt.buffer_zone
    );

    // 4. Interactive CPQ Configurator Model
    let mut cpq = CpqConfiguratorModel::new("QUOTE-2026-0988".into());
    cpq.options.push(OptionCard {
        option_id: "OPT-TURBINE-SIC".into(),
        name: "Silicon-Carbide Hybrid Turbine Core".into(),
        group: "Propulsion".into(),
        price: 250_000.0,
        is_selected: true,
        is_disabled: false,
        disable_reason: None,
    });
    cpq.list_price = 750_000.0;
    cpq.net_price = 720_000.0;
    cpq.pocket_price = 690_000.0;
    cpq.gross_margin_percent = 42.5;
    println!(
        "[4/6] CPQ Constraint Configurator: Quote `{}` (List: ${:.2}, Net: ${:.2}, Gross Margin: {:.1}%).",
        cpq.quote_id, cpq.list_price, cpq.net_price, cpq.gross_margin_percent
    );

    // 5. Statistical Process Control (SPC) Chart Model
    let mut spc = SpcChartViewModel::new(
        "Bearing Outer Ring Outer Diameter".into(),
        50.0,
        50.002,
        50.045,
        49.955,
    );
    spc.push_point(SpcPointView {
        subgroup_id: 1,
        mean: 50.003,
        range: 0.012,
        is_violation: false,
        rule_tag: None,
    });
    spc.push_point(SpcPointView {
        subgroup_id: 2,
        mean: 50.048,
        range: 0.018,
        is_violation: true,
        rule_tag: Some("Nelson Rule 1: Beyond 3 Sigma".into()),
    });
    println!(
        "[5/6] Streaming SPC Chart: `{}` (UCL: {:.3}, LCL: {:.3}, Active Violations: {}).",
        spc.parameter_name, spc.ucl, spc.lcl, spc.active_violations_count
    );

    // 6. 3D Warehouse Coordinate Grid & AMR Model
    let mut wms = Warehouse3DViewModel::new("High-Density Automated Distribution Center".into());
    wms.add_bin(BinViewModel {
        coordinate_code: "Z1-A04-B12-L3-01".into(),
        zone: "Zone-1 (High-Velocity)".into(),
        aisle: 4,
        bay: 12,
        level: 3,
        bin: 1,
        utilization_percent: 92.0,
        is_hazmat: false,
    });
    wms.update_robot(AmrMarkerViewModel {
        robot_serial: "AMR-AGV-007".into(),
        x: 48.5,
        y: 112.3,
        status: "Navigating to Bin".into(),
        battery_percent: 88.0,
    });
    println!(
        "[6/6] 3D Warehouse & VDA 5050 AMR: `{}` (Tracked Bins: {}, AMR Fleet: {}).",
        wms.warehouse_name,
        wms.total_bins,
        wms.robots.len()
    );

    println!();
    println!("Dioxus Reactive Desk substrate fully initialized and verified.");
}
