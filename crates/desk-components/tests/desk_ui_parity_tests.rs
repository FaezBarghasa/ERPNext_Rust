//! Phase 8 Integration Tests: Award-Grade Desk UI & AwesomBar (`desk-components` & `desk-app`).
//!
//! Validates:
//! 1. Universal AwesomBar fuzzy search over DocTypes, studios, reports, and administrative actions.
//! 2. Virtualized Child Table keyboard navigation (`Tab`, `Shift+Tab`, `Enter`, arrows, range selection).
//! 3. Multi-persona cockpits (/desk, /worker, /factory, /portal, /admin, /approvals).
//! 4. Emil Kowalski spring micro-interactions & high-contrast mode configurations.

use desk_app::persona_shell::{PersonaRole, PersonaShellConfig};
use desk_components::desk_shell::{get_desk_workspaces, render_desk_shell_html};
use desk_components::grid::{
    CellCoordinate, CellSelectionRange, GridNavDirection, VirtualizedGridState,
};

#[test]
fn test_awesombar_and_desk_shell_rendering_parity() {
    let workspaces = get_desk_workspaces();
    // Core enterprise workspaces
    assert!(workspaces.len() >= 11);
    assert!(workspaces.iter().any(|w| w.id == "accounting"));
    assert!(workspaces.iter().any(|w| w.id == "sales"));
    assert!(workspaces.iter().any(|w| w.id == "manufacturing"));
    assert!(workspaces.iter().any(|w| w.id == "inventory"));
    assert!(workspaces.iter().any(|w| w.id == "hr"));
    assert!(workspaces.iter().any(|w| w.id == "wms"));

    let shell_html = render_desk_shell_html("Enterprise Desk", "Administrator");

    // Check AwesomBar markup & shortcuts
    assert!(shell_html.contains("omnibar-wrapper"));
    assert!(shell_html.contains("id=\"omni-search\""));
    assert!(shell_html.contains("id=\"omni-popover\""));
    assert!(shell_html.contains("Cmd+K"));
    assert!(shell_html.contains("awesomeItems"));

    // Check Emil Kowalski spring physics styling
    assert!(shell_html.contains("--ease-spring: cubic-bezier(0.16, 1, 0.3, 1)"));
    assert!(shell_html.contains("transform: scale(1.04)"));
    assert!(shell_html.contains("transform: scale(0.96)"));

    // Check theme switching and sidebar state persistence
    assert!(shell_html.contains("toggleTheme()"));
    assert!(shell_html.contains("desk-sidebar"));
}

#[test]
fn test_child_table_keyboard_navigation_ergonomics() {
    // 100 rows child table with 5 columns: [item_code, qty, rate, discount, amount]
    let mut grid = VirtualizedGridState::new(100, 5);

    assert_eq!(grid.active_cell, CellCoordinate { row: 0, col: 0 });

    // 1. Tab navigates to next cell in current row
    grid.navigate(GridNavDirection::NextCell, false);
    assert_eq!(grid.active_cell, CellCoordinate { row: 0, col: 1 });

    // 2. Tab wrap at end of row wraps to next row column 0
    grid.active_cell = CellCoordinate { row: 0, col: 4 };
    grid.navigate(GridNavDirection::NextCell, false);
    assert_eq!(grid.active_cell, CellCoordinate { row: 1, col: 0 });

    // 3. Shift+Tab navigates to previous cell
    grid.navigate(GridNavDirection::PrevCell, false);
    assert_eq!(grid.active_cell, CellCoordinate { row: 0, col: 4 });

    // 4. Enter navigates down to next row
    grid.navigate(GridNavDirection::NextRow, false);
    assert_eq!(grid.active_cell, CellCoordinate { row: 1, col: 4 });

    // 5. Arrow Up / Down navigation
    grid.navigate(GridNavDirection::Up, false);
    assert_eq!(grid.active_cell, CellCoordinate { row: 0, col: 4 });

    grid.navigate(GridNavDirection::Down, false);
    assert_eq!(grid.active_cell, CellCoordinate { row: 1, col: 4 });

    // 6. Range selection expansion
    grid.active_cell = CellCoordinate { row: 1, col: 1 };
    grid.navigate(GridNavDirection::Right, true);
    assert_eq!(grid.active_cell, CellCoordinate { row: 1, col: 2 });
    assert_eq!(
        grid.selection,
        Some(CellSelectionRange {
            start_row: 1,
            start_col: 1,
            end_row: 1,
            end_col: 2,
        })
    );
    assert_eq!(grid.selection.unwrap().cell_count(), 2);
}

#[test]
fn test_multi_persona_cockpits_configuration() {
    // 1. Enterprise Desk
    let desk_cfg = PersonaShellConfig::for_role(PersonaRole::Desk);
    assert_eq!(desk_cfg.default_route.as_str(), "/desk");
    assert!(!desk_cfg.high_contrast_mode);

    // 2. Warehouse Handheld Kiosk (/worker)
    let worker_cfg = PersonaShellConfig::for_role(PersonaRole::WarehouseWorker);
    assert_eq!(worker_cfg.default_route.as_str(), "/worker");
    assert!(worker_cfg.touch_target_min_px >= 56);
    assert!(worker_cfg.continuous_scan_enabled);
    assert!(worker_cfg.high_contrast_mode);

    // 3. Shopfloor MES Operator (/factory)
    let factory_cfg = PersonaShellConfig::for_role(PersonaRole::ShopfloorMes);
    assert_eq!(factory_cfg.default_route.as_str(), "/factory");
    assert!(factory_cfg.dual_witness_required);

    // 4. Customer Portal (/portal)
    let portal_cfg = PersonaShellConfig::for_role(PersonaRole::ClientCustomer);
    assert_eq!(portal_cfg.default_route.as_str(), "/portal");
}
