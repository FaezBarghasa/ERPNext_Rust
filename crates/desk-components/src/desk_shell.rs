//! Unified Enterprise Desk & Multi-Persona Shell SSR Engine (`desk_components::desk_shell`).
//!
//! Renders the Awwwards-grade, zero-external-dependency administrative desktop shell,
//! module workspaces, AwesomeBar autocomplete, quick entry drawers, child table grids,
//! bulk action bars, and real-time live telemetry HUD matching Frappe / ERPNext 1:1.

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// Workspace category specification for the Desk home navigation.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceModule {
    pub id: CompactString,
    pub title: CompactString,
    pub icon_svg: CompactString,
    pub badge: CompactString,
    pub description: CompactString,
    pub doctypes: Vec<CompactString>,
    pub route: CompactString,
}

/// Returns the 11 standard core ERPNext workspaces.
#[must_use]
pub fn get_desk_workspaces() -> Vec<WorkspaceModule> {
    vec![
        WorkspaceModule {
            id: "accounting".into(),
            title: "Accounting & Finance".into(),
            icon_svg: "M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z".into(),
            badge: "ACID Double-Entry".into(),
            description: "General Ledger, Invoicing, Multi-Currency, Fiscal Years & Statutory Tax".into(),
            doctypes: vec!["Sales Invoice".into(), "Purchase Invoice".into(), "Journal Entry".into(), "Account".into()],
            route: "/desk/list/sales_invoice".into(),
        },
        WorkspaceModule {
            id: "sales".into(),
            title: "Sales & Commerce".into(),
            icon_svg: "M16 11V7a4 4 0 00-8 0v4M5 9h14l1 12H4L5 9z".into(),
            badge: "Omnichannel".into(),
            description: "Quotations, Sales Orders, Customer Portals, CPQ & Storefront Fulfillment".into(),
            doctypes: vec!["Quotation".into(), "Sales Order".into(), "Customer".into(), "Pricing Rule".into()],
            route: "/desk/list/sales_order".into(),
        },
        WorkspaceModule {
            id: "purchasing".into(),
            title: "Buying & Procurement".into(),
            icon_svg: "M3 3h2l.4 2M7 13h10l4-8H5.4M7 13L5.4 5M7 13l-2.293 2.293c-.63.63-.184 1.707.707 1.707H17m0 0a2 2 0 100 4 2 2 0 000-4zm-8 2a2 2 0 11-4 0 2 2 0 014 0z".into(),
            badge: "3-Way Matching".into(),
            description: "Material Requests, Supplier RFQs, Purchase Orders & Landed Cost Vouchers".into(),
            doctypes: vec!["Purchase Order".into(), "Supplier".into(), "Material Request".into()],
            route: "/desk/list/purchase_order".into(),
        },
        WorkspaceModule {
            id: "inventory".into(),
            title: "Stock & Inventory".into(),
            icon_svg: "M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4".into(),
            badge: "FIFO Valuation".into(),
            description: "Multi-Warehouse Bins, Serial/Batch Tracking, Reorder Levels & Stock Ledger".into(),
            doctypes: vec!["Item".into(), "Stock Entry".into(), "Warehouse".into(), "Batch".into()],
            route: "/desk/list/item".into(),
        },
        WorkspaceModule {
            id: "manufacturing".into(),
            title: "Manufacturing & MES".into(),
            icon_svg: "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z".into(),
            badge: "Multi-Level BOM".into(),
            description: "Work Orders, Routing Operations, Job Cards, Digital Travelers & Live SPC".into(),
            doctypes: vec!["BOM".into(), "Work Order".into(), "Job Card".into(), "Operation".into()],
            route: "/desk/list/work_order".into(),
        },
        WorkspaceModule {
            id: "wms".into(),
            title: "WMS & AMR Logistics".into(),
            icon_svg: "M9 17a2 2 0 11-4 0 2 2 0 014 0zM19 17a2 2 0 11-4 0 2 2 0 014 0z".into(),
            badge: "VDA 5050 Robot Fleet".into(),
            description: "3D Bin Visualizer, Directed Pick-Paths, AMR Mission Dispatcher & Cross-Docking".into(),
            doctypes: vec!["Putaway Rule".into(), "Pick List".into(), "AMR Vehicle".into()],
            route: "/worker".into(),
        },
        WorkspaceModule {
            id: "crm".into(),
            title: "CRM & Pipelines".into(),
            icon_svg: "M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z".into(),
            badge: "Kanban Pipeline".into(),
            description: "Lead Enrichment, Deal Stages, Opportunity Forecasting & Activity Streams".into(),
            doctypes: vec!["Lead".into(), "Opportunity".into(), "Campaign".into()],
            route: "/desk/list/lead".into(),
        },
        WorkspaceModule {
            id: "hr".into(),
            title: "HR & Payroll".into(),
            icon_svg: "M21 13.255A23.931 23.931 0 0112 15c-3.183 0-6.22-.62-9-1.745M16 6V4a2 2 0 00-2-2h-4a2 2 0 00-2 2v2m4 6h.01M5 20h14a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z".into(),
            badge: "Salary Slips".into(),
            description: "Employee Records, Attendance, Leave Allocations, Tax Deductions & Payroll Slips".into(),
            doctypes: vec!["Employee".into(), "Salary Slip".into(), "Leave Application".into()],
            route: "/desk/list/employee".into(),
        },
        WorkspaceModule {
            id: "assets".into(),
            title: "Assets & Maintenance".into(),
            icon_svg: "M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4".into(),
            badge: "Depreciation Schedules".into(),
            description: "Fixed Asset Registry, Straight-Line Depreciation, Repair Work Orders & Telemetry".into(),
            doctypes: vec!["Asset".into(), "Asset Repair".into(), "Maintenance Schedule".into()],
            route: "/desk/list/asset".into(),
        },
        WorkspaceModule {
            id: "learning".into(),
            title: "LMS & Education".into(),
            icon_svg: "M12 14l9-5-9-5-9 5 9 5zm0 0l6.16-3.422a12.083 12.083 0 01.665 6.479A11.952 11.952 0 0012 20.055a11.952 11.952 0 00-6.824-2.998 12.078 12.078 0 01.665-6.479L12 14zm-4 6v-7.5l4-2.222".into(),
            badge: "SVOD Streaming".into(),
            description: "Course Catalogs, Student Enrollment, Interactive Quizzes & Typst Diploma Engine".into(),
            doctypes: vec!["Course".into(), "Lesson".into(), "Student".into()],
            route: "/portal".into(),
        },
        WorkspaceModule {
            id: "cockpit".into(),
            title: "System Admin Cockpit".into(),
            icon_svg: "M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z".into(),
            badge: "Cluster Operations".into(),
            description: "Multi-Tenant Database Pools, Wasmtime Fuel Monitor, Rate Limiters & Tracing Telemetry".into(),
            doctypes: vec!["Tenant".into(), "DocType".into(), "User".into(), "Role".into()],
            route: "/admin".into(),
        },
    ]
}

