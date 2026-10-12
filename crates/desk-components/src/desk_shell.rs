//! Unified Enterprise Desk & Multi-Persona Shell SSR Engine (`desk_components::desk_shell`).
//!
//! Renders the Awwwards-grade, zero-external-dependency administrative desktop shell,
//! module workspaces, OmniBar, quick entry drawers, and real-time live telemetry HUD.

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
<html lang="en">
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
            --font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
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
            background: rgba(10, 11, 16, 0.85); backdrop-filter: blur(16px);
            border-bottom: 1px solid var(--border);
            padding: 0.75rem 2rem; display: flex; justify-content: space-between; align-items: center;
        }
        .brand { display: flex; align-items: center; gap: 0.75rem; font-weight: 700; font-size: 1.25rem; letter-spacing: -0.02em; }
        .brand-logo { width: 32px; height: 32px; background: linear-gradient(135deg, #6366f1, #a855f7); border-radius: 8px; display: flex; align-items: center; justify-content: center; font-weight: 900; color: white; }
        
        .omnibar {
            display: flex; align-items: center; gap: 0.5rem;
            background: var(--bg-secondary); border: 1px solid var(--border); border-radius: 10px;
            padding: 0.5rem 1rem; width: 420px; transition: all 0.2s;
        }
        .omnibar:focus-within { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-glow); }
        .omnibar input { background: transparent; border: none; color: white; width: 100%; outline: none; font-size: 0.875rem; }
        .omnibar kbd { background: rgba(255, 255, 255, 0.1); border-radius: 4px; padding: 0.15rem 0.4rem; font-size: 0.75rem; color: var(--text-secondary); }
        
        .nav-actions { display: flex; align-items: center; gap: 1rem; }
        .persona-select {
            background: var(--bg-secondary); border: 1px solid var(--border); color: var(--text-primary);
            padding: 0.4rem 0.8rem; border-radius: 8px; font-size: 0.875rem; outline: none; cursor: pointer;
        }
        .user-chip { display: flex; align-items: center; gap: 0.5rem; font-size: 0.875rem; font-weight: 500; color: var(--text-secondary); }
        .user-avatar { width: 32px; height: 32px; border-radius: 50%; background: #334155; display: flex; align-items: center; justify-content: center; color: white; font-weight: 600; font-size: 0.875rem; }
        
        main { max-width: 1440px; margin: 0 auto; width: 100%; padding: 2.5rem 2rem; flex: 1; }
        .welcome-hero { margin-bottom: 2.5rem; }
        .welcome-hero h1 { font-size: 2.25rem; font-weight: 800; letter-spacing: -0.03em; margin-bottom: 0.5rem; background: linear-gradient(135deg, #ffffff, #94a3b8); -webkit-background-clip: text; -webkit-text-fill-color: transparent; }
        .welcome-hero p { color: var(--text-secondary); font-size: 1rem; }
        
        .grid {
            display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 1.5rem;
        }
        .card {
            background: var(--bg-card); border: 1px solid var(--border); border-radius: 14px;
            padding: 1.5rem; cursor: pointer; transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
            display: flex; flex-direction: column; position: relative; overflow: hidden;
        }
        .card:hover {
            transform: translateY(-4px); border-color: rgba(99, 102, 241, 0.4);
            box-shadow: 0 12px 24px -10px var(--accent-glow);
        }
        .card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; }
        .card-icon { width: 44px; height: 44px; border-radius: 10px; background: rgba(99, 102, 241, 0.12); color: #818cf8; display: flex; align-items: center; justify-content: center; }
        .badge { font-size: 0.75rem; font-weight: 600; padding: 0.25rem 0.6rem; border-radius: 9999px; background: rgba(255, 255, 255, 0.06); color: #c7d2fe; border: 1px solid rgba(255, 255, 255, 0.08); }
        .card-title { font-size: 1.15rem; font-weight: 700; margin-bottom: 0.5rem; letter-spacing: -0.01em; }
        .card-desc { font-size: 0.875rem; color: var(--text-secondary); line-height: 1.5; margin-bottom: 1.25rem; flex: 1; }
        .pills-container { display: flex; flex-wrap: wrap; gap: 0.4rem; }
        .pill { font-size: 0.75rem; padding: 0.2rem 0.5rem; border-radius: 6px; background: rgba(255, 255, 255, 0.04); color: #cbd5e1; border: 1px solid rgba(255, 255, 255, 0.04); }
        
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
        <div class="brand">
            <div class="brand-logo">R</div>
            <span>RustNext Desk</span>
        </div>
        <div class="omnibar">
            <input type="text" placeholder="Search DocTypes, records, methods, or shortcuts..." id="omni-search" />
            <kbd>Ctrl+K</kbd>
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
            <div class="user-chip">
                <div class="user-avatar">__AVATAR__</div>
                <span>__USERNAME__</span>
            </div>
        </div>
    </header>

    <main>
        <div class="welcome-hero">
            <h1>Welcome to __PERSONA__</h1>
            <p>High-performance local-first Rust ERP runtime with ACID transaction ledger and sub-millisecond hypermedia response.</p>
        </div>
        <div class="grid">
            __CARDS__
        </div>
    </main>

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

        // Real-Time OmniBar Search Filtering
        const searchInput = document.getElementById('omni-search');
        if (searchInput) {
            searchInput.addEventListener('input', (e) => {
                const q = e.target.value.toLowerCase().trim();
                document.querySelectorAll('.card').forEach(card => {
                    const text = card.textContent.toLowerCase();
                    card.style.display = text.includes(q) ? '' : 'none';
                });
            });
        }

        document.addEventListener('keydown', (e) => {
            if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
                e.preventDefault();
                if (searchInput) searchInput.focus();
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

    for ws in workspaces {
        let doctypes_pills = ws
            .doctypes
            .iter()
            .map(|d| format!("<span class=\"pill\">{d}</span>"))
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
        .replace("__CARDS__", &cards_html)
}

/// Compiles an interactive, responsive HTML List View for any DocType.
#[must_use]
pub fn render_list_view_html(doctype: &str, user_name: &str) -> String {
    let avatar_char = user_name.chars().next().unwrap_or('A').to_string();
    let display_title = doctype.replace('_', " ");
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
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
        * {{ margin: 0; padding: 0; box-sizing: border-box; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }}
        ::-webkit-scrollbar {{ width: 6px; height: 6px; }}
        ::-webkit-scrollbar-track {{ background: transparent; }}
        ::-webkit-scrollbar-thumb {{ background: rgba(255, 255, 255, 0.15); border-radius: 9999px; }}
        ::-webkit-scrollbar-thumb:hover {{ background: rgba(255, 255, 255, 0.25); }}
        .btn:active, button:active {{ transform: scale(0.98); }}
        body {{ background-color: var(--bg-primary); color: var(--text-primary); min-height: 100vh; display: flex; flex-direction: column; }}
        header {{
            position: sticky; top: 0; z-index: 50; background: rgba(10, 11, 16, 0.85); backdrop-filter: blur(16px);
            border-bottom: 1px solid var(--border); padding: 0.75rem 2rem; display: flex; justify-content: space-between; align-items: center;
        }}
        .brand {{ display: flex; align-items: center; gap: 0.75rem; font-weight: 700; font-size: 1.25rem; text-decoration: none; color: inherit; }}
        .brand-logo {{ width: 32px; height: 32px; background: linear-gradient(135deg, #6366f1, #a855f7); border-radius: 8px; display: flex; align-items: center; justify-content: center; font-weight: 900; color: white; }}
        .nav-actions {{ display: flex; align-items: center; gap: 1rem; }}
        .user-chip {{ display: flex; align-items: center; gap: 0.5rem; font-size: 0.875rem; color: var(--text-secondary); }}
        .user-avatar {{ width: 32px; height: 32px; border-radius: 50%; background: #334155; display: flex; align-items: center; justify-content: center; color: white; font-weight: 600; }}
        main {{ max-width: 1440px; margin: 0 auto; width: 100%; padding: 2rem; flex: 1; }}
        .action-bar {{ display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.5rem; flex-wrap: wrap; gap: 1rem; }}
        .breadcrumbs {{ display: flex; align-items: center; gap: 0.5rem; font-size: 0.875rem; color: var(--text-secondary); margin-bottom: 0.5rem; }}
        .breadcrumbs a {{ color: var(--accent); text-decoration: none; }}
        .page-title {{ font-size: 1.75rem; font-weight: 800; text-transform: capitalize; }}
        .btn {{ padding: 0.5rem 1.2rem; border-radius: 8px; font-weight: 600; font-size: 0.875rem; cursor: pointer; border: none; transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1); display: inline-flex; align-items: center; gap: 0.5rem; text-decoration: none; }}
        .btn-primary {{ background: var(--accent); color: white; box-shadow: 0 0 12px var(--accent-glow); }}
        .btn-primary:hover {{ opacity: 0.9; transform: translateY(-1px); }}
        .btn-secondary {{ background: var(--bg-secondary); color: var(--text-primary); border: 1px solid var(--border); }}
        .btn-secondary:hover {{ border-color: var(--accent); }}
        .table-container {{ background: var(--bg-card); border: 1px solid var(--border); border-radius: 12px; overflow: hidden; }}
        .table-filter-bar {{ padding: 1rem 1.5rem; border-bottom: 1px solid var(--border); display: flex; gap: 1rem; align-items: center; flex-wrap: wrap; }}
        .search-input {{ background: var(--bg-secondary); border: 1px solid var(--border); color: white; padding: 0.5rem 1rem; border-radius: 8px; width: 300px; outline: none; font-size: 0.875rem; }}
        .search-input:focus {{ border-color: var(--accent); }}
        table {{ width: 100%; border-collapse: collapse; text-align: left; font-size: 0.875rem; }}
        th {{ background: rgba(255, 255, 255, 0.02); padding: 0.85rem 1.5rem; color: var(--text-secondary); font-weight: 600; border-bottom: 1px solid var(--border); }}
        td {{ padding: 1rem 1.5rem; border-bottom: 1px solid var(--border); color: var(--text-primary); }}
        tr.clickable-row {{ cursor: pointer; transition: background 0.15s; }}
        tr.clickable-row:hover {{ background: rgba(99, 102, 241, 0.08); }}
        .badge {{ font-size: 0.75rem; font-weight: 600; padding: 0.2rem 0.6rem; border-radius: 9999px; }}
        .badge-draft {{ background: rgba(245, 158, 11, 0.15); color: #fbbf24; border: 1px solid rgba(245, 158, 11, 0.3); }}
        .badge-submitted {{ background: rgba(16, 185, 129, 0.15); color: #34d399; border: 1px solid rgba(16, 185, 129, 0.3); }}
        .badge-cancelled {{ background: rgba(239, 68, 68, 0.15); color: #f87171; border: 1px solid rgba(239, 68, 68, 0.3); }}
        .empty-state {{ text-align: center; padding: 4rem 2rem; color: var(--text-secondary); }}
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
                <h1 class="page-title">{display_title}</h1>
            </div>
            <div style="display: flex; gap: 0.75rem;">
                <button class="btn btn-secondary" onclick="loadRecords()"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21.5 2v6h-6M21.34 15.57a10 10 0 1 1-.57-8.38l5.67-5.67"/></svg> Refresh</button>
                <a href="/desk/form/{doctype}/new" class="btn btn-primary"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 5v14M5 12h14"/></svg> New {display_title}</a>
            </div>
        </div>
        <div class="table-container">
            <div class="table-filter-bar">
                <input type="text" id="filter-search" class="search-input" placeholder="Search by name, ID or details..." oninput="filterTable()" />
                <span id="record-count" style="font-size: 0.85rem; color: var(--text-secondary); margin-left: auto;">Loading records...</span>
            </div>
            <table>
                <thead>
                    <tr>
                        <th>Document Identifier</th>
                        <th>Status</th>
                        <th>Created</th>
                        <th>Action</th>
                    </tr>
                </thead>
                <tbody id="table-body">
                    <tr><td colspan="4" class="empty-state">Loading {display_title} records from SurrealDB...</td></tr>
                </tbody>
            </table>
        </div>
    </main>
    <div id="toast"></div>
    <script>
        let allRecords = [];
        async function loadRecords() {{
            const tbody = document.getElementById('table-body');
            const countSpan = document.getElementById('record-count');
            try {{
                const res = await fetch('/api/v2/document/{doctype}?limit=50');
                if (!res.ok) throw new Error('Status: ' + res.status);
                const json = await res.json();
                allRecords = json.data || [];
                renderRows(allRecords);
            }} catch (err) {{
                tbody.innerHTML = `<tr><td colspan="4" class="empty-state" style="color:var(--danger)">Failed to load records: ${{err.message}}</td></tr>`;
                countSpan.textContent = 'Error loading records';
            }}
        }}

        function renderRows(records) {{
            const tbody = document.getElementById('table-body');
            const countSpan = document.getElementById('record-count');
            countSpan.textContent = `${{records.length}} record(s) found`;
            if (records.length === 0) {{
                tbody.innerHTML = `<tr><td colspan="4" class="empty-state">No records found. Click "+ New {display_title}" to create one.</td></tr>`;
                return;
            }}
            tbody.innerHTML = records.map(r => {{
                const docname = r.name || r.id || 'unnamed';
                const status = r.docstatus === 1 ? 'Submitted' : (r.docstatus === 2 ? 'Cancelled' : 'Draft');
                const badgeClass = r.docstatus === 1 ? 'badge-submitted' : (r.docstatus === 2 ? 'badge-cancelled' : 'badge-draft');
                const created = r.creation || r.created_at || 'Just now';
                return `<tr class="clickable-row" onclick="window.location.href='/desk/form/{doctype}/${{encodeURIComponent(docname)}}'">
                    <td style="font-weight:600; color:var(--accent);">${{docname}}</td>
                    <td><span class="badge ${{badgeClass}}">${{status}}</span></td>
                    <td style="color:var(--text-secondary);">${{created}}</td>
                    <td><a href="/desk/form/{doctype}/${{encodeURIComponent(docname)}}" class="btn btn-secondary" style="padding:0.25rem 0.6rem;font-size:0.75rem;">Edit</a></td>
                </tr>`;
            }}).join('');
        }}

        function filterTable() {{
            const q = document.getElementById('filter-search').value.toLowerCase().trim();
            if (!q) {{ renderRows(allRecords); return; }}
            const filtered = allRecords.filter(r => JSON.stringify(r).toLowerCase().includes(q));
            renderRows(filtered);
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
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
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
        * {{ margin: 0; padding: 0; box-sizing: border-box; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }}
        ::-webkit-scrollbar {{ width: 6px; height: 6px; }}
        ::-webkit-scrollbar-track {{ background: transparent; }}
        ::-webkit-scrollbar-thumb {{ background: rgba(255, 255, 255, 0.15); border-radius: 9999px; }}
        ::-webkit-scrollbar-thumb:hover {{ background: rgba(255, 255, 255, 0.25); }}
        .btn:active, button:active {{ transform: scale(0.98); }}
        body {{ background-color: var(--bg-primary); color: var(--text-primary); min-height: 100vh; display: flex; flex-direction: column; }}
        header {{
            position: sticky; top: 0; z-index: 50; background: rgba(10, 11, 16, 0.85); backdrop-filter: blur(16px);
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
        .action-bar {{ display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.5rem; flex-wrap: wrap; gap: 1rem; }}
        .page-title {{ font-size: 1.75rem; font-weight: 800; display: flex; align-items: center; gap: 0.75rem; }}
        .btn {{ padding: 0.5rem 1.2rem; border-radius: 8px; font-weight: 600; font-size: 0.875rem; cursor: pointer; border: none; transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1); display: inline-flex; align-items: center; gap: 0.5rem; text-decoration: none; }}
        .btn-primary {{ background: var(--accent); color: white; box-shadow: 0 0 12px var(--accent-glow); }}
        .btn-primary:hover {{ opacity: 0.9; transform: translateY(-1px); }}
        .btn-success {{ background: var(--success); color: white; }}
        .btn-danger {{ background: var(--danger); color: white; }}
        .btn-secondary {{ background: var(--bg-secondary); color: var(--text-primary); border: 1px solid var(--border); }}
        .btn-secondary:hover {{ border-color: var(--accent); }}
        .form-card {{ background: var(--bg-card); border: 1px solid var(--border); border-radius: 14px; padding: 2rem; margin-bottom: 1.5rem; }}
        .form-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 1.5rem; margin-bottom: 1.5rem; }}
        .form-group {{ display: flex; flex-direction: column; gap: 0.4rem; }}
        .form-group label {{ font-size: 0.8rem; font-weight: 600; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.05em; }}
        .form-control {{ background: var(--bg-secondary); border: 1px solid var(--border); color: white; padding: 0.6rem 0.85rem; border-radius: 8px; font-size: 0.875rem; outline: none; transition: border-color 0.2s; width: 100%; }}
        .form-control:focus {{ border-color: var(--accent); }}
        textarea.form-control {{ min-height: 120px; resize: vertical; font-family: monospace; }}
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
                    <span>{docname}</span>
                    <span id="docstatus-badge" class="badge badge-draft">Draft</span>
                </h1>
            </div>
            <div style="display: flex; gap: 0.75rem;">
                <button id="btn-save" class="btn btn-primary" onclick="saveDocument()"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg> Save</button>
                <button id="btn-submit" class="btn btn-success" style="display:none;" onclick="submitDocument()"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg> Submit</button>
                <button id="btn-cancel" class="btn btn-danger" style="display:none;" onclick="cancelDocument()"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg> Cancel</button>
                <button id="btn-delete" class="btn btn-secondary" style="display:none;" onclick="deleteDocument()"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg> Delete</button>
            </div>
        </div>

        <form id="doc-form" onsubmit="event.preventDefault(); saveDocument();">
            <div class="form-card">
                <div class="form-grid">
                    <div class="form-group">
                        <label>Identifier / Name</label>
                        <input type="text" id="field-name" class="form-control" value="{docname}" disabled />
                    </div>
                    <div class="form-group">
                        <label>Posting / Transaction Date</label>
                        <input type="date" id="field-posting_date" class="form-control" />
                    </div>
                    <div class="form-group">
                        <label>Party / Reference</label>
                        <input type="text" id="field-party" class="form-control" placeholder="Customer / Supplier / Reference..." />
                    </div>
                    <div class="form-group">
                        <label>Total Amount / Value</label>
                        <input type="number" step="0.01" id="field-total" class="form-control" placeholder="0.00" />
                    </div>
                </div>
                <div class="form-group">
                    <label>Remarks / Description / Payload</label>
                    <textarea id="field-remarks" class="form-control" placeholder="Enter notes or JSON data..."></textarea>
                </div>
            </div>
        </form>
    </main>
    <div id="toast"></div>

    <script>
        const isNew = {is_new};
        const doctype = '{doctype}';
        let currentName = '{docname}';

        function showToast(msg, ok = true) {{
            const t = document.getElementById('toast');
            t.style.display = 'block';
            t.style.borderColor = ok ? 'var(--success)' : 'var(--danger)';
            t.innerHTML = `<span style="color:${{ok ? 'var(--success)' : 'var(--danger)'}}">●</span> ${{msg}}`;
            setTimeout(() => {{ t.style.display = 'none'; }}, 3500);
        }}

        async function loadDocument() {{
            if (isNew) {{
                document.getElementById('field-posting_date').value = new Date().toISOString().split('T')[0];
                return;
            }}
            try {{
                const res = await fetch(`/api/v2/document/${{doctype}}/${{encodeURIComponent(currentName)}}`);
                if (!res.ok) throw new Error('Status ' + res.status);
                const json = await res.json();
                const d = json.data || {{}};
                if (d.posting_date) document.getElementById('field-posting_date').value = d.posting_date;
                if (d.customer || d.supplier || d.party) document.getElementById('field-party').value = d.customer || d.supplier || d.party || '';
                if (d.grand_total || d.total) document.getElementById('field-total').value = d.grand_total || d.total || '';
                if (d.remarks || d.description) document.getElementById('field-remarks').value = d.remarks || d.description || JSON.stringify(d, null, 2);

                const badge = document.getElementById('docstatus-badge');
                if (d.docstatus === 1) {{
                    badge.className = 'badge badge-submitted';
                    badge.textContent = 'Submitted';
                    document.getElementById('btn-cancel').style.display = 'inline-flex';
                }} else if (d.docstatus === 2) {{
                    badge.className = 'badge badge-cancelled';
                    badge.textContent = 'Cancelled';
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

        async function saveDocument() {{
            const body = {{
                doctype: doctype,
                posting_date: document.getElementById('field-posting_date').value,
                party: document.getElementById('field-party').value,
                total: parseFloat(document.getElementById('field-total').value) || 0.0,
                remarks: document.getElementById('field-remarks').value,
            }};
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

        loadDocument();
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
    }

    #[test]
    fn test_render_list_view_html() {
        let html = render_list_view_html("sales_invoice", "Administrator");
        assert!(html.contains("sales invoice List — RustNext Desk"));
        assert!(html.contains("Document Identifier"));
        assert!(html.contains("/api/v2/document/sales_invoice"));
    }

    #[test]
    fn test_render_form_view_html() {
        let html = render_form_view_html("sales_invoice", "ACC-SINV-2026-00001", "Administrator");
        assert!(html.contains("ACC-SINV-2026-00001"));
        assert!(html.contains("field-posting_date"));
        assert!(html.contains("field-total"));
        assert!(html.contains("/api/v2/document/"));
    }
}