const DESK_BASE_HTML: &str = r#"<!DOCTYPE html>
<html lang="en" data-theme="dark">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RustNext Enterprise Desk — __PERSONA__</title>
    <style>
        :root {
            --bg-primary: #0a0b10;
            --bg-secondary: #12141c;
            --bg-card: rgba(22, 25, 37, 0.75);
            --border: rgba(255, 255, 255, 0.08);
            --accent: #6366f1;
            --accent-glow: rgba(99, 102, 241, 0.25);
            --text-primary: #f8fafc;
            --text-secondary: #94a3b8;
            --success: #10b981;
            --warning: #f59e0b;
            --danger: #ef4444;
            --sidebar-width: 250px;
            --sidebar-collapsed-width: 68px;
            --font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
        }
        [data-theme="light"] {
            --bg-primary: #f8fafc;
            --bg-secondary: #ffffff;
            --bg-card: #ffffff;
            --border: rgba(0, 0, 0, 0.08);
            --accent: #4f46e5;
            --accent-glow: rgba(79, 70, 229, 0.15);
            --text-primary: #0f172a;
            --text-secondary: #64748b;
        }
        * { margin: 0; padding: 0; box-sizing: border-box; font-family: var(--font-family); }
        ::-webkit-scrollbar { width: 6px; height: 6px; }
        ::-webkit-scrollbar-track { background: transparent; }
        ::-webkit-scrollbar-thumb { background: rgba(255, 255, 255, 0.15); border-radius: 9999px; }
        ::-webkit-scrollbar-thumb:hover { background: rgba(255, 255, 255, 0.25); }
        button:active, .persona-select:active { transform: scale(0.98); }
        body { background: var(--bg-primary); color: var(--text-primary); min-height: 100vh; display: flex; flex-direction: column; overflow-x: hidden; }
        
        header {
            position: sticky; top: 0; z-index: 50;
            background: rgba(10, 11, 16, 0.88); backdrop-filter: blur(16px);
            border-bottom: 1px solid var(--border);
            padding: 0.75rem 1.5rem; display: flex; justify-content: space-between; align-items: center; gap: 1rem;
        }
        .header-left { display: flex; align-items: center; gap: 1rem; }
        .sidebar-toggle-btn {
            background: var(--bg-secondary); border: 1px solid var(--border); color: var(--text-secondary);
            width: 32px; height: 32px; border-radius: 8px; display: flex; align-items: center; justify-content: center;
            cursor: pointer; transition: all 0.2s;
        }
        .sidebar-toggle-btn:hover { color: var(--text-primary); border-color: var(--accent); }
        .brand { display: flex; align-items: center; gap: 0.75rem; font-weight: 700; font-size: 1.15rem; letter-spacing: -0.02em; text-decoration: none; color: inherit; }
        .brand-logo { width: 32px; height: 32px; background: linear-gradient(135deg, #6366f1, #a855f7); border-radius: 8px; display: flex; align-items: center; justify-content: center; font-weight: 900; color: white; }
        
        .omnibar-wrapper { position: relative; width: 440px; }
        .omnibar {
            display: flex; align-items: center; gap: 0.5rem;
            background: var(--bg-secondary); border: 1px solid var(--border); border-radius: 10px;
            padding: 0.5rem 1rem; width: 100%; transition: all 0.2s;
        }
        .omnibar:focus-within { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-glow); }
        .omnibar input { background: transparent; border: none; color: var(--text-primary); width: 100%; outline: none; font-size: 0.875rem; }
        .omnibar kbd { background: rgba(255, 255, 255, 0.1); border-radius: 4px; padding: 0.15rem 0.4rem; font-size: 0.75rem; color: var(--text-secondary); }
        
        .omni-popover {
            position: absolute; top: calc(100% + 6px); left: 0; right: 0;
            background: var(--bg-secondary); border: 1px solid var(--border); border-radius: 12px;
            box-shadow: 0 16px 36px rgba(0, 0, 0, 0.45); backdrop-filter: blur(12px);
            z-index: 100; max-height: 380px; overflow-y: auto; display: none; padding: 0.5rem;
        }
        .omni-group-title { font-size: 0.7rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.05em; color: var(--text-secondary); padding: 0.4rem 0.6rem; }
        .omni-item {
            display: flex; align-items: center; justify-content: space-between; padding: 0.5rem 0.75rem;
            border-radius: 8px; font-size: 0.85rem; color: var(--text-primary); cursor: pointer; text-decoration: none;
            transition: background 0.15s;
        }
        .omni-item:hover, .omni-item.active { background: var(--accent-glow); color: var(--accent); }
        .omni-item-badge { font-size: 0.7rem; padding: 0.1rem 0.4rem; border-radius: 4px; background: rgba(255,255,255,0.08); color: var(--text-secondary); }
        
        .nav-actions { display: flex; align-items: center; gap: 0.75rem; position: relative; }
        .persona-select {
            background: var(--bg-secondary); border: 1px solid var(--border); color: var(--text-primary);
            padding: 0.4rem 0.8rem; border-radius: 8px; font-size: 0.85rem; outline: none; cursor: pointer;
        }
        .user-chip { display: flex; align-items: center; gap: 0.5rem; font-size: 0.875rem; font-weight: 500; color: var(--text-secondary); cursor: pointer; padding: 0.3rem 0.6rem; border-radius: 8px; transition: background 0.2s; }
        .user-chip:hover { background: rgba(255, 255, 255, 0.05); }
        .user-avatar { width: 32px; height: 32px; border-radius: 50%; background: #334155; display: flex; align-items: center; justify-content: center; color: white; font-weight: 600; font-size: 0.875rem; }
        
        .user-menu-popover {
            position: absolute; top: calc(100% + 8px); right: 0; width: 220px;
            background: var(--bg-secondary); border: 1px solid var(--border); border-radius: 12px;
            box-shadow: 0 12px 32px rgba(0, 0, 0, 0.45); padding: 0.5rem; display: none; z-index: 100;
        }
        .user-menu-item { display: flex; align-items: center; gap: 0.6rem; padding: 0.5rem 0.75rem; font-size: 0.85rem; border-radius: 6px; color: var(--text-primary); cursor: pointer; text-decoration: none; }
        .user-menu-item:hover { background: rgba(255, 255, 255, 0.06); color: var(--accent); }
        .user-menu-divider { height: 1px; background: var(--border); margin: 0.3rem 0; }
        
        .layout-container { display: flex; flex: 1; min-height: calc(100vh - 58px); }
        
        aside.desk-sidebar {
            width: var(--sidebar-width); background: var(--bg-secondary); border-right: 1px solid var(--border);
            display: flex; flex-direction: column; transition: width 0.25s cubic-bezier(0.16, 1, 0.3, 1);
            overflow-y: auto; overflow-x: hidden; flex-shrink: 0;
        }
        aside.desk-sidebar.collapsed { width: var(--sidebar-collapsed-width); }
        .sidebar-nav { padding: 1rem 0.5rem; display: flex; flex-direction: column; gap: 0.25rem; }
        .nav-item {
            display: flex; align-items: center; gap: 0.75rem; padding: 0.6rem 0.75rem; border-radius: 8px;
            color: var(--text-secondary); text-decoration: none; font-size: 0.875rem; font-weight: 500;
            cursor: pointer; transition: all 0.15s; white-space: nowrap;
        }
        .nav-item:hover, .nav-item.active { background: rgba(99, 102, 241, 0.12); color: var(--accent); }
        .nav-item svg { width: 18px; height: 18px; flex-shrink: 0; }
        .nav-text { overflow: hidden; text-overflow: ellipsis; }
        aside.desk-sidebar.collapsed .nav-text { display: none; }
        aside.desk-sidebar.collapsed .sidebar-header-title { display: none; }
        .sidebar-header-title { font-size: 0.7rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.05em; color: var(--text-secondary); padding: 0.5rem 0.75rem 0.25rem; }
        
        main { flex: 1; padding: 2rem; max-width: 1400px; margin: 0 auto; width: 100%; }
        
        .shortcuts-bar {
            display: flex; align-items: center; gap: 0.6rem; overflow-x: auto; padding-bottom: 1rem; margin-bottom: 1.5rem;
        }
        .shortcut-chip {
            display: inline-flex; align-items: center; gap: 0.5rem; background: var(--bg-card);
            border: 1px solid var(--border); padding: 0.4rem 0.85rem; border-radius: 9999px;
            font-size: 0.8rem; font-weight: 600; color: var(--text-primary); text-decoration: none;
            cursor: pointer; transition: all 0.2s; white-space: nowrap;
        }
        .shortcut-chip:hover { border-color: var(--accent); transform: translateY(-1px); }
        .shortcut-badge { font-size: 0.7rem; padding: 0.1rem 0.4rem; border-radius: 9999px; background: rgba(99, 102, 241, 0.2); color: #818cf8; }
        
        .welcome-hero { margin-bottom: 2rem; }
        .welcome-hero h1 { font-size: 2rem; font-weight: 800; letter-spacing: -0.03em; margin-bottom: 0.35rem; background: linear-gradient(135deg, var(--text-primary), var(--text-secondary)); -webkit-background-clip: text; -webkit-text-fill-color: transparent; }
        .welcome-hero p { color: var(--text-secondary); font-size: 0.95rem; }
        
        .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 1.25rem; }
        .card {
            background: var(--bg-card); border: 1px solid var(--border); border-radius: 14px;
            padding: 1.25rem; cursor: pointer; transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
            display: flex; flex-direction: column; position: relative; overflow: hidden;
        }
        .card:hover {
            transform: translateY(-4px); border-color: rgba(99, 102, 241, 0.4);
            box-shadow: 0 12px 24px -10px var(--accent-glow);
        }
        .card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem; }
        .card-icon { width: 40px; height: 40px; border-radius: 10px; background: rgba(99, 102, 241, 0.12); color: #818cf8; display: flex; align-items: center; justify-content: center; }
        .badge { font-size: 0.7rem; font-weight: 600; padding: 0.2rem 0.5rem; border-radius: 9999px; background: rgba(255, 255, 255, 0.06); color: #c7d2fe; border: 1px solid rgba(255, 255, 255, 0.08); }
        .card-title { font-size: 1.1rem; font-weight: 700; margin-bottom: 0.4rem; letter-spacing: -0.01em; }
        .card-desc { font-size: 0.85rem; color: var(--text-secondary); line-height: 1.45; margin-bottom: 1rem; flex: 1; }
        .pills-container { display: flex; flex-wrap: wrap; gap: 0.35rem; }
        .pill { font-size: 0.75rem; padding: 0.2rem 0.5rem; border-radius: 6px; background: rgba(255, 255, 255, 0.04); color: #cbd5e1; border: 1px solid rgba(255, 255, 255, 0.04); text-decoration: none; transition: all 0.15s; }
        .pill:hover { background: rgba(99, 102, 241, 0.25); color: #fff; }
        
        footer {
            background: var(--bg-secondary); border-top: 1px solid var(--border);
            padding: 0.6rem 2rem; display: flex; justify-content: space-between; align-items: center; font-size: 0.8rem; color: var(--text-secondary);
        }
        .status-badge { display: flex; align-items: center; gap: 0.4rem; }
        .status-dot { width: 8px; height: 8px; border-radius: 50%; background: var(--success); box-shadow: 0 0 8px var(--success); }
    </style>
</head>
<body>
    <header>
        <div class="header-left">
            <button class="sidebar-toggle-btn" id="sidebar-toggle" title="Toggle Sidebar (Ctrl+B)">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 12h18M3 6h18M3 18h18"/></svg>
            </button>
            <a href="/desk" class="brand">
                <div class="brand-logo">R</div>
                <span>RustNext Desk</span>
            </a>
        </div>

        <div class="omnibar-wrapper">
            <div class="omnibar">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="color:var(--text-secondary)"><circle cx="11" cy="11" r="8"/><path d="M21 21l-4.35-4.35"/></svg>
                <input type="text" placeholder="Search DocTypes, records, methods, or shortcuts..." id="omni-search" autocomplete="off" />
                <kbd>Ctrl+K</kbd>
            </div>
            <div class="omni-popover" id="omni-popover"></div>
        </div>

        <div class="nav-actions">
            <select class="persona-select" onchange="window.location.href=this.value">
                <option value="/desk" __DESK_SEL__>Unified Enterprise Desk</option>
                <option value="/portal" __PORTAL_SEL__>Customer Portal</option>
                <option value="/worker" __WORKER_SEL__>Warehouse Scanner</option>
                <option value="/factory" __FACTORY_SEL__>Shopfloor MES</option>
                <option value="/approvals" __APPROVALS_SEL__>Approval Deck</option>
                <option value="/admin" __ADMIN_SEL__>Admin Cockpit</option>
            </select>
            <div class="user-chip" id="user-chip">
                <div class="user-avatar">__AVATAR__</div>
                <span>__USERNAME__</span>
            </div>
            <div class="user-menu-popover" id="user-menu-popover">
                <div style="padding:0.4rem 0.6rem; font-weight:700; font-size:0.85rem; color:var(--text-primary);">__USERNAME__</div>
                <div style="padding:0 0.6rem 0.4rem; font-size:0.75rem; color:var(--text-secondary);">System Administrator</div>
                <div class="user-menu-divider"></div>
                <div class="user-menu-item" onclick="toggleTheme()"><span id="theme-text">☀️ Switch to Light Mode</span></div>
                <div class="user-menu-item" onclick="showToast('Session Defaults active for default site', 'info')">⚙️ Session Defaults</div>
                <div class="user-menu-item" onclick="alert('Shortcuts:\nCtrl+K: AwesomeBar Global Search\nCtrl+S: Save Document\nCtrl+B: Toggle Sidebar\nCtrl+P: Print')">⌨️ Keyboard Shortcuts</div>
                <div class="user-menu-divider"></div>
                <a href="/desk" class="user-menu-item">🔄 Reload Cache</a>
                <a href="/healthz/ready" class="user-menu-item">🩺 Health & DB Status</a>
            </div>
        </div>
    </header>

    <div class="layout-container">
        <aside class="desk-sidebar" id="desk-sidebar">
            <div class="sidebar-nav">
                <div class="sidebar-header-title">Workspaces</div>
                __SIDEBAR_LINKS__
            </div>
        </aside>

        <main>
            <div class="shortcuts-bar">
                <span style="font-size:0.75rem; font-weight:700; color:var(--text-secondary); text-transform:uppercase; margin-right:0.25rem;">Shortcuts:</span>
                <a href="/desk/list/sales_invoice" class="shortcut-chip">Sales Invoices <span class="shortcut-badge">Draft / Unpaid</span></a>
                <a href="/desk/list/sales_order" class="shortcut-chip">Sales Orders <span class="shortcut-badge">Active</span></a>
                <a href="/desk/list/purchase_order" class="shortcut-chip">Purchase Orders <span class="shortcut-badge">Procurement</span></a>
                <a href="/desk/list/item" class="shortcut-chip">Stock Ledger <span class="shortcut-badge">Inventory</span></a>
                <a href="/desk/list/journal_entry" class="shortcut-chip">General Ledger <span class="shortcut-badge">Chart of Accounts</span></a>
                <a href="/desk/form/sales_invoice/new" class="shortcut-chip" style="border-color:var(--accent); color:var(--accent);">+ New Invoice</a>
            </div>

            <div class="welcome-hero">
                <h1>Welcome to __PERSONA__</h1>
                <p>High-performance local-first Rust ERP runtime with ACID transaction ledger and sub-millisecond hypermedia response.</p>
            </div>
            <div class="grid" id="cards-grid">
                __CARDS__
            </div>
        </main>
    </div>

    <footer>
        <div class="status-badge">
            <div class="status-dot"></div>
            <span>SurrealDB 3.3.0 KV Local Engine • Tokio Multi-Thread • Wasmtime Sandboxed</span>
        </div>
        <div>RustNext v0.2.0 (2024 Edition) • Zero-IPC In-Memory Cache</div>
    </footer>

    <div id="toast-container" style="position:fixed;bottom:24px;right:24px;display:flex;flex-direction:column;gap:8px;z-index:9999;"></div>

    <script>
        function showToast(msg, type = 'info') {
            const container = document.getElementById('toast-container');
            const toast = document.createElement('div');
            toast.style.cssText = 'background:rgba(18,24,38,0.95);border:1px solid rgba(255,255,255,0.15);color:#fff;padding:12px 18px;border-radius:8px;font-size:13px;backdrop-filter:blur(8px);box-shadow:0 8px 32px rgba(0,0,0,0.5);display:flex;align-items:center;gap:10px;animation:fadeIn 0.2s ease;';
            toast.innerHTML = `<span style="color:${type==='success'?'#10b981':'#60a5fa'};">●</span> <span>${msg}</span>`;
            container.appendChild(toast);
            setTimeout(() => { toast.style.opacity = '0'; setTimeout(() => toast.remove(), 300); }, 4000);
        }

        // Live WebSocket Telemetry Stream
        (function connectLive() {
            try {
                const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
                const ws = new WebSocket(`${protocol}//${window.location.host}/api/v1/live`);
                ws.onmessage = (event) => {
                    try {
                        const data = JSON.parse(event.data);
                        if (data.topic || data.doctype) {
                            showToast(`Live update: ${data.doctype || data.topic} ${data.action || 'updated'}`, 'info');
                        }
                    } catch (_) {}
                };
                ws.onclose = () => setTimeout(connectLive, 5000);
            } catch (_) {}
        })();

        // Sidebar Toggle & LocalStorage Memory
        const sidebar = document.getElementById('desk-sidebar');
        const sidebarToggle = document.getElementById('sidebar-toggle');
        if (localStorage.getItem('sidebar_collapsed') === 'true') {
            sidebar.classList.add('collapsed');
        }
        sidebarToggle.addEventListener('click', () => {
            sidebar.classList.toggle('collapsed');
            localStorage.setItem('sidebar_collapsed', sidebar.classList.contains('collapsed'));
        });

        // Theme Toggle (Dark / Light)
        function toggleTheme() {
            const html = document.documentElement;
            const current = html.getAttribute('data-theme') || 'dark';
            const next = current === 'dark' ? 'light' : 'dark';
            html.setAttribute('data-theme', next);
            localStorage.setItem('desk_theme', next);
            const themeText = document.getElementById('theme-text');
            if (themeText) themeText.textContent = next === 'dark' ? '☀️ Switch to Light Mode' : '🌙 Switch to Dark Mode';
        }
        if (localStorage.getItem('desk_theme')) {
            document.documentElement.setAttribute('data-theme', localStorage.getItem('desk_theme'));
        }

        // User Menu Popover
        const userChip = document.getElementById('user-chip');
        const userMenu = document.getElementById('user-menu-popover');
        userChip.addEventListener('click', (e) => {
            e.stopPropagation();
            userMenu.style.display = userMenu.style.display === 'block' ? 'none' : 'block';
        });
        document.addEventListener('click', () => {
            if (userMenu) userMenu.style.display = 'none';
        });

        // Standard AwesomeBar (Frappe 1:1 Omnibar Search Navigation)
        const awesomeItems = [
            { label: 'Open Sales Invoice List', url: '/desk/list/sales_invoice', group: 'DocTypes' },
            { label: 'Open Purchase Invoice List', url: '/desk/list/purchase_invoice', group: 'DocTypes' },
            { label: 'Open Sales Order List', url: '/desk/list/sales_order', group: 'DocTypes' },
            { label: 'Open Purchase Order List', url: '/desk/list/purchase_order', group: 'DocTypes' },
            { label: 'Open Item List', url: '/desk/list/item', group: 'DocTypes' },
            { label: 'Open Customer List', url: '/desk/list/customer', group: 'DocTypes' },
            { label: 'Open Supplier List', url: '/desk/list/supplier', group: 'DocTypes' },
            { label: 'Open Journal Entry List', url: '/desk/list/journal_entry', group: 'DocTypes' },
            { label: 'Open Stock Entry List', url: '/desk/list/stock_entry', group: 'DocTypes' },
            { label: 'Open Work Order List', url: '/desk/list/work_order', group: 'DocTypes' },
            { label: 'Open BOM List', url: '/desk/list/bom', group: 'DocTypes' },
            { label: 'Open Employee List', url: '/desk/list/employee', group: 'DocTypes' },
            { label: 'Open Lead List', url: '/desk/list/lead', group: 'DocTypes' },
            { label: 'Open Asset List', url: '/desk/list/asset', group: 'DocTypes' },
            { label: '+ New Sales Invoice', url: '/desk/form/sales_invoice/new', group: 'Actions' },
            { label: '+ New Purchase Invoice', url: '/desk/form/purchase_invoice/new', group: 'Actions' },
            { label: '+ New Sales Order', url: '/desk/form/sales_order/new', group: 'Actions' },
            { label: '+ New Item', url: '/desk/form/item/new', group: 'Actions' },
            { label: '+ New Customer', url: '/desk/form/customer/new', group: 'Actions' },
            { label: '+ New Journal Entry', url: '/desk/form/journal_entry/new', group: 'Actions' },
            { label: 'Report: General Ledger', url: '/desk/list/journal_entry', group: 'Reports' },
            { label: 'Report: Accounts Receivable', url: '/desk/list/sales_invoice', group: 'Reports' },
            { label: 'Report: Stock Balance', url: '/desk/list/stock_entry', group: 'Reports' }
        ];

        const searchInput = document.getElementById('omni-search');
        const popover = document.getElementById('omni-popover');
        let selectedIdx = 0;

        function renderOmniResults(q) {
            if (!q) { popover.style.display = 'none'; return; }
            const matches = awesomeItems.filter(item => item.label.toLowerCase().includes(q));
            if (matches.length === 0) {
                popover.innerHTML = '<div style="padding:0.75rem;color:var(--text-secondary);font-size:0.85rem;">No matching documents or shortcuts</div>';
                popover.style.display = 'block';
                return;
            }
            selectedIdx = 0;
            popover.innerHTML = matches.map((m, idx) => `
                <a href="${m.url}" class="omni-item ${idx === 0 ? 'active' : ''}" data-idx="${idx}">
                    <span>${m.label}</span>
                    <span class="omni-item-badge">${m.group}</span>
                </a>
            `).join('');
            popover.style.display = 'block';
        }

        if (searchInput) {
            searchInput.addEventListener('input', (e) => {
                const q = e.target.value.toLowerCase().trim();
                renderOmniResults(q);
                // Also filter cards in current view
                document.querySelectorAll('.card').forEach(card => {
                    const text = card.textContent.toLowerCase();
                    card.style.display = (!q || text.includes(q)) ? '' : 'none';
                });
            });

            searchInput.addEventListener('keydown', (e) => {
                const items = popover.querySelectorAll('.omni-item');
                if (popover.style.display === 'block' && items.length > 0) {
                    if (e.key === 'ArrowDown') {
                        e.preventDefault();
                        selectedIdx = (selectedIdx + 1) % items.length;
                        items.forEach((it, i) => it.classList.toggle('active', i === selectedIdx));
                    } else if (e.key === 'ArrowUp') {
                        e.preventDefault();
                        selectedIdx = (selectedIdx - 1 + items.length) % items.length;
                        items.forEach((it, i) => it.classList.toggle('active', i === selectedIdx));
                    } else if (e.key === 'Enter') {
                        e.preventDefault();
                        if (items[selectedIdx]) window.location.href = items[selectedIdx].getAttribute('href');
                    } else if (e.key === 'Escape') {
                        popover.style.display = 'none';
                    }
                }
            });
        }

        document.addEventListener('click', (e) => {
            if (!e.target.closest('.omnibar-wrapper') && popover) popover.style.display = 'none';
        });

        document.addEventListener('keydown', (e) => {
            if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
                e.preventDefault();
                if (searchInput) searchInput.focus();
            }
            if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'b') {
                e.preventDefault();
                sidebarToggle.click();
            }
        });
    </script>
</body>
</html>"#;

/// Compiles the complete, standalone Enterprise Desk HTML user interface.
#[must_use]
pub fn render_desk_shell_html(persona: &str, user_name: &str) -> String {
    let workspaces = get_desk_workspaces();
    let mut cards_html = String::new();
    let mut sidebar_links = String::new();

    for ws in &workspaces {
        let doctypes_pills = ws
            .doctypes
            .iter()
            .map(|d| {
                let doc_route = format!("/desk/list/{}", d.to_lowercase().replace(' ', "_"));
                format!("<a href=\"{doc_route}\" class=\"pill\">{d}</a>")
            })
            .collect::<Vec<_>>()
            .join(" ");

        cards_html.push_str(&format!(
            r#"<div class="card" onclick="window.location.href='{}'">
                <div class="card-header">
                    <div class="card-icon">
                        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="{}"/>
                        </svg>
                    </div>
                    <span class="badge">{}</span>
                </div>
                <h3 class="card-title">{}</h3>
                <p class="card-desc">{}</p>
                <div class="pills-container">{}</div>
            </div>"#,
            ws.route, ws.icon_svg, ws.badge, ws.title, ws.description, doctypes_pills,
        ));

        sidebar_links.push_str(&format!(
            r#"<a href="{}" class="nav-item" title="{}">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="{}"/>
                </svg>
                <span class="nav-text">{}</span>
            </a>"#,
            ws.route, ws.title, ws.icon_svg, ws.title,
        ));
    }

    let desk_sel = if persona == "Enterprise Desk" || persona == "Desk" {
        "selected"
    } else {
        ""
    };
    let portal_sel = if persona == "Customer Portal" {
        "selected"
    } else {
        ""
    };
    let worker_sel = if persona == "Warehouse Scanner" {
        "selected"
    } else {
        ""
    };
    let factory_sel = if persona == "Shopfloor MES" {
        "selected"
    } else {
        ""
    };
    let approvals_sel = if persona == "Approval Deck" {
        "selected"
    } else {
        ""
    };
    let admin_sel = if persona == "Admin Cockpit" {
        "selected"
    } else {
        ""
    };
    let avatar_char = user_name.chars().next().unwrap_or('A').to_string();

    DESK_BASE_HTML
        .replace("__PERSONA__", persona)
        .replace("__USERNAME__", user_name)
        .replace("__AVATAR__", &avatar_char)
        .replace("__DESK_SEL__", desk_sel)
        .replace("__PORTAL_SEL__", portal_sel)
        .replace("__WORKER_SEL__", worker_sel)
        .replace("__FACTORY_SEL__", factory_sel)
        .replace("__APPROVALS_SEL__", approvals_sel)
        .replace("__ADMIN_SEL__", admin_sel)
        .replace("__SIDEBAR_LINKS__", &sidebar_links)
        .replace("__CARDS__", &cards_html)
}

/// Compiles an interactive, responsive HTML List View for any DocType.
#[must_use]
pub fn render_list_view_html(doctype: &str, user_name: &str) -> String {
    let avatar_char = user_name.chars().next().unwrap_or('A').to_string();
    let display_title = doctype.replace('_', " ");
    format!(
        r##"<!DOCTYPE html>
<html lang="en" data-theme="dark">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{display_title} List — RustNext Desk</title>
    <style>
        :root {{
            --bg-primary: #0a0b10; --bg-secondary: #12141c; --bg-card: rgba(22, 25, 37, 0.75);
            --border: rgba(255, 255, 255, 0.08); --accent: #6366f1; --accent-glow: rgba(99, 102, 241, 0.25);
            --text-primary: #f8fafc; --text-secondary: #94a3b8; --success: #10b981; --warning: #f59e0b; --danger: #ef4444;
        }}
        [data-theme="light"] {{
            --bg-primary: #f8fafc; --bg-secondary: #ffffff; --bg-card: #ffffff;
            --border: rgba(0, 0, 0, 0.08); --accent: #4f46e5; --accent-glow: rgba(79, 70, 229, 0.15);
            --text-primary: #0f172a; --text-secondary: #64748b;
        }}
        * {{ margin: 0; padding: 0; box-sizing: border-box; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }}
        ::-webkit-scrollbar {{ width: 6px; height: 6px; }}
        ::-webkit-scrollbar-track {{ background: transparent; }}
        ::-webkit-scrollbar-thumb {{ background: rgba(255, 255, 255, 0.15); border-radius: 9999px; }}
        ::-webkit-scrollbar-thumb:hover {{ background: rgba(255, 255, 255, 0.25); }}
        .btn:active, button:active {{ transform: scale(0.98); }}
        body {{ background-color: var(--bg-primary); color: var(--text-primary); min-height: 100vh; display: flex; flex-direction: column; }}
        header {{
            position: sticky; top: 0; z-index: 50; background: rgba(10, 11, 16, 0.88); backdrop-filter: blur(16px);
            border-bottom: 1px solid var(--border); padding: 0.75rem 2rem; display: flex; justify-content: space-between; align-items: center;
        }}
        .brand {{ display: flex; align-items: center; gap: 0.75rem; font-weight: 700; font-size: 1.25rem; text-decoration: none; color: inherit; }}
        .brand-logo {{ width: 32px; height: 32px; background: linear-gradient(135deg, #6366f1, #a855f7); border-radius: 8px; display: flex; align-items: center; justify-content: center; font-weight: 900; color: white; }}
        .nav-actions {{ display: flex; align-items: center; gap: 1rem; }}
        .user-chip {{ display: flex; align-items: center; gap: 0.5rem; font-size: 0.875rem; color: var(--text-secondary); }}
        .user-avatar {{ width: 32px; height: 32px; border-radius: 50%; background: #334155; display: flex; align-items: center; justify-content: center; color: white; font-weight: 600; }}
        main {{ max-width: 1440px; margin: 0 auto; width: 100%; padding: 2rem; flex: 1; }}
        .action-bar {{ display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.25rem; flex-wrap: wrap; gap: 1rem; }}
        .breadcrumbs {{ display: flex; align-items: center; gap: 0.5rem; font-size: 0.875rem; color: var(--text-secondary); margin-bottom: 0.5rem; }}
        .breadcrumbs a {{ color: var(--accent); text-decoration: none; }}
        .page-title {{ font-size: 1.75rem; font-weight: 800; text-transform: capitalize; display: flex; align-items: center; gap: 0.75rem; }}
        .btn {{ padding: 0.5rem 1.2rem; border-radius: 8px; font-weight: 600; font-size: 0.875rem; cursor: pointer; border: none; transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1); display: inline-flex; align-items: center; gap: 0.5rem; text-decoration: none; }}
        .btn-primary {{ background: var(--accent); color: white; box-shadow: 0 0 12px var(--accent-glow); }}
        .btn-primary:hover {{ opacity: 0.9; transform: translateY(-1px); }}
        .btn-secondary {{ background: var(--bg-secondary); color: var(--text-primary); border: 1px solid var(--border); }}
        .btn-secondary:hover {{ border-color: var(--accent); }}
        .btn-danger {{ background: rgba(239, 68, 68, 0.2); color: #f87171; border: 1px solid rgba(239, 68, 68, 0.3); }}
        .btn-danger:hover {{ background: rgba(239, 68, 68, 0.35); }}
        .table-container {{ background: var(--bg-card); border: 1px solid var(--border); border-radius: 12px; overflow: hidden; position: relative; }}
        
        .filter-section {{ padding: 1rem 1.5rem; border-bottom: 1px solid var(--border); display: flex; gap: 1rem; align-items: center; flex-wrap: wrap; justify-content: space-between; }}
        .search-input {{ background: var(--bg-secondary); border: 1px solid var(--border); color: var(--text-primary); padding: 0.5rem 1rem; border-radius: 8px; width: 280px; outline: none; font-size: 0.875rem; }}
        .search-input:focus {{ border-color: var(--accent); }}
        
        .status-pills-bar {{ display: flex; gap: 0.4rem; align-items: center; }}
        .status-pill {{ padding: 0.3rem 0.75rem; border-radius: 9999px; font-size: 0.75rem; font-weight: 600; cursor: pointer; border: 1px solid var(--border); background: var(--bg-secondary); color: var(--text-secondary); transition: all 0.15s; }}
        .status-pill.active {{ border-color: var(--accent); background: var(--accent-glow); color: var(--accent); }}
        
        .bulk-action-bar {{
            background: rgba(99, 102, 241, 0.15); border-bottom: 1px solid var(--accent);
            padding: 0.6rem 1.5rem; display: none; align-items: center; justify-content: space-between; font-size: 0.85rem;
        }}
        .bulk-actions-group {{ display: flex; gap: 0.5rem; }}
        
        table {{ width: 100%; border-collapse: collapse; text-align: left; font-size: 0.875rem; }}
        th {{ background: rgba(255, 255, 255, 0.02); padding: 0.85rem 1.25rem; color: var(--text-secondary); font-weight: 600; border-bottom: 1px solid var(--border); cursor: pointer; user-select: none; }}
        th:hover {{ color: var(--text-primary); }}
        td {{ padding: 0.85rem 1.25rem; border-bottom: 1px solid var(--border); color: var(--text-primary); }}
        tr.clickable-row {{ cursor: pointer; transition: background 0.15s; }}
        tr.clickable-row:hover {{ background: rgba(99, 102, 241, 0.08); }}
        
        .checkbox-cell {{ width: 36px; text-align: center; }}
        .checkbox-cell input {{ width: 16px; height: 16px; accent-color: var(--accent); cursor: pointer; }}
        
        .badge {{ font-size: 0.75rem; font-weight: 600; padding: 0.2rem 0.6rem; border-radius: 9999px; display: inline-flex; align-items: center; gap: 0.3rem; }}
        .badge-draft {{ background: rgba(245, 158, 11, 0.15); color: #fbbf24; border: 1px solid rgba(245, 158, 11, 0.3); }}
        .badge-submitted {{ background: rgba(16, 185, 129, 0.15); color: #34d399; border: 1px solid rgba(16, 185, 129, 0.3); }}
        .badge-cancelled {{ background: rgba(239, 68, 68, 0.15); color: #f87171; border: 1px solid rgba(239, 68, 68, 0.3); }}
        .empty-state {{ text-align: center; padding: 4rem 2rem; color: var(--text-secondary); }}
        
        .pagination-bar {{ padding: 0.75rem 1.5rem; border-top: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center; font-size: 0.8rem; color: var(--text-secondary); }}
        .pagination-controls {{ display: flex; align-items: center; gap: 0.5rem; }}
        #toast {{ position: fixed; bottom: 24px; right: 24px; background: rgba(18,24,38,0.95); border: 1px solid var(--border); padding: 12px 18px; border-radius: 8px; display: none; z-index: 9999; }}
    </style>
</head>
<body>
    <header>
        <a href="/desk" class="brand">
            <div class="brand-logo">R</div>
            <span>RustNext Desk</span>
        </a>
        <div class="nav-actions">
            <a href="/desk" class="btn btn-secondary"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 12H5M12 19l-7-7 7-7"/></svg> Back to Workspace</a>
            <div class="user-chip">
                <div class="user-avatar">{avatar_char}</div>
                <span>{user_name}</span>
            </div>
        </div>
    </header>
    <main>
        <div class="breadcrumbs">
            <a href="/desk">Desk</a> <span>/</span> <span>{display_title}</span>
        </div>
        <div class="action-bar">
            <div>
                <h1 class="page-title">
                    <span>{display_title}</span>
                    <span id="record-count" style="font-size: 0.85rem; font-weight: 600; padding: 0.2rem 0.6rem; border-radius: 9999px; background: rgba(99, 102, 241, 0.15); color: var(--accent);">Loading...</span>
                </h1>
            </div>
            <div style="display: flex; gap: 0.75rem;">
                <button class="btn btn-secondary" onclick="exportCsv()"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/></svg> Export CSV</button>
                <button class="btn btn-secondary" onclick="loadRecords()"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21.5 2v6h-6M21.34 15.57a10 10 0 1 1-.57-8.38l5.67-5.67"/></svg> Refresh</button>
                <a href="/desk/form/{doctype}/new" class="btn btn-primary"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 5v14M5 12h14"/></svg> New {display_title}</a>
            </div>
        </div>

        <div class="table-container">
            <div class="filter-section">
                <input type="text" id="filter-search" class="search-input" placeholder="Search by name, ID, party, or details..." oninput="filterTable()" />
                <div class="status-pills-bar">
                    <span class="status-pill active" onclick="setStatusFilter('all', this)">All</span>
                    <span class="status-pill" onclick="setStatusFilter('draft', this)">● Draft</span>
                    <span class="status-pill" onclick="setStatusFilter('submitted', this)">● Submitted</span>
                    <span class="status-pill" onclick="setStatusFilter('cancelled', this)">● Cancelled</span>
                </div>
            </div>

            <div class="bulk-action-bar" id="bulk-action-bar">
                <div style="font-weight:600;"><span id="selected-count">0</span> document(s) selected</div>
                <div class="bulk-actions-group">
                    <button class="btn btn-secondary" style="padding:0.3rem 0.7rem; font-size:0.75rem;" onclick="batchSubmit()">Submit Selected</button>
                    <button class="btn btn-secondary" style="padding:0.3rem 0.7rem; font-size:0.75rem;" onclick="batchCancel()">Cancel Selected</button>
                    <button class="btn btn-danger" style="padding:0.3rem 0.7rem; font-size:0.75rem;" onclick="batchDelete()">Delete Selected</button>
                    <button class="btn btn-secondary" style="padding:0.3rem 0.7rem; font-size:0.75rem;" onclick="deselectAll()">Deselect</button>
                </div>
            </div>

            <table>
                <thead>
                    <tr>
                        <th class="checkbox-cell"><input type="checkbox" id="select-all" onchange="toggleSelectAll(this)" /></th>
                        <th onclick="toggleSort('name')">Document Identifier <span id="sort-name">↕</span></th>
                        <th onclick="toggleSort('status')">Status <span id="sort-status">↕</span></th>
                        <th onclick="toggleSort('created')">Created / Date <span id="sort-created">↕</span></th>
                        <th>Action</th>
                    </tr>
                </thead>
                <tbody id="table-body">
                    <tr><td colspan="5" class="empty-state">Loading {display_title} records from SurrealDB...</td></tr>
                </tbody>
            </table>

            <div class="pagination-bar">
                <div>
                    <span>Rows per page: </span>
                    <select id="page-size-select" onchange="changePageSize(this.value)" style="background:var(--bg-secondary);color:var(--text-primary);border:1px solid var(--border);border-radius:4px;padding:0.2rem 0.4rem;">
                        <option value="20" selected>20</option>
                        <option value="50">50</option>
                        <option value="100">100</option>
                    </select>
                </div>
                <div class="pagination-controls">
                    <button class="btn btn-secondary" style="padding:0.25rem 0.6rem; font-size:0.75rem;" id="prev-page" onclick="prevPage()">&lt; Prev</button>
                    <span id="page-indicator">Page 1</span>
                    <button class="btn btn-secondary" style="padding:0.25rem 0.6rem; font-size:0.75rem;" id="next-page" onclick="nextPage()">Next &gt;</button>
                </div>
            </div>
        </div>
    </main>
    <div id="toast"></div>

    <script>
        let allRecords = [];
        let filteredRecords = [];
        let activeStatusFilter = 'all';
        let currentSort = {{ field: 'name', asc: true }};
        let currentPage = 1;
        let pageSize = 20;

        async function loadRecords() {{
            const tbody = document.getElementById('table-body');
            const countSpan = document.getElementById('record-count');
            try {{
                const res = await fetch('/api/v2/document/{doctype}?limit=250');
                if (!res.ok) throw new Error('Status: ' + res.status);
                const json = await res.json();
                allRecords = json.data || [];
                applyFilters();
            }} catch (err) {{
                tbody.innerHTML = `<tr><td colspan="5" class="empty-state" style="color:var(--danger)">Failed to load records: ${{err.message}}</td></tr>`;
                countSpan.textContent = 'Error';
            }}
        }}

        function setStatusFilter(status, elem) {{
            activeStatusFilter = status;
            document.querySelectorAll('.status-pill').forEach(p => p.classList.remove('active'));
            if (elem) elem.classList.add('active');
            applyFilters();
        }}

        function applyFilters() {{
            const q = document.getElementById('filter-search').value.toLowerCase().trim();
            filteredRecords = allRecords.filter(r => {{
                const matchesText = !q || JSON.stringify(r).toLowerCase().includes(q);
                let matchesStatus = true;
                if (activeStatusFilter === 'draft') matchesStatus = (r.docstatus === 0 || !r.docstatus);
                else if (activeStatusFilter === 'submitted') matchesStatus = (r.docstatus === 1);
                else if (activeStatusFilter === 'cancelled') matchesStatus = (r.docstatus === 2);
                return matchesText && matchesStatus;
            }});

            // Sort
            filteredRecords.sort((a, b) => {{
                let valA = a[currentSort.field] || a.name || '';
                let valB = b[currentSort.field] || b.name || '';
                if (typeof valA === 'string') valA = valA.toLowerCase();
                if (typeof valB === 'string') valB = valB.toLowerCase();
                if (valA < valB) return currentSort.asc ? -1 : 1;
                if (valA > valB) return currentSort.asc ? 1 : -1;
                return 0;
            }});

            currentPage = 1;
            renderTablePage();
        }}

        function renderTablePage() {{
            const tbody = document.getElementById('table-body');
            const countSpan = document.getElementById('record-count');
            const total = filteredRecords.length;
            countSpan.textContent = `${{total}} record(s)`;

            if (total === 0) {{
                tbody.innerHTML = `<tr><td colspan="5" class="empty-state">No records found matching filters. Click "+ New {display_title}" to create one.</td></tr>`;
                document.getElementById('page-indicator').textContent = 'Page 0 of 0';
                return;
            }}

            const totalPages = Math.ceil(total / pageSize);
            const start = (currentPage - 1) * pageSize;
            const paged = filteredRecords.slice(start, start + pageSize);

            document.getElementById('page-indicator').textContent = `Page ${{currentPage}} of ${{totalPages}} (${{start + 1}}-${{Math.min(start + pageSize, total)}} of ${{total}})`;

            tbody.innerHTML = paged.map(r => {{
                const docname = r.name || r.id || 'unnamed';
                const status = r.docstatus === 1 ? 'Submitted' : (r.docstatus === 2 ? 'Cancelled' : 'Draft');
                const badgeClass = r.docstatus === 1 ? 'badge-submitted' : (r.docstatus === 2 ? 'badge-cancelled' : 'badge-draft');
                const created = r.creation || r.created_at || r.posting_date || 'Just now';
                return `<tr>
                    <td class="checkbox-cell"><input type="checkbox" class="row-checkbox" data-name="${{docname}}" onchange="updateSelectedCount()" /></td>
                    <td style="font-weight:600; color:var(--accent); cursor:pointer;" onclick="window.location.href='/desk/form/{doctype}/${{encodeURIComponent(docname)}}'">${{docname}}</td>
                    <td><span class="badge ${{badgeClass}}">${{status}}</span></td>
                    <td style="color:var(--text-secondary);">${{created}}</td>
                    <td><a href="/desk/form/{doctype}/${{encodeURIComponent(docname)}}" class="btn btn-secondary" style="padding:0.25rem 0.6rem;font-size:0.75rem;">Edit</a></td>
                </tr>`;
            }}).join('');

            updateSelectedCount();
        }}

        function toggleSort(field) {{
            if (currentSort.field === field) {{
                currentSort.asc = !currentSort.asc;
            }} else {{
                currentSort.field = field;
                currentSort.asc = true;
            }}
            document.querySelectorAll('[id^="sort-"]').forEach(s => s.textContent = '↕');
            const arrow = document.getElementById(`sort-${{field}}`);
            if (arrow) arrow.textContent = currentSort.asc ? '▲' : '▼';
            applyFilters();
        }}

        function toggleSelectAll(master) {{
            document.querySelectorAll('.row-checkbox').forEach(cb => cb.checked = master.checked);
            updateSelectedCount();
        }}

        function deselectAll() {{
            const master = document.getElementById('select-all');
            if (master) master.checked = false;
            document.querySelectorAll('.row-checkbox').forEach(cb => cb.checked = false);
            updateSelectedCount();
        }}

        function getSelectedNames() {{
            return Array.from(document.querySelectorAll('.row-checkbox:checked')).map(cb => cb.getAttribute('data-name'));
        }}

        function updateSelectedCount() {{
            const selected = getSelectedNames();
            const bar = document.getElementById('bulk-action-bar');
            const countLabel = document.getElementById('selected-count');
            countLabel.textContent = selected.length;
            bar.style.display = selected.length > 0 ? 'flex' : 'none';
        }}

        async function batchSubmit() {{
            const names = getSelectedNames();
            if (names.length === 0) return;
            if (!confirm(`Submit ${{names.length}} selected document(s)?`)) return;
            for (const name of names) {{
                await fetch(`/api/v2/document/{doctype}/${{encodeURIComponent(name)}}/submit`, {{ method: 'POST' }});
            }}
            loadRecords();
        }}

        async function batchCancel() {{
            const names = getSelectedNames();
            if (names.length === 0) return;
            if (!confirm(`Cancel ${{names.length}} selected document(s)?`)) return;
            for (const name of names) {{
                await fetch(`/api/v2/document/{doctype}/${{encodeURIComponent(name)}}/cancel`, {{ method: 'POST' }});
            }}
            loadRecords();
        }}

        async function batchDelete() {{
            const names = getSelectedNames();
            if (names.length === 0) return;
            if (!confirm(`Permanently delete ${{names.length}} selected document(s)?`)) return;
            for (const name of names) {{
                await fetch(`/api/v2/document/{doctype}/${{encodeURIComponent(name)}}`, {{ method: 'DELETE' }});
            }}
            loadRecords();
        }}

        function changePageSize(size) {{
            pageSize = parseInt(size, 10);
            renderTablePage();
        }}

        function prevPage() {{
            if (currentPage > 1) {{ currentPage--; renderTablePage(); }}
        }}

        function nextPage() {{
            const totalPages = Math.ceil(filteredRecords.length / pageSize);
            if (currentPage < totalPages) {{ currentPage++; renderTablePage(); }}
        }}

        function filterTable() {{
            applyFilters();
        }}

        function exportCsv() {{
            if (filteredRecords.length === 0) {{ alert('No records to export'); return; }}
            const keys = Object.keys(filteredRecords[0]).filter(k => typeof filteredRecords[0][k] !== 'object');
            const csvRows = [keys.join(',')];
            for (const r of filteredRecords) {{
                const vals = keys.map(k => `"${{String(r[k] || '').replace(/"/g, '""')}}"`);
                csvRows.push(vals.join(','));
            }}
            const blob = new Blob([csvRows.join('\\n')], {{ type: 'text/csv' }});
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `{doctype}_export_${{new Date().toISOString().split('T')[0]}}.csv`;
            a.click();
        }}

        loadRecords();
    </script>
</body>
</html>"##
    )
}

/// Compiles an interactive, responsive HTML Form View for editing or creating documents.
#[must_use]
pub fn render_form_view_html(doctype: &str, docname: &str, user_name: &str) -> String {
    let avatar_char = user_name.chars().next().unwrap_or('A').to_string();
    let display_title = doctype.replace('_', " ");
    let is_new = docname == "new";
    let is_itemized = matches!(
        doctype,
        "sales_invoice"
            | "sales_order"
            | "purchase_order"
            | "purchase_invoice"
            | "stock_entry"
            | "bom"
    );
    let child_table_display = if is_itemized { "block" } else { "none" };

    format!(
        r##"<!DOCTYPE html>
<html lang="en" data-theme="dark">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{display_title} — {docname} — RustNext Desk</title>
    <style>
        :root {{
            --bg-primary: #0a0b10; --bg-secondary: #12141c; --bg-card: rgba(22, 25, 37, 0.75);
            --border: rgba(255, 255, 255, 0.08); --accent: #6366f1; --accent-glow: rgba(99, 102, 241, 0.25);
            --text-primary: #f8fafc; --text-secondary: #94a3b8; --success: #10b981; --warning: #f59e0b; --danger: #ef4444;
        }}
        [data-theme="light"] {{
            --bg-primary: #f8fafc; --bg-secondary: #ffffff; --bg-card: #ffffff;
            --border: rgba(0, 0, 0, 0.08); --accent: #4f46e5; --accent-glow: rgba(79, 70, 229, 0.15);
            --text-primary: #0f172a; --text-secondary: #64748b;
        }}
        * {{ margin: 0; padding: 0; box-sizing: border-box; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }}
        ::-webkit-scrollbar {{ width: 6px; height: 6px; }}
        ::-webkit-scrollbar-track {{ background: transparent; }}
        ::-webkit-scrollbar-thumb {{ background: rgba(255, 255, 255, 0.15); border-radius: 9999px; }}
        ::-webkit-scrollbar-thumb:hover {{ background: rgba(255, 255, 255, 0.25); }}
        .btn:active, button:active {{ transform: scale(0.98); }}
        body {{ background-color: var(--bg-primary); color: var(--text-primary); min-height: 100vh; display: flex; flex-direction: column; }}
        header {{
            position: sticky; top: 0; z-index: 50; background: rgba(10, 11, 16, 0.88); backdrop-filter: blur(16px);
            border-bottom: 1px solid var(--border); padding: 0.75rem 2rem; display: flex; justify-content: space-between; align-items: center;
        }}
        .brand {{ display: flex; align-items: center; gap: 0.75rem; font-weight: 700; font-size: 1.25rem; text-decoration: none; color: inherit; }}
        .brand-logo {{ width: 32px; height: 32px; background: linear-gradient(135deg, #6366f1, #a855f7); border-radius: 8px; display: flex; align-items: center; justify-content: center; font-weight: 900; color: white; }}
        .nav-actions {{ display: flex; align-items: center; gap: 1rem; }}
        .user-chip {{ display: flex; align-items: center; gap: 0.5rem; font-size: 0.875rem; color: var(--text-secondary); }}
        .user-avatar {{ width: 32px; height: 32px; border-radius: 50%; background: #334155; display: flex; align-items: center; justify-content: center; color: white; font-weight: 600; }}
        main {{ max-width: 1100px; margin: 0 auto; width: 100%; padding: 2rem; flex: 1; }}
        .breadcrumbs {{ display: flex; align-items: center; gap: 0.5rem; font-size: 0.875rem; color: var(--text-secondary); margin-bottom: 0.5rem; }}
        .breadcrumbs a {{ color: var(--accent); text-decoration: none; }}
        .action-bar {{ display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.25rem; flex-wrap: wrap; gap: 1rem; }}
        .page-title {{ font-size: 1.75rem; font-weight: 800; display: flex; align-items: center; gap: 0.75rem; }}
        .btn {{ padding: 0.5rem 1.2rem; border-radius: 8px; font-weight: 600; font-size: 0.875rem; cursor: pointer; border: none; transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1); display: inline-flex; align-items: center; gap: 0.5rem; text-decoration: none; }}
        .btn-primary {{ background: var(--accent); color: white; box-shadow: 0 0 12px var(--accent-glow); }}
        .btn-primary:hover {{ opacity: 0.9; transform: translateY(-1px); }}
        .btn-success {{ background: var(--success); color: white; }}
        .btn-danger {{ background: var(--danger); color: white; }}
        .btn-secondary {{ background: var(--bg-secondary); color: var(--text-primary); border: 1px solid var(--border); }}
        .btn-secondary:hover {{ border-color: var(--accent); }}
        
        .connections-dashboard {{
            background: var(--bg-card); border: 1px solid var(--border); border-radius: 12px;
            padding: 1rem 1.5rem; margin-bottom: 1.25rem; display: flex; gap: 1rem; align-items: center; flex-wrap: wrap;
        }}
        .conn-item {{
            display: inline-flex; align-items: center; gap: 0.5rem; background: var(--bg-secondary);
            border: 1px solid var(--border); padding: 0.35rem 0.75rem; border-radius: 8px; font-size: 0.8rem;
            color: var(--text-primary); text-decoration: none; cursor: pointer; transition: all 0.15s;
        }}
        .conn-item:hover {{ border-color: var(--accent); }}
        .conn-count {{ font-weight: 700; color: var(--accent); background: var(--accent-glow); padding: 0.1rem 0.4rem; border-radius: 4px; font-size: 0.75rem; }}
        
        .form-card {{ background: var(--bg-card); border: 1px solid var(--border); border-radius: 14px; padding: 2rem; margin-bottom: 1.5rem; }}
        .form-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 1.25rem; margin-bottom: 1.25rem; }}
        .form-group {{ display: flex; flex-direction: column; gap: 0.35rem; position: relative; }}
        .form-group label {{ font-size: 0.75rem; font-weight: 700; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.05em; }}
        .form-control {{ background: var(--bg-secondary); border: 1px solid var(--border); color: var(--text-primary); padding: 0.55rem 0.8rem; border-radius: 8px; font-size: 0.875rem; outline: none; transition: border-color 0.2s; width: 100%; }}
        .form-control:focus {{ border-color: var(--accent); }}
        textarea.form-control {{ min-height: 90px; resize: vertical; }}
        
        .child-table-section {{ margin-top: 1.5rem; border-top: 1px solid var(--border); padding-top: 1.5rem; }}
        .child-table {{ width: 100%; border-collapse: collapse; margin-top: 0.75rem; font-size: 0.85rem; }}
        .child-table th {{ background: rgba(255, 255, 255, 0.02); padding: 0.6rem 0.8rem; text-align: left; color: var(--text-secondary); font-weight: 600; border-bottom: 1px solid var(--border); }}
        .child-table td {{ padding: 0.5rem 0.8rem; border-bottom: 1px solid var(--border); }}
        .child-table input {{ background: var(--bg-secondary); border: 1px solid var(--border); color: var(--text-primary); padding: 0.4rem 0.6rem; border-radius: 6px; font-size: 0.85rem; width: 100%; }}
        .child-table input:focus {{ border-color: var(--accent); outline: none; }}
        
        .badge {{ font-size: 0.75rem; font-weight: 600; padding: 0.2rem 0.6rem; border-radius: 9999px; }}
        .badge-draft {{ background: rgba(245, 158, 11, 0.15); color: #fbbf24; border: 1px solid rgba(245, 158, 11, 0.3); }}
        .badge-submitted {{ background: rgba(16, 185, 129, 0.15); color: #34d399; border: 1px solid rgba(16, 185, 129, 0.3); }}
        .badge-cancelled {{ background: rgba(239, 68, 68, 0.15); color: #f87171; border: 1px solid rgba(239, 68, 68, 0.3); }}
        #toast {{ position: fixed; bottom: 24px; right: 24px; background: rgba(18,24,38,0.95); border: 1px solid var(--border); padding: 12px 18px; border-radius: 8px; font-size: 13px; z-index: 9999; display: none; }}
    </style>
</head>
<body>
    <header>
        <a href="/desk" class="brand">
            <div class="brand-logo">R</div>
            <span>RustNext Desk</span>
        </a>
        <div class="nav-actions">
            <a href="/desk/list/{doctype}" class="btn btn-secondary"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 12H5M12 19l-7-7 7-7"/></svg> Back to List</a>
            <div class="user-chip">
                <div class="user-avatar">{avatar_char}</div>
                <span>{user_name}</span>
            </div>
        </div>
    </header>
    <main>
        <div class="breadcrumbs">
            <a href="/desk">Desk</a> <span>/</span> <a href="/desk/list/{doctype}">{display_title}</a> <span>/</span> <span>{docname}</span>
        </div>
        <div class="action-bar">
            <div>
                <h1 class="page-title">
                    <span id="title-docname">{docname}</span>
                    <span id="docstatus-badge" class="badge badge-draft">Draft</span>
                </h1>
            </div>
            <div style="display: flex; gap: 0.6rem; flex-wrap: wrap;">
                <button id="btn-save" class="btn btn-primary" onclick="saveDocument()"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg> Save</button>
                <button id="btn-submit" class="btn btn-success" style="display:none;" onclick="submitDocument()"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg> Submit</button>
                <button id="btn-cancel" class="btn btn-danger" style="display:none;" onclick="cancelDocument()"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg> Cancel</button>
                <button id="btn-amend" class="btn btn-primary" style="display:none;" onclick="amendDocument()"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg> Amend</button>
                <button id="btn-print" class="btn btn-secondary" onclick="printDocument()"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 6 2 18 2 18 9"/><path d="M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2"/><rect x="6" y="14" width="12" height="8"/></svg> Print</button>
                <button id="btn-duplicate" class="btn btn-secondary" onclick="duplicateDocument()"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg> Duplicate</button>
                <button id="btn-delete" class="btn btn-danger" style="display:none;" onclick="deleteDocument()"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg> Delete</button>
            </div>
        </div>

        <div class="connections-dashboard" id="connections-dashboard">
            <span style="font-size:0.75rem; font-weight:700; color:var(--text-secondary); text-transform:uppercase;">Connections:</span>
            <a href="/desk/list/sales_invoice" class="conn-item">Payments <span class="conn-count" id="conn-payments">0</span></a>
            <a href="/desk/list/sales_order" class="conn-item">Orders <span class="conn-count" id="conn-orders">0</span></a>
            <a href="/desk/list/journal_entry" class="conn-item">GL Vouchers <span class="conn-count" id="conn-gl">0</span></a>
            <button type="button" class="btn btn-secondary" style="padding:0.2rem 0.5rem; font-size:0.75rem; margin-left:auto;" onclick="showToast('Connections linked via SurrealDB graph relations', 'info')">+ Link Reference</button>
        </div>

        <form id="doc-form" onsubmit="event.preventDefault(); saveDocument();">
            <div class="form-card">
                <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom: 1.25rem;">
                    <div style="font-size:0.875rem; font-weight:700; color:var(--text-secondary); text-transform:uppercase; letter-spacing:0.05em;" id="form-section-title">Document Properties</div>
                    <button type="button" class="btn btn-secondary" style="padding:0.25rem 0.6rem; font-size:0.75rem;" onclick="toggleRawView()">Toggle JSON View</button>
                </div>
                <div id="form-grid" class="form-grid">
                    <div class="form-group">
                        <label>Identifier / Name</label>
                        <input type="text" id="field-name" data-fieldname="name" class="form-control" value="{docname}" disabled />
                    </div>
                    <div class="form-group">
                        <label>Posting / Transaction Date</label>
                        <input type="date" id="field-posting_date" data-fieldname="posting_date" class="form-control" />
                    </div>
                    <div class="form-group">
                        <label>Party / Reference</label>
                        <input type="text" id="field-party" data-fieldname="party" class="form-control" placeholder="Customer / Supplier / Reference..." />
                    </div>
                    <div class="form-group">
                        <label>Total Amount / Value</label>
                        <input type="number" step="0.01" id="field-total" data-fieldname="total" class="form-control" placeholder="0.00" />
                    </div>
                </div>

                <div id="child-table-container" class="child-table-section" style="display:{child_table_display};">
                    <div style="display:flex; justify-content:space-between; align-items:center;">
                        <label style="font-size:0.75rem; font-weight:700; color:var(--text-secondary); text-transform:uppercase;">Items / Child Table</label>
                        <button type="button" class="btn btn-secondary" style="padding:0.25rem 0.6rem; font-size:0.75rem;" onclick="addChildTableRow()">+ Add Row</button>
                    </div>
                    <table class="child-table">
                        <thead>
                            <tr>
                                <th style="width:40px;">#</th>
                                <th style="width:25%;">Item Code</th>
                                <th>Item Name / Description</th>
                                <th style="width:15%;">Qty</th>
                                <th style="width:15%;">Rate</th>
                                <th style="width:15%;">Amount</th>
                                <th style="width:40px;"></th>
                            </tr>
                        </thead>
                        <tbody id="child-table-body">
                        </tbody>
                    </table>
                </div>

                <div id="form-remarks-group" class="form-group" style="margin-top: 1.25rem;">
                    <label>Remarks / Description / Payload</label>
                    <textarea id="field-remarks" data-fieldname="remarks" class="form-control" placeholder="Enter notes or JSON data..."></textarea>
                </div>
                <div id="raw-json-container" class="form-group" style="margin-top: 1rem; display:none;">
                    <label>Raw Document Payload (JSON)</label>
                    <textarea id="field-raw-json" class="form-control" style="min-height: 180px; font-family: monospace; font-size: 12px;"></textarea>
                </div>
            </div>
        </form>
    </main>
    <div id="toast"></div>

    <script>
        const isNew = {is_new};
        const doctype = '{doctype}';
        let currentName = '{docname}';
        let schemaMeta = null;
        let loadedDoc = {{}};
        let childItems = [];

        function showToast(msg, ok = true) {{
            const t = document.getElementById('toast');
            t.style.display = 'block';
            t.style.borderColor = ok ? 'var(--success)' : 'var(--danger)';
            t.innerHTML = `<span style="color:${{ok ? 'var(--success)' : 'var(--danger)'}}">●</span> ${{msg}}`;
            setTimeout(() => {{ t.style.display = 'none'; }}, 3500);
        }}

        function toggleRawView() {{
            const rawBox = document.getElementById('raw-json-container');
            const isHidden = rawBox.style.display === 'none';
            rawBox.style.display = isHidden ? 'block' : 'none';
            if (isHidden) {{
                document.getElementById('field-raw-json').value = JSON.stringify(collectFormData(), null, 2);
            }}
        }}

        function addChildTableRow(item = {{ item_code: '', item_name: '', qty: 1, rate: 0, amount: 0 }}) {{
            childItems.push(item);
            renderChildTable();
        }}

        function removeChildTableRow(idx) {{
            childItems.splice(idx, 1);
            renderChildTable();
            recalcChildTable();
        }}

        function renderChildTable() {{
            const tbody = document.getElementById('child-table-body');
            if (!tbody) return;
            tbody.innerHTML = childItems.map((item, idx) => `
                <tr>
                    <td style="color:var(--text-secondary); text-align:center;">${{idx + 1}}</td>
                    <td><input type="text" value="${{item.item_code || ''}}" placeholder="Item Code" oninput="updateChildItem(${{idx}}, 'item_code', this.value)" /></td>
                    <td><input type="text" value="${{item.item_name || ''}}" placeholder="Description" oninput="updateChildItem(${{idx}}, 'item_name', this.value)" /></td>
                    <td><input type="number" step="1" value="${{item.qty || 1}}" oninput="updateChildItem(${{idx}}, 'qty', parseFloat(this.value)||0)" /></td>
                    <td><input type="number" step="0.01" value="${{item.rate || 0}}" oninput="updateChildItem(${{idx}}, 'rate', parseFloat(this.value)||0)" /></td>
                    <td><input type="number" step="0.01" value="${{(item.qty || 1) * (item.rate || 0)}}" readonly style="opacity:0.8;" /></td>
                    <td style="text-align:center;"><button type="button" class="btn btn-danger" style="padding:0.2rem 0.5rem;font-size:0.75rem;" onclick="removeChildTableRow(${{idx}})">✕</button></td>
                </tr>
            `).join('');
        }}

        function updateChildItem(idx, key, val) {{
            if (childItems[idx]) {{
                childItems[idx][key] = val;
                if (key === 'qty' || key === 'rate') {{
                    childItems[idx].amount = (childItems[idx].qty || 1) * (childItems[idx].rate || 0);
                    recalcChildTable();
                }}
            }}
        }}

        function recalcChildTable() {{
            let total = 0;
            childItems.forEach(it => {{
                total += (it.qty || 1) * (it.rate || 0);
            }});
            const totalInput = document.getElementById('field-total');
            if (totalInput) totalInput.value = total.toFixed(2);
        }}

        async function initSchema() {{
            try {{
                const res = await fetch(`/api/v2/meta/${{doctype}}`);
                if (res.ok) {{
                    const meta = await res.json();
                    if (meta && meta.fields && meta.fields.length > 0) {{
                        schemaMeta = meta;
                        renderDynamicForm(meta);
                    }}
                }}
            }} catch (e) {{
                console.warn('Meta fetch skipped or not available, using standard layout', e);
            }}
            await loadDocument();
        }}

        function renderDynamicForm(meta) {{
            const grid = document.getElementById('form-grid');
            let html = `
                <div class="form-group">
                    <label>Identifier / Name</label>
                    <input type="text" id="field-name" data-fieldname="name" class="form-control" value="${{currentName}}" disabled />
                </div>
            `;
            const supportedFields = meta.fields.filter(f => !f.hidden && f.fieldtype !== 'Table');
            for (const f of supportedFields) {{
                if (f.fieldname === 'name') continue;
                const reqdAttr = f.reqd ? 'required' : '';
                const reqdStar = f.reqd ? '<span style="color:var(--danger)">*</span>' : '';
                const readonlyAttr = f.read_only ? 'readonly' : '';
                let inputHtml = '';
                const ftype = f.fieldtype || 'Data';

                if (ftype === 'Currency' || ftype === 'Float') {{
                    inputHtml = `<input type="number" step="0.01" class="form-control" data-fieldname="${{f.fieldname}}" id="field-${{f.fieldname}}" placeholder="0.00" ${{reqdAttr}} ${{readonlyAttr}} />`;
                }} else if (ftype === 'Int') {{
                    inputHtml = `<input type="number" step="1" class="form-control" data-fieldname="${{f.fieldname}}" id="field-${{f.fieldname}}" placeholder="0" ${{reqdAttr}} ${{readonlyAttr}} />`;
                }} else if (ftype === 'Date') {{
                    inputHtml = `<input type="date" class="form-control" data-fieldname="${{f.fieldname}}" id="field-${{f.fieldname}}" ${{reqdAttr}} ${{readonlyAttr}} />`;
                }} else if (ftype === 'Datetime') {{
                    inputHtml = `<input type="datetime-local" class="form-control" data-fieldname="${{f.fieldname}}" id="field-${{f.fieldname}}" ${{reqdAttr}} ${{readonlyAttr}} />`;
                }} else if (ftype === 'Check') {{
                    inputHtml = `<div style="display:flex; align-items:center; height:38px;"><input type="checkbox" style="width:18px; height:18px; accent-color:var(--accent);" data-fieldname="${{f.fieldname}}" id="field-${{f.fieldname}}" /></div>`;
                }} else if (ftype === 'Select' && f.options) {{
                    const opts = f.options.split('\\n').map(o => o.trim()).filter(Boolean);
                    inputHtml = `<select class="form-control" data-fieldname="${{f.fieldname}}" id="field-${{f.fieldname}}" ${{reqdAttr}}>
                        <option value="">-- Select ${{f.label || f.fieldname}} --</option>
                        ${{opts.map(o => `<option value="${{o}}">${{o}}</option>`).join('')}}
                    </select>`;
                }} else if (ftype === 'SmallText' || ftype === 'LongText' || ftype === 'Code' || ftype === 'TextEditor') {{
                    inputHtml = `<textarea class="form-control" style="min-height:80px;" data-fieldname="${{f.fieldname}}" id="field-${{f.fieldname}}" placeholder="${{f.label || f.fieldname}}" ${{reqdAttr}} ${{readonlyAttr}}></textarea>`;
                }} else {{
                    inputHtml = `<input type="text" class="form-control" data-fieldname="${{f.fieldname}}" id="field-${{f.fieldname}}" placeholder="${{f.label || f.fieldname}}" ${{reqdAttr}} ${{readonlyAttr}} />`;
                }}

                html += `
                    <div class="form-group">
                        <label>${{f.label || f.fieldname}} ${{reqdStar}}</label>
                        ${{inputHtml}}
                    </div>
                `;
            }}
            grid.innerHTML = html;
        }}

        async function loadDocument() {{
            if (isNew) {{
                const dateElem = document.getElementById('field-posting_date');
                if (dateElem) dateElem.value = new Date().toISOString().split('T')[0];
                if (childItems.length === 0) {{
                    addChildTableRow({{ item_code: 'ITEM-DEFAULT', item_name: 'Standard Catalog Item', qty: 1, rate: 100.0, amount: 100.0 }});
                    recalcChildTable();
                }}
                return;
            }}
            try {{
                const res = await fetch(`/api/v2/document/${{doctype}}/${{encodeURIComponent(currentName)}}`);
                if (!res.ok) throw new Error('Status ' + res.status);
                const json = await res.json();
                const d = json.doc || json.data || {{}};
                loadedDoc = d;

                document.querySelectorAll('#doc-form [data-fieldname]').forEach(el => {{
                    const fn = el.getAttribute('data-fieldname');
                    if (d[fn] !== undefined && d[fn] !== null) {{
                        if (el.type === 'checkbox') {{
                            el.checked = Boolean(d[fn] === 1 || d[fn] === true || d[fn] === '1');
                        }} else {{
                            el.value = d[fn];
                        }}
                    }}
                }});

                if (d.posting_date && document.getElementById('field-posting_date')) {{
                    document.getElementById('field-posting_date').value = d.posting_date;
                }}
                if ((d.customer || d.supplier || d.party) && document.getElementById('field-party')) {{
                    document.getElementById('field-party').value = d.customer || d.supplier || d.party || '';
                }}
                if ((d.grand_total || d.total) && document.getElementById('field-total')) {{
                    document.getElementById('field-total').value = d.grand_total || d.total || '';
                }}
                if (d.remarks && document.getElementById('field-remarks')) {{
                    document.getElementById('field-remarks').value = d.remarks;
                }}

                // Populate child items if present
                if (d.items && Array.isArray(d.items)) {{
                    childItems = d.items;
                    renderChildTable();
                }} else if (childItems.length === 0) {{
                    addChildTableRow();
                }}

                document.getElementById('field-raw-json').value = JSON.stringify(d, null, 2);

                const badge = document.getElementById('docstatus-badge');
                if (d.docstatus === 1) {{
                    badge.className = 'badge badge-submitted';
                    badge.textContent = 'Submitted';
                    document.getElementById('btn-cancel').style.display = 'inline-flex';
                    document.getElementById('btn-save').style.display = 'none';
                }} else if (d.docstatus === 2) {{
                    badge.className = 'badge badge-cancelled';
                    badge.textContent = 'Cancelled';
                    document.getElementById('btn-amend').style.display = 'inline-flex';
                    document.getElementById('btn-save').style.display = 'none';
                }} else {{
                    badge.className = 'badge badge-draft';
                    badge.textContent = 'Draft';
                    document.getElementById('btn-submit').style.display = 'inline-flex';
                    document.getElementById('btn-delete').style.display = 'inline-flex';
                }}
            }} catch (err) {{
                showToast('Failed to load document: ' + err.message, false);
            }}
        }}

        function collectFormData() {{
            const body = Object.assign({{}}, loadedDoc, {{ doctype: doctype }});
            document.querySelectorAll('#doc-form [data-fieldname]').forEach(el => {{
                const fn = el.getAttribute('data-fieldname');
                if (fn === 'name' && isNew) return;
                if (el.type === 'checkbox') {{
                    body[fn] = el.checked ? 1 : 0;
                }} else if (el.type === 'number') {{
                    body[fn] = el.value !== '' ? (parseFloat(el.value) || 0) : 0;
                }} else {{
                    body[fn] = el.value;
                }}
            }});
            if (childItems.length > 0) {{
                body.items = childItems;
            }}
            return body;
        }}

        async function saveDocument() {{
            const rawBox = document.getElementById('raw-json-container');
            let body = {{}};
            if (rawBox.style.display !== 'none' && document.getElementById('field-raw-json').value.trim()) {{
                try {{
                    body = JSON.parse(document.getElementById('field-raw-json').value);
                }} catch (e) {{
                    showToast('Invalid JSON in Raw Payload: ' + e.message, false);
                    return;
                }}
            }} else {{
                body = collectFormData();
            }}

            try {{
                const url = isNew ? `/api/v2/document/${{doctype}}` : `/api/v2/document/${{doctype}}/${{encodeURIComponent(currentName)}}`;
                const method = isNew ? 'POST' : 'PUT';
                const res = await fetch(url, {{
                    method: method,
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify(body)
                }});
                const json = await res.json();
                if (!res.ok) throw new Error(json.message || 'Save failed');
                showToast('Document saved successfully!', true);
                setTimeout(() => {{ window.location.href = `/desk/list/${{doctype}}`; }}, 800);
            }} catch (err) {{
                showToast('Error: ' + err.message, false);
            }}
        }}

        async function submitDocument() {{
            try {{
                const res = await fetch(`/api/v2/document/${{doctype}}/${{encodeURIComponent(currentName)}}/submit`, {{ method: 'POST' }});
                if (!res.ok) throw new Error('Submit failed');
                showToast('Document submitted successfully!', true);
                setTimeout(() => {{ window.location.reload(); }}, 800);
            }} catch (err) {{
                showToast('Submit error: ' + err.message, false);
            }}
        }}

        async function cancelDocument() {{
            try {{
                const res = await fetch(`/api/v2/document/${{doctype}}/${{encodeURIComponent(currentName)}}/cancel`, {{ method: 'POST' }});
                if (!res.ok) throw new Error('Cancel failed');
                showToast('Document cancelled successfully!', true);
                setTimeout(() => {{ window.location.reload(); }}, 800);
            }} catch (err) {{
                showToast('Cancel error: ' + err.message, false);
            }}
        }}

        async function amendDocument() {{
            const cloned = Object.assign({{}}, loadedDoc);
            delete cloned.id;
            delete cloned.creation;
            cloned.name = `${{currentName}}-1`;
            cloned.docstatus = 0;
            try {{
                const res = await fetch(`/api/v2/document/${{doctype}}`, {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify(cloned)
                }});
                if (!res.ok) throw new Error('Amend failed');
                showToast('Amended draft created!', true);
                setTimeout(() => {{ window.location.href = `/desk/form/${{doctype}}/${{encodeURIComponent(cloned.name)}}`; }}, 800);
            }} catch (err) {{
                showToast('Amend error: ' + err.message, false);
            }}
        }}

        function printDocument() {{
            window.open(`/api/v2/document/${{doctype}}/${{encodeURIComponent(currentName)}}/print`, '_blank');
        }}

        function duplicateDocument() {{
            const cloned = Object.assign({{}}, loadedDoc);
            delete cloned.id;
            delete cloned.name;
            cloned.docstatus = 0;
            sessionStorage.setItem('cloned_doc', JSON.stringify(cloned));
            window.location.href = `/desk/form/${{doctype}}/new`;
        }}

        async function deleteDocument() {{
            if (!confirm('Are you sure you want to delete this document?')) return;
            try {{
                const res = await fetch(`/api/v2/document/${{doctype}}/${{encodeURIComponent(currentName)}}`, {{ method: 'DELETE' }});
                if (!res.ok) throw new Error('Delete failed');
                showToast('Document deleted!', true);
                setTimeout(() => {{ window.location.href = `/desk/list/${{doctype}}`; }}, 800);
            }} catch (err) {{
                showToast('Delete error: ' + err.message, false);
            }}
        }}

        // Keyboard Shortcuts (Ctrl+S / Cmd+S to save, Ctrl+P to print)
        document.addEventListener('keydown', (e) => {{
            if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's') {{
                e.preventDefault();
                saveDocument();
            }}
            if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'p') {{
                e.preventDefault();
                printDocument();
            }}
        }});

        initSchema();
    </script>
</body>
</html>"##
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desk_workspaces_count() {
        let ws = get_desk_workspaces();
        assert_eq!(ws.len(), 11);
        assert_eq!(ws[0].id, "accounting");
    }

    #[test]
    fn test_render_desk_shell_html() {
        let html = render_desk_shell_html("Enterprise Desk", "Administrator");
        assert!(html.contains("RustNext Desk"));
        assert!(html.contains("omnibar"));
        assert!(html.contains("Accounting & Finance"));
        assert!(html.contains("SurrealDB 3.3.0"));
        assert!(html.contains("desk-sidebar"));
        assert!(html.contains("shortcuts-bar"));
        assert!(html.contains("omni-popover"));
    }

    #[test]
    fn test_render_list_view_html() {
        let html = render_list_view_html("sales_invoice", "Administrator");
        assert!(html.contains("sales invoice List — RustNext Desk"));
        assert!(html.contains("Document Identifier"));
        assert!(html.contains("/api/v2/document/sales_invoice"));
        assert!(html.contains("status-pills-bar"));
        assert!(html.contains("bulk-action-bar"));
        assert!(html.contains("pagination-bar"));
    }

    #[test]
    fn test_render_form_view_html() {
        let html = render_form_view_html("sales_invoice", "ACC-SINV-2026-00001", "Administrator");
        assert!(html.contains("ACC-SINV-2026-00001"));
        assert!(html.contains("field-posting_date"));
        assert!(html.contains("field-total"));
        assert!(html.contains("/api/v2/document/"));
        assert!(html.contains("connections-dashboard"));
        assert!(html.contains("child-table-container"));
        assert!(html.contains("btn-print"));
    }
}
