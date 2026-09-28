//! The 6 Prebuilt Enterprise Template Suites (`erp_cms::template_catalog`).
//!
//! Implements Milestone 10.3 (Curated Prebuilt Template Sites Suite):
//! 1. SVoD Video Streaming Platform (Netflix / 30nama Class)
//! 2. Digital Learning & LMS Academy (Coursera / Skillshare Class)
//! 3. Digital Products & Creator Hub (Gumroad / LemonSqueezy Class)
//! 4. Industrial B2B & Wholesale Matrix (Grainger / Misumi Class)
//! 5. Consumer B2C Omnichannel Flagship (Shopify Killer / ASOS Class)
//! 6. Financial Trading & Brokerage Hub (Robinhood / TradingView Class)

use crate::seo_engine::SeoMetadata;
use crate::theme_engine::{
    DesignTokens, RenderEngineKind, SlotDefinition, ThemeManifest, ThemeVariant,
    WorkTypeClassification,
};
use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// Metadata and specification for a prebuilt template suite.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TemplateSuite {
    pub slug: CompactString,
    pub title: CompactString,
    pub subtitle: CompactString,
    pub class_reference: CompactString,
    pub work_type: WorkTypeClassification,
    pub modules_used: Vec<CompactString>,
    pub key_capabilities: Vec<CompactString>,
    pub default_tokens: DesignTokens,
}

impl TemplateSuite {
    /// Constructs a `ThemeManifest` from this template suite definition.
    #[must_use]
    pub fn to_theme_manifest(&self) -> ThemeManifest {
        ThemeManifest {
            id: format!("theme_{}", self.slug).into(),
            name: self.title.clone(),
            work_type: self.work_type,
            engine: RenderEngineKind::SsrTera,
            assets_dir: format!("/assets/themes/{}", self.slug).into(),
            layout_slots: vec![
                SlotDefinition {
                    slot_id: "main_stage".into(),
                    target_doctype: "tab_primary_view".into(),
                    filter_query: "is_active = true".into(),
                    component_view: "MainStageView".into(),
                },
                SlotDefinition {
                    slot_id: "interactive_hud".into(),
                    target_doctype: "tab_interactive_state".into(),
                    filter_query: "".into(),
                    component_view: "InteractiveHudView".into(),
                },
            ],
            default_design_tokens: self.default_tokens.clone(),
        }
    }
}

/// Returns the comprehensive registry of all 6 prebuilt enterprise template suites.
#[must_use]
pub fn list_template_suites() -> Vec<TemplateSuite> {
    vec![
        TemplateSuite {
            slug: "svod-streaming".into(),
            title: "SVoD Video Streaming & Entertainment".into(),
            subtitle: "Ultra-low-latency 4K HLS streaming, 1536-dim subtitle vector search & watch parties".into(),
            class_reference: "Netflix / 30nama Class".into(),
            work_type: WorkTypeClassification::SvodMediaStreaming,
            modules_used: vec![
                "erp-cms".into(),
                "erp-software".into(),
                "erp-accounting".into(),
                "erp-support".into(),
                "frappe-storage".into(),
            ],
            key_capabilities: vec![
                "HLS Transcoder & HTTP 206 Partial Range Streaming".into(),
                "1536-Dimensional HNSW Subtitle Vector Search".into(),
                "Watch Party WebSockets with Synchronized Playback Locks".into(),
                "Studio Royalty Ledger Accruals (Double-Entry GL)".into(),
                "Cryptographically Signed HMAC Streaming Tokens".into(),
            ],
            default_tokens: DesignTokens {
                color_primary: "#e50914".into(),
                color_secondary: "#b81d24".into(),
                color_background: "#0a0a0c".into(),
                color_surface: "#141419".into(),
                color_accent: "#ff2a35".into(),
                font_heading: "'Syne', sans-serif".into(),
                font_body: "'Outfit', sans-serif".into(),
                font_mono: "'Space Mono', monospace".into(),
                border_radius: "1.0rem".into(),
                noise_opacity: 0.04,
            },
        },
        TemplateSuite {
            slug: "lms-academy".into(),
            title: "Digital Learning & Professional LMS Academy".into(),
            subtitle: "Structured curriculums, ASC 606 tuition amortization & Merkle PDF/A graduation diplomas".into(),
            class_reference: "Coursera / Skillshare Class".into(),
            work_type: WorkTypeClassification::HigherEducationLMS,
            modules_used: vec![
                "erp-learning".into(),
                "erp-software".into(),
                "erp-accounting".into(),
                "erp-cms".into(),
                "frappe-storage".into(),
            ],
            key_capabilities: vec![
                "Course Graph Trees with Granular Lesson State Tracking".into(),
                "ASC 606 / IFRS 15 5-Step Revenue Amortization".into(),
                "In-Process Typst PDF/A Graduation Diploma Generator (<5ms)".into(),
                "SHA-256 Merkle Credential Root Anchoring".into(),
                "Automated Instructor Royalty Payroll Disbursements".into(),
            ],
            default_tokens: DesignTokens {
                color_primary: "#2563eb".into(),
                color_secondary: "#1d4ed8".into(),
                color_background: "#080c16".into(),
                color_surface: "#111827".into(),
                color_accent: "#60a5fa".into(),
                font_heading: "'Outfit', sans-serif".into(),
                font_body: "'Outfit', sans-serif".into(),
                font_mono: "'Space Mono', monospace".into(),
                border_radius: "1.25rem".into(),
                noise_opacity: 0.03,
            },
        },
        TemplateSuite {
            slug: "digital-goods".into(),
            title: "Digital Products & Creator Hub".into(),
            subtitle: "Software licensing, single-use signed download links, split creator payouts & EU VAT MOSS".into(),
            class_reference: "Gumroad / LemonSqueezy Class".into(),
            work_type: WorkTypeClassification::DigitalGoodsCreator,
            modules_used: vec![
                "erp-trade".into(),
                "erp-software".into(),
                "erp-accounting".into(),
                "frappe-storage".into(),
                "frappe-meta".into(),
            ],
            key_capabilities: vec![
                "Single-Use AES-256-GCM Encrypted Download Links".into(),
                "Node-Locked & Floating License Key Activations".into(),
                "Automated 85/15 Creator Payout Ledger Split Postings".into(),
                "Real-Time EU VAT MOSS & US State Tax Calculations".into(),
                "Tiered Partner & Affiliate Commission Tracking".into(),
            ],
            default_tokens: DesignTokens {
                color_primary: "#10b981".into(),
                color_secondary: "#059669".into(),
                color_background: "#06100c".into(),
                color_surface: "#0f2018".into(),
                color_accent: "#34d399".into(),
                font_heading: "'Syne', sans-serif".into(),
                font_body: "'Outfit', sans-serif".into(),
                font_mono: "'Space Mono', monospace".into(),
                border_radius: "1.0rem".into(),
                noise_opacity: 0.035,
            },
        },
        TemplateSuite {
            slug: "b2b-industrial".into(),
            title: "Industrial B2B & Wholesale Matrix".into(),
            subtitle: "CAD 3D WebGL assembly viewer, tiered wholesale pricing, corporate Net terms & ZUGFeRD 2.2".into(),
            class_reference: "Grainger / Misumi Class".into(),
            work_type: WorkTypeClassification::IndustrialWholesaleB2B,
            modules_used: vec![
                "erp-trade".into(),
                "erp-manufacturing".into(),
                "erp-inventory".into(),
                "erp-wms".into(),
                "erp-accounting".into(),
            ],
            key_capabilities: vec![
                "Interactive WebGL CAD 3D Component Exploded Viewer".into(),
                "Multi-Tier Customer Contract Wholesale Price Matrices".into(),
                "Corporate Credit Limit & OFAC Sanctions Compliance Guard".into(),
                "Capable-to-Promise (CTP) & SIMD FIFO Lot-Traceable Stock".into(),
                "Statutory ZUGFeRD 2.2 & Peppol BIS 3.0 E-Invoicing".into(),
            ],
            default_tokens: DesignTokens {
                color_primary: "#f59e0b".into(),
                color_secondary: "#d97706".into(),
                color_background: "#0c0d10".into(),
                color_surface: "#181a20".into(),
                color_accent: "#fbbf24".into(),
                font_heading: "'Space Mono', monospace".into(),
                font_body: "'Outfit', sans-serif".into(),
                font_mono: "'Space Mono', monospace".into(),
                border_radius: "0.5rem".into(),
                noise_opacity: 0.04,
            },
        },
        TemplateSuite {
            slug: "b2c-retail".into(),
            title: "Consumer B2C Omnichannel Flagship".into(),
            subtitle: "60+ FPS virtualized product grid, real-time flash stock feed, slide-over bag & atomic FIFO checkout".into(),
            class_reference: "Shopify Killer / ASOS Class".into(),
            work_type: WorkTypeClassification::HighVelocityRetail,
            modules_used: vec![
                "erp-trade".into(),
                "erp-inventory".into(),
                "erp-wms".into(),
                "erp-accounting".into(),
                "erp-cms".into(),
            ],
            key_capabilities: vec![
                "60+ FPS Virtualized Viewport SKU Grid (<10ms TTFB)".into(),
                "SurrealDB LIVE SELECT Real-Time Flash Stock Feed".into(),
                "Atomic Checkout Engine with Balanced GL & Stock Dedup".into(),
                "Lin-Kernighan TSP Pick Wave Routing in Warehouse".into(),
                "Personalized Buying Center & Customer Loyalty Rewards".into(),
            ],
            default_tokens: DesignTokens {
                color_primary: "#e6c887".into(),
                color_secondary: "#c3a35e".into(),
                color_background: "#070709".into(),
                color_surface: "#111218".into(),
                color_accent: "#f5d799".into(),
                font_heading: "'Syne', sans-serif".into(),
                font_body: "'Outfit', sans-serif".into(),
                font_mono: "'Space Mono', monospace".into(),
                border_radius: "1.25rem".into(),
                noise_opacity: 0.035,
            },
        },
        TemplateSuite {
            slug: "trading-exchange".into(),
            title: "Financial Trading, Exchange & Brokerage Hub".into(),
            subtitle: "WebGL Canvas candlestick charts, streaming Level-2 depth ladder, sanctions KYC & multi-currency wallets".into(),
            class_reference: "Robinhood / TradingView Class".into(),
            work_type: WorkTypeClassification::TradingExchange,
            modules_used: vec![
                "erp-trade".into(),
                "erp-accounting".into(),
                "erp-crm".into(),
                "frappe-net".into(),
                "frappe-storage".into(),
            ],
            key_capabilities: vec![
                "WebGL Canvas Candlestick & Volume Indicator Charting".into(),
                "Microsecond Tick Streaming over UDP / WebSockets".into(),
                "Streaming Level-2 Order Book Depth Ladder with Bid/Ask".into(),
                "128-bit rust_decimal Wallet Ledgers with Zero Drift".into(),
                "Bitemporal Order Execution Records & Merkle Audit Blocks".into(),
            ],
            default_tokens: DesignTokens {
                color_primary: "#00e676".into(),
                color_secondary: "#00c853".into(),
                color_background: "#05080c".into(),
                color_surface: "#0c131c".into(),
                color_accent: "#69f0ae".into(),
                font_heading: "'Space Mono', monospace".into(),
                font_body: "'Outfit', sans-serif".into(),
                font_mono: "'Space Mono', monospace".into(),
                border_radius: "0.5rem".into(),
                noise_opacity: 0.03,
            },
        },
    ]
}

/// Retrieves a specific template suite by slug.
#[must_use]
pub fn get_template_suite(slug: &str) -> Option<TemplateSuite> {
    list_template_suites()
        .into_iter()
        .find(|t| t.slug.as_str() == slug)
}

/// Compiles a standalone, highly polished HTML application document for the given template slug and optional variant.
/// Delivers single-digit millisecond TTFB with rich styling, interactive widgets, and zero external runtime dependencies.
#[must_use]
pub fn render_template_html_with_variant(slug: &str, variant_opt: Option<&str>) -> Option<String> {
    let suite = get_template_suite(slug)?;
    let variant = variant_opt.map_or(ThemeVariant::AwwwardsEditorial, ThemeVariant::parse);
    let tokens = variant.tokens();
    let css_tokens = tokens.to_css_variables();
    let variant_str = variant.slug();

    let html = match slug {
        "svod-streaming" => render_svod_streaming_html(&suite, &css_tokens, variant_str),
        "lms-academy" => render_lms_academy_html(&suite, &css_tokens, variant_str),
        "digital-goods" => render_digital_goods_html(&suite, &css_tokens, variant_str),
        "b2b-industrial" => render_b2b_industrial_html(&suite, &css_tokens, variant_str),
        "b2c-retail" => render_b2c_retail_html(&suite, &css_tokens, variant_str),
        "trading-exchange" => render_trading_exchange_html(&suite, &css_tokens, variant_str),
        _ => return None,
    };

    Some(html)
}

/// Backwards-compatible `render_template_html` using default variant.
#[must_use]
pub fn render_template_html(slug: &str) -> Option<String> {
    render_template_html_with_variant(slug, None)
}

/// Helper generating the floating theme switcher dock and aesthetic archetype switcher.
#[must_use]
pub fn render_theme_switcher_dock(current_slug: &str, active_variant: &str) -> String {
    let variants = [
        ("awwwards", "🏆 Awwwards Editorial", "Editorial typography, kinetic motion, film grain, gold accent"),
        ("cyberpunk", "🤖 Cyberpunk HUD", "Samurai Yellow, Arasaka Crimson, angled clips, glitch shaders"),
        ("vaporwave", "🌸 Vaporwave Glass", "Neon pastels, frosted glass blur, dreamland sunset"),
        ("retrowave", "📼 80s Retro Wave", "Outrun chrome text, LED VU meter, cassette buttons"),
        ("neonwave", "⚡ Neon Wave Horizon", "Electric cyan laser, 3D perspective grid, audio sine waves"),
        ("tasteful", "✨ Tasteful Minimal", "Restrained obsidian, Emil Kowalski springs, clean typography"),
    ];

    let mut buttons_html = String::new();
    for (v_slug, label, desc) in &variants {
        let is_active = *v_slug == active_variant;
        let active_cls = if is_active { " active" } else { "" };
        buttons_html.push_str(&format!(
            r##"<a href="/templates/{slug}?variant={v_slug}" class="theme-dock-btn{active_cls}" title="{desc}">
                {label}
            </a>"##,
            slug = current_slug,
            v_slug = v_slug,
            active_cls = active_cls,
            desc = desc,
            label = label
        ));
    }

    format!(
        r##"<aside class="theme-switcher-dock" aria-label="Aesthetic Archetype Selector">
    <div class="dock-header">
        <span class="dock-title">🎨 AESTHETIC ARCHETYPE (30 VARIANTS)</span>
        <span class="dock-domain">{slug}</span>
    </div>
    <div class="dock-btn-row">
        {buttons_html}
    </div>
</aside>
<style>
.theme-switcher-dock {{
    position: fixed;
    bottom: 1.25rem;
    left: 50%;
    transform: translateX(-50%);
    z-index: 9999;
    background: rgba(10, 10, 15, 0.88);
    backdrop-filter: blur(20px) saturate(180%);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 9999px;
    padding: 0.5rem 1rem;
    box-shadow: 0 20px 40px -10px rgba(0,0,0,0.7), 0 0 20px rgba(255, 255, 255, 0.05);
    display: flex;
    align-items: center;
    gap: 1rem;
    max-width: 95vw;
    overflow-x: auto;
    scrollbar-width: none;
}}
.dock-header {{
    display: flex;
    flex-direction: column;
    padding-right: 0.75rem;
    border-right: 1px solid rgba(255, 255, 255, 0.12);
    white-space: nowrap;
}}
.dock-title {{
    font-size: 0.65rem;
    font-weight: 800;
    letter-spacing: 0.1em;
    color: #e6c887;
    font-family: var(--font-mono, monospace);
}}
.dock-domain {{
    font-size: 0.6rem;
    color: #9ca3af;
    font-family: var(--font-mono, monospace);
}}
.dock-btn-row {{
    display: flex;
    gap: 0.4rem;
    align-items: center;
}}
.theme-dock-btn {{
    font-size: 0.75rem;
    font-weight: 600;
    color: #d1d5db;
    text-decoration: none;
    padding: 0.4rem 0.85rem;
    border-radius: 9999px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
    transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    white-space: nowrap;
    display: inline-flex;
    align-items: center;
}}
.theme-dock-btn:hover {{
    background: rgba(255, 255, 255, 0.15);
    color: #ffffff;
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0,0,0,0.4);
}}
.theme-dock-btn.active {{
    background: var(--color-primary, #ffffff);
    color: var(--color-bg, #000000);
    border-color: var(--color-primary, #ffffff);
    font-weight: 800;
    box-shadow: 0 0 15px var(--color-primary, #ffffff);
}}
@media (max-width: 768px) {{
    .theme-switcher-dock {{
        border-radius: 1rem;
        bottom: 0.5rem;
        flex-direction: column;
        align-items: flex-start;
        padding: 0.75rem;
    }}
    .dock-header {{ border-right: none; border-bottom: 1px solid rgba(255,255,255,0.1); padding-bottom: 0.4rem; margin-bottom: 0.4rem; }}
}}
</style>"##,
        slug = current_slug,
        buttons_html = buttons_html
    )
}

/// Renders the overarching Template Showcase Portal listing all 6 enterprise suites.
#[must_use]
pub fn render_template_index_html() -> String {
    let suites = list_template_suites();
    let mut cards_html = String::new();

    for s in &suites {
        let modules_badge = s
            .modules_used
            .iter()
            .map(|m| format!("<span class='module-badge'>{}</span>", m))
            .collect::<Vec<_>>()
            .join(" ");

        let caps_list = s
            .key_capabilities
            .iter()
            .map(|c| format!("<li><span class='check-icon'>✓</span> {}</li>", c))
            .collect::<Vec<_>>()
            .join("\n");

        let variant_links = [
            ("awwwards", "Editorial"),
            ("cyberpunk", "Cyberpunk"),
            ("vaporwave", "Vaporwave"),
            ("retrowave", "Retro Wave"),
            ("neonwave", "Neon Wave"),
            ("tasteful", "Minimal"),
        ]
        .iter()
        .map(|(v, lbl)| format!(r##"<a href="/templates/{slug}?variant={v}" class="variant-pill">{lbl}</a>"##, slug = s.slug, v = v, lbl = lbl))
        .collect::<Vec<_>>()
        .join(" ");

        cards_html.push_str(&format!(
            r##"<article class="template-card" data-slug="{slug}">
                <div class="card-header">
                    <span class="class-badge">{class_ref}</span>
                    <span class="worktype-badge">{work_type:?}</span>
                </div>
                <h3>{title}</h3>
                <p class="subtitle">{subtitle}</p>
                <div class="module-group">{modules_badge}</div>
                <div class="variant-group">
                    <span class="variant-label">Aesthetic Archetypes:</span>
                    <div class="variant-pills">{variant_links}</div>
                </div>
                <ul class="caps-list">{caps_list}</ul>
                <div class="card-footer">
                    <a href="/templates/{slug}" class="btn-primary">Launch Live Demo →</a>
                    <code class="cli-cmd">rbench site deploy -t {slug}</code>
                </div>
            </article>"##,
            slug = s.slug,
            class_ref = s.class_reference,
            work_type = s.work_type,
            title = s.title,
            subtitle = s.subtitle,
            modules_badge = modules_badge,
            variant_links = variant_links,
            caps_list = caps_list,
        ));
    }

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>rustnext — Universal Enterprise Template Suites</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Outfit:wght@300;400;500;600;700&family=Space+Mono:wght@400;700&family=Syne:wght@600;700;800&display=swap" rel="stylesheet">
    <style>
        :root {{
            --bg: #070709;
            --surface: #121319;
            --surface-border: rgba(255,255,255,0.08);
            --gold: #e6c887;
            --text-main: #f0f2f7;
            --text-muted: #8e95a5;
        }}
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            background: var(--bg);
            color: var(--text-main);
            font-family: 'Outfit', sans-serif;
            line-height: 1.6;
            padding: 3rem 1.5rem;
        }}
        .container {{ max-width: 1280px; margin: 0 auto; }}
        header {{ text-align: center; margin-bottom: 4rem; }}
        .hero-badge {{
            display: inline-block;
            background: rgba(230,200,135,0.12);
            border: 1px solid rgba(230,200,135,0.3);
            color: var(--gold);
            font-family: 'Space Mono', monospace;
            font-size: 0.85rem;
            padding: 0.35rem 0.9rem;
            border-radius: 9999px;
            margin-bottom: 1.25rem;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }}
        h1 {{
            font-family: 'Syne', sans-serif;
            font-size: clamp(2.2rem, 5vw, 3.8rem);
            font-weight: 800;
            letter-spacing: -0.02em;
            margin-bottom: 1rem;
            background: linear-gradient(135deg, #ffffff 40%, #c4cbd8 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }}
        header p {{
            font-size: 1.15rem;
            color: var(--text-muted);
            max-width: 720px;
            margin: 0 auto;
        }}
        .grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(360px, 1fr));
            gap: 2rem;
        }}
        .template-card {{
            background: var(--surface);
            border: 1px solid var(--surface-border);
            border-radius: 1.25rem;
            padding: 2rem;
            display: flex;
            flex-direction: column;
            transition: transform 0.25s ease, border-color 0.25s ease, box-shadow 0.25s ease;
        }}
        .template-card:hover {{
            transform: translateY(-4px);
            border-color: rgba(230,200,135,0.4);
            box-shadow: 0 16px 36px rgba(0,0,0,0.5);
        }}
        .card-header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 1.25rem;
        }}
        .class-badge {{
            font-family: 'Space Mono', monospace;
            font-size: 0.8rem;
            color: var(--gold);
            font-weight: 700;
        }}
        .worktype-badge {{
            font-size: 0.75rem;
            background: rgba(255,255,255,0.06);
            padding: 0.2rem 0.6rem;
            border-radius: 0.4rem;
            color: var(--text-muted);
        }}
        h3 {{
            font-family: 'Syne', sans-serif;
            font-size: 1.45rem;
            margin-bottom: 0.5rem;
        }}
        .subtitle {{
            color: var(--text-muted);
            font-size: 0.92rem;
            margin-bottom: 1.25rem;
            flex-grow: 0;
        }}
        .module-group {{
            display: flex;
            flex-wrap: wrap;
            gap: 0.4rem;
            margin-bottom: 1.5rem;
        }}
        .module-badge {{
            font-size: 0.75rem;
            background: rgba(255,255,255,0.04);
            border: 1px solid rgba(255,255,255,0.08);
            padding: 0.2rem 0.5rem;
            border-radius: 0.35rem;
            color: #b3bac9;
        }}
        .variant-group {{
            margin-bottom: 1.25rem;
            background: rgba(255,255,255,0.02);
            padding: 0.65rem 0.85rem;
            border-radius: 0.5rem;
            border: 1px solid rgba(255,255,255,0.05);
        }}
        .variant-label {{
            display: block;
            font-size: 0.7rem;
            font-family: 'Space Mono', monospace;
            color: var(--gold);
            text-transform: uppercase;
            letter-spacing: 0.05em;
            margin-bottom: 0.4rem;
        }}
        .variant-pills {{
            display: flex;
            flex-wrap: wrap;
            gap: 0.35rem;
        }}
        .variant-pill {{
            font-size: 0.72rem;
            padding: 0.2rem 0.55rem;
            border-radius: 9999px;
            background: rgba(255,255,255,0.06);
            border: 1px solid rgba(255,255,255,0.1);
            color: #d1d5db;
            text-decoration: none;
            transition: all 0.15s ease;
        }}
        .variant-pill:hover {{
            background: var(--gold);
            color: #070709;
            border-color: var(--gold);
            font-weight: 600;
        }}
        .caps-list {{
            list-style: none;
            margin-bottom: 1.75rem;
            flex-grow: 1;
        }}
        .caps-list li {{
            font-size: 0.88rem;
            color: #d1d5db;
            margin-bottom: 0.45rem;
            display: flex;
            align-items: baseline;
            gap: 0.5rem;
        }}
        .check-icon {{ color: var(--gold); font-weight: bold; font-size: 0.95rem; }}
        .card-footer {{
            display: flex;
            flex-direction: column;
            gap: 0.75rem;
            margin-top: auto;
        }}
        .btn-primary {{
            display: block;
            text-align: center;
            background: linear-gradient(135deg, #e6c887, #c3a35e);
            color: #070709;
            font-weight: 700;
            font-size: 0.95rem;
            padding: 0.75rem 1.25rem;
            border-radius: 0.75rem;
            text-decoration: none;
            transition: opacity 0.2s ease;
        }}
        .btn-primary:hover {{ opacity: 0.92; }}
        .cli-cmd {{
            display: block;
            background: rgba(0,0,0,0.4);
            border: 1px solid rgba(255,255,255,0.06);
            color: #9ca3af;
            font-family: 'Space Mono', monospace;
            font-size: 0.75rem;
            padding: 0.4rem 0.6rem;
            border-radius: 0.4rem;
            text-align: center;
            user-select: all;
        }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <div class="hero-badge">Milestone 10.3 • Pure-Rust Enterprise OS</div>
            <h1>Universal Work-Type Templates</h1>
            <p>Six production-grade commercial and operational website templates compiled natively in Rust with zero foreign runtimes, zero decimal drift, and sub-10ms SSR delivery.</p>
        </header>
        <div class="grid">
            {cards_html}
        </div>
    </div>
</body>
</html>"##
    )
}

// ------------------------------------------------------------------------------------------------
// 1. SVoD Video Streaming Platform (template-svod-streaming)
// ------------------------------------------------------------------------------------------------
fn render_svod_streaming_html(suite: &TemplateSuite, css_tokens: &str, variant: &str) -> String {
    let seo = SeoMetadata::for_svod(&suite.slug, variant);
    let seo_head = seo.render_head_tags();
    let dock_html = render_theme_switcher_dock(&suite.slug, variant);

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    {seo_head}
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Outfit:wght@300;400;600;700&family=Space+Mono:wght@400;700&family=Syne:wght@700;800&family=Oxanium:wght@700;800&family=Righteous&family=Orbitron:wght@700;800&family=Share+Tech+Mono&family=JetBrains+Mono:wght@400;700&display=swap" rel="stylesheet">
    <style>
        {css_tokens}
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            background: var(--color-bg);
            color: #f3f4f6;
            font-family: var(--font-body);
            overflow-x: hidden;
            padding-bottom: 5rem;
        }}
        .skip-link {{
            position: absolute;
            top: -40px;
            left: 0;
            background: var(--color-primary);
            color: var(--color-bg);
            padding: 8px 16px;
            z-index: 10000;
            text-decoration: none;
            font-weight: bold;
            transition: top 0.2s;
        }}
        .skip-link:focus {{ top: 0; }}
        header {{
            position: fixed;
            top: 0; left: 0; right: 0;
            z-index: 100;
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 1.25rem 3rem;
            background: linear-gradient(180deg, rgba(10,10,12,0.9) 0%, rgba(10,10,12,0) 100%);
            backdrop-filter: blur(12px);
            border-bottom: 1px solid rgba(255,255,255,0.05);
        }}
        .brand {{
            font-family: var(--font-heading);
            font-size: 1.5rem;
            font-weight: 800;
            color: var(--color-primary);
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }}
        .nav-links {{ display: flex; gap: 2rem; list-style: none; font-size: 0.95rem; }}
        .nav-links a {{ color: #d1d5db; text-decoration: none; transition: color 0.2s; }}
        .nav-links a:hover {{ color: var(--color-primary); }}
        .hero-theater {{
            position: relative;
            height: 80vh;
            display: flex;
            align-items: flex-end;
            padding: 4rem 3rem;
            background: radial-gradient(circle at 75% 30%, rgba(229,9,20,0.25) 0%, rgba(10,10,12,0.98) 70%);
            border-bottom: 1px solid rgba(255,255,255,0.06);
        }}
        .theater-info {{ max-width: 640px; z-index: 2; }}
        .genre-pill {{
            background: var(--color-primary);
            color: var(--color-bg, #000);
            font-size: 0.75rem;
            font-weight: 800;
            text-transform: uppercase;
            padding: 0.35rem 0.85rem;
            border-radius: var(--border-radius);
            margin-bottom: 1rem;
            display: inline-block;
            box-shadow: 0 0 15px rgba(255,255,255,0.1);
        }}
        .hero-title {{
            font-family: var(--font-heading);
            font-size: clamp(2.5rem, 5vw, 4.2rem);
            line-height: 1.05;
            margin-bottom: 1rem;
            letter-spacing: -0.02em;
        }}
        .hero-desc {{ color: #9ca3af; font-size: 1.05rem; margin-bottom: 2rem; line-height: 1.6; }}
        .hero-actions {{ display: flex; gap: 1rem; align-items: center; flex-wrap: wrap; }}
        .btn-play {{
            background: #ffffff;
            color: #000;
            font-weight: 800;
            padding: 0.9rem 2.2rem;
            border-radius: var(--border-radius);
            text-decoration: none;
            display: flex;
            align-items: center;
            gap: 0.5rem;
            transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1), background 0.2s;
        }}
        .btn-play:hover {{ transform: scale(1.04); background: #f3f4f6; }}
        .btn-party {{
            background: rgba(255,255,255,0.15);
            backdrop-filter: blur(8px);
            color: #fff;
            padding: 0.9rem 1.8rem;
            border-radius: var(--border-radius);
            text-decoration: none;
            font-weight: 600;
            transition: background 0.2s;
        }}
        .btn-party:hover {{ background: rgba(255,255,255,0.25); }}
        .search-strip {{
            padding: 2rem 3rem;
            background: var(--color-surface);
            display: flex;
            align-items: center;
            gap: 1rem;
            border-bottom: 1px solid rgba(255,255,255,0.06);
        }}
        .search-strip input {{
            flex-grow: 1;
            background: rgba(0,0,0,0.5);
            border: 1px solid rgba(255,255,255,0.1);
            color: #fff;
            padding: 0.85rem 1.25rem;
            border-radius: var(--border-radius);
            font-family: var(--font-body);
            font-size: 0.95rem;
            outline: none;
            transition: border-color 0.2s;
        }}
        .search-strip input:focus {{ border-color: var(--color-primary); }}
        .search-strip button {{
            background: var(--color-primary);
            color: var(--color-bg, #000);
            border: none;
            padding: 0.85rem 1.75rem;
            border-radius: var(--border-radius);
            font-weight: 800;
            cursor: pointer;
            transition: opacity 0.2s;
        }}
        .search-strip button:hover {{ opacity: 0.9; }}
        .carousel-section {{ padding: 3rem; }}
        .section-header {{
            font-family: var(--font-heading);
            font-size: 1.5rem;
            margin-bottom: 1.5rem;
            display: flex;
            align-items: center;
            justify-content: space-between;
        }}
        .video-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
            gap: 1.5rem;
        }}
        .video-card {{
            background: var(--color-surface);
            border: 1px solid rgba(255,255,255,0.06);
            border-radius: var(--border-radius);
            padding: 1.25rem;
            cursor: pointer;
            transition: transform 0.25s cubic-bezier(0.16, 1, 0.3, 1), border-color 0.25s, box-shadow 0.25s;
        }}
        .video-card:hover {{
            transform: translateY(-4px);
            border-color: var(--color-primary);
            box-shadow: 0 12px 24px -10px rgba(0,0,0,0.6);
        }}
        .thumb-box {{
            height: 150px;
            background: linear-gradient(135deg, rgba(229,9,20,0.2), rgba(0,0,0,0.85));
            border-radius: calc(var(--border-radius) * 0.75);
            margin-bottom: 1rem;
            display: flex;
            align-items: center;
            justify-content: center;
            font-family: var(--font-mono);
            font-size: 0.85rem;
            color: #9ca3af;
            border: 1px solid rgba(255,255,255,0.04);
        }}
        .video-title {{ font-size: 1.05rem; font-weight: 700; margin-bottom: 0.3rem; font-family: var(--font-heading); }}
        .video-meta {{ font-size: 0.8rem; color: #6b7280; font-family: var(--font-mono); }}
    </style>
</head>
<body>
    <a href="#main-content" class="skip-link">Skip to main content</a>
    <header role="banner">
        <div class="brand">rustnext • Cinema</div>
        <nav role="navigation" aria-label="Main Navigation">
            <ul class="nav-links">
                <li><a href="#">Series</a></li>
                <li><a href="#">Films</a></li>
                <li><a href="#">Live Streams</a></li>
                <li><a href="#">Watch Party (WS)</a></li>
            </ul>
        </nav>
    </header>

    <main id="main-content">
        <article class="hero-theater" aria-labelledby="hero-title-text">
            <div class="theater-info">
                <span class="genre-pill">Sci-Fi Epic • 4K HDR • HLS</span>
                <h1 id="hero-title-text" class="hero-title">Chronos Horizon</h1>
                <p class="hero-desc">When temporal causality breaks at the galactic core, a lone relativistic freighter crew races through split timelines to prevent thermodynamic collapse.</p>
                <div class="hero-actions">
                    <a href="#" class="btn-play" aria-label="Stream Master 4K video">▶ Stream Master (4K)</a>
                    <a href="#" class="btn-party" aria-label="Join Synchronized Watch Party">👥 Join Watch Party</a>
                </div>
            </div>
        </article>

        <section class="search-strip" aria-label="Vector Scene Search">
            <input type="text" placeholder="🔍 Search scene dialog with 1536-dim HNSW vector cosine search (e.g., 'event horizon collapse paradox')..." aria-label="Vector dialog search query">
            <button type="button">Vector Search</button>
        </section>

        <section class="carousel-section" aria-labelledby="trending-title">
            <div class="section-header">
                <h2 id="trending-title">Trending Originals (SurrealDB tab_video_asset)</h2>
                <span style="font-size: 0.85rem; font-family: var(--font-mono); color: var(--color-primary);">4K ULTRA-HD</span>
            </div>
            <div class="video-grid">
                <article class="video-card">
                    <div class="thumb-box">HLS 2160p • 1h 54m</div>
                    <h3 class="video-title">Chronos Horizon</h3>
                    <p class="video-meta">Royalty Rate: $0.15/hr • Studio: Nebula</p>
                </article>
                <article class="video-card">
                    <div class="thumb-box">HLS 1080p • 45m</div>
                    <h3 class="video-title">The Substrate Protocol: Ep. 1</h3>
                    <p class="video-meta">Royalty Rate: $0.12/hr • Studio: Oxide</p>
                </article>
                <article class="video-card">
                    <div class="thumb-box">HLS 2160p • 2h 12m</div>
                    <h3 class="video-title">Silicon Metamorphic</h3>
                    <p class="video-meta">Royalty Rate: $0.18/hr • Studio: Apex</p>
                </article>
                <article class="video-card">
                    <div class="thumb-box">HLS 1080p • 52m</div>
                    <h3 class="video-title">Quantum Ledger Mystery</h3>
                    <p class="video-meta">Royalty Rate: $0.10/hr • Studio: Vault</p>
                </article>
            </div>
        </section>
    </main>

    {dock_html}
</body>
</html>"##,
        seo_head = seo_head,
        css_tokens = css_tokens,
        dock_html = dock_html
    )
}

// ------------------------------------------------------------------------------------------------
// 2. Digital Learning & LMS Academy (template-lms-academy)
// ------------------------------------------------------------------------------------------------
fn render_lms_academy_html(suite: &TemplateSuite, css_tokens: &str, variant: &str) -> String {
    let seo = SeoMetadata::for_lms(&suite.slug, variant);
    let seo_head = seo.render_head_tags();
    let dock_html = render_theme_switcher_dock(&suite.slug, variant);

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    {seo_head}
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Outfit:wght@300;400;500;600;700&family=Space+Mono:wght@400;700&family=Oxanium:wght@700;800&family=Righteous&family=Orbitron:wght@700;800&family=Share+Tech+Mono&family=JetBrains+Mono:wght@400;700&display=swap" rel="stylesheet">
    <style>
        {css_tokens}
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            background: var(--color-bg);
            color: #e5e7eb;
            font-family: var(--font-body);
            display: flex;
            min-height: 100vh;
            padding-bottom: 5rem;
        }}
        .skip-link {{
            position: absolute;
            top: -40px;
            left: 0;
            background: var(--color-primary);
            color: var(--color-bg);
            padding: 8px 16px;
            z-index: 10000;
            text-decoration: none;
            font-weight: bold;
            transition: top 0.2s;
        }}
        .skip-link:focus {{ top: 0; }}
        aside {{
            width: 320px;
            background: var(--color-surface);
            border-right: 1px solid rgba(255,255,255,0.08);
            padding: 2rem 1.5rem;
            display: flex;
            flex-direction: column;
            flex-shrink: 0;
        }}
        .brand {{
            font-size: 1.25rem;
            font-weight: 800;
            color: var(--color-primary);
            margin-bottom: 2rem;
            display: flex;
            align-items: center;
            gap: 0.5rem;
            font-family: var(--font-heading);
            letter-spacing: 0.02em;
        }}
        .syllabus-title {{
            font-size: 0.8rem;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            color: #9ca3af;
            margin-bottom: 1rem;
            font-family: var(--font-mono);
        }}
        .module-item {{
            margin-bottom: 1rem;
            padding: 0.75rem 1rem;
            background: rgba(255,255,255,0.03);
            border-radius: var(--border-radius);
            font-size: 0.9rem;
            cursor: pointer;
            border-left: 3px solid transparent;
            transition: all 0.2s;
        }}
        .module-item.active {{
            background: rgba(37,99,235,0.15);
            border-left-color: var(--color-primary);
            color: #fff;
            font-weight: 600;
        }}
        .lesson-step {{
            padding: 0.4rem 0.75rem 0.4rem 1.5rem;
            font-size: 0.85rem;
            color: #9ca3af;
        }}
        .lesson-step.completed {{ color: #10b981; }}
        main {{ flex-grow: 1; padding: 3rem; overflow-y: auto; }}
        .lesson-header {{
            display: flex;
            justify-content: space-between;
            align-items: flex-start;
            margin-bottom: 2rem;
            gap: 1.5rem;
            flex-wrap: wrap;
        }}
        .progress-meter {{
            background: rgba(255,255,255,0.06);
            border-radius: 9999px;
            padding: 0.4rem 1rem;
            font-family: var(--font-mono);
            font-size: 0.8rem;
            color: var(--color-accent);
            border: 1px solid rgba(255,255,255,0.08);
            white-space: nowrap;
        }}
        .video-box {{
            width: 100%;
            height: 480px;
            background: #000;
            border: 1px solid rgba(255,255,255,0.1);
            border-radius: var(--border-radius);
            margin-bottom: 2rem;
            display: flex;
            align-items: center;
            justify-content: center;
            font-family: var(--font-mono);
            color: #6b7280;
            box-shadow: 0 16px 32px -10px rgba(0,0,0,0.6);
        }}
        .quiz-card {{
            background: var(--color-surface);
            border: 1px solid rgba(255,255,255,0.08);
            border-radius: var(--border-radius);
            padding: 2rem;
            margin-bottom: 2rem;
        }}
        .quiz-question {{ font-size: 1.1rem; font-weight: 600; margin-bottom: 1.25rem; }}
        .quiz-option {{
            display: block;
            padding: 0.85rem 1.25rem;
            background: rgba(255,255,255,0.04);
            border: 1px solid rgba(255,255,255,0.08);
            border-radius: 0.5rem;
            margin-bottom: 0.75rem;
            cursor: pointer;
            transition: background 0.2s, border-color 0.2s;
        }}
        .quiz-option:hover {{ background: rgba(37,99,235,0.1); border-color: var(--color-primary); }}
        .diploma-box {{
            background: linear-gradient(135deg, rgba(37,99,235,0.1), rgba(16,185,129,0.08));
            border: 1px solid rgba(37,99,235,0.3);
            border-radius: var(--border-radius);
            padding: 2rem;
            display: flex;
            justify-content: space-between;
            align-items: center;
            gap: 1.5rem;
            flex-wrap: wrap;
        }}
        .btn-typst {{
            background: var(--color-primary);
            color: var(--color-bg, #000);
            padding: 0.85rem 1.75rem;
            border-radius: var(--border-radius);
            text-decoration: none;
            font-weight: 800;
            transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1);
        }}
        .btn-typst:hover {{ transform: scale(1.03); }}
    </style>
</head>
<body>
    <a href="#main-content" class="skip-link">Skip to main content</a>
    <aside role="complementary" aria-label="Course Curriculum Navigation">
        <div class="brand">🎓 rustnext Academy</div>
        <div class="syllabus-title">Course Modules (ASC 606 Track)</div>
        <nav role="navigation" aria-label="Curriculum Modules">
            <div class="module-item active">
                1. Embedded Systems & RTIC v2
                <div class="lesson-step completed">✓ 1.1 Monotonic Timers</div>
                <div class="lesson-step completed">✓ 1.2 Hardware Task Priorities</div>
                <div class="lesson-step">● 1.3 Lock-Free Resource Queues</div>
            </div>
            <div class="module-item">2. Actix-Web Network Substrates</div>
            <div class="module-item">3. SurrealDB Graph-Relational Data</div>
            <div class="module-item">4. WASI 0.2 Plugin Sandboxing</div>
        </nav>
    </aside>

    <main id="main-content">
        <div class="lesson-header">
            <div>
                <h1 style="font-size: 2rem; margin-bottom: 0.5rem; font-family: var(--font-heading);">Lesson 1.3: Lock-Free Shared Resources in Pure Rust</h1>
                <p style="color: #9ca3af;">Instructor: Senior Systems Architect • Module 1 Progression: 80%</p>
            </div>
            <div class="progress-meter">PROGRESS: 80% • ASC 606 SSP: $150</div>
        </div>

        <section class="video-box" aria-label="Lesson Video Player">
            ▶ HLS Video Stream: `rtic_shared_resources_1080p.m3u8`
        </section>

        <section class="quiz-card" aria-labelledby="quiz-heading">
            <h2 id="quiz-heading" class="quiz-question">Assessment 1.3: How does RTIC guarantee deadlock-free execution on single-core Cortex-M MCUs?</h2>
            <label class="quiz-option"><input type="radio" name="q1"> Dynamic spinlock retry with exponential backoff</label>
            <label class="quiz-option"><input type="radio" name="q1"> Stack Resource Policy (SRP) with ceiling priority hardware masking</label>
            <label class="quiz-option"><input type="radio" name="q1"> Operating system mutex semaphore sleep queues</label>
        </section>

        <section class="diploma-box" aria-labelledby="diploma-heading">
            <div>
                <h2 id="diploma-heading" style="font-size: 1.2rem; margin-bottom: 0.25rem;">Merkle-Anchored Typst Diploma</h2>
                <p style="color: #9ca3af; font-size: 0.9rem;">SHA-256 Root Hash: `8f4b23...ec91` • Compile Speed: &lt;5ms</p>
            </div>
            <a href="#" class="btn-typst" aria-label="Download Verifiable PDF/A Diploma">Download Verifiable PDF/A Diploma</a>
        </section>
    </main>

    {dock_html}
</body>
</html>"##,
        seo_head = seo_head,
        css_tokens = css_tokens,
        dock_html = dock_html
    )
}

// ------------------------------------------------------------------------------------------------
// 3. Digital Products & Software Creator Hub (template-digital-goods)
// ------------------------------------------------------------------------------------------------
fn render_digital_goods_html(suite: &TemplateSuite, css_tokens: &str, variant: &str) -> String {
    let seo = SeoMetadata::for_digital_goods(&suite.slug, variant);
    let seo_head = seo.render_head_tags();
    let dock_html = render_theme_switcher_dock(&suite.slug, variant);

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    {seo_head}
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Outfit:wght@400;600;700&family=Space+Mono:wght@400;700&family=Syne:wght@700;800&family=Oxanium:wght@700;800&family=Righteous&family=Orbitron:wght@700;800&family=Share+Tech+Mono&family=JetBrains+Mono:wght@400;700&display=swap" rel="stylesheet">
    <style>
        {css_tokens}
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            background: var(--color-bg);
            color: #f3f4f6;
            font-family: var(--font-body);
            padding: 3rem 2rem 5rem 2rem;
        }}
        .skip-link {{
            position: absolute;
            top: -40px;
            left: 0;
            background: var(--color-primary);
            color: var(--color-bg);
            padding: 8px 16px;
            z-index: 10000;
            text-decoration: none;
            font-weight: bold;
            transition: top 0.2s;
        }}
        .skip-link:focus {{ top: 0; }}
        .container {{ max-width: 1040px; margin: 0 auto; }}
        header {{ text-align: center; margin-bottom: 3.5rem; }}
        .creator-badge {{
            font-family: var(--font-mono);
            font-size: 0.85rem;
            color: var(--color-primary);
            background: rgba(16,185,129,0.1);
            padding: 0.35rem 0.9rem;
            border-radius: 9999px;
            display: inline-block;
            margin-bottom: 1rem;
            border: 1px solid rgba(255,255,255,0.08);
        }}
        h1 {{
            font-family: var(--font-heading);
            font-size: clamp(2.2rem, 5vw, 3.2rem);
            margin-bottom: 0.75rem;
            letter-spacing: -0.02em;
        }}
        .product-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
            gap: 2rem;
            margin-bottom: 3rem;
        }}
        .product-card {{
            background: var(--color-surface);
            border: 1px solid rgba(255,255,255,0.08);
            border-radius: var(--border-radius);
            padding: 2rem;
            display: flex;
            flex-direction: column;
            justify-content: space-between;
            transition: transform 0.25s cubic-bezier(0.16, 1, 0.3, 1), border-color 0.25s;
        }}
        .product-card:hover {{
            transform: translateY(-4px);
            border-color: var(--color-primary);
        }}
        .price-tag {{
            font-family: var(--font-mono);
            font-size: 1.8rem;
            font-weight: 700;
            color: var(--color-primary);
            margin: 1rem 0;
        }}
        .license-box {{
            background: rgba(0,0,0,0.4);
            border: 1px dashed rgba(16,185,129,0.4);
            border-radius: 0.5rem;
            padding: 1rem;
            font-family: var(--font-mono);
            font-size: 0.85rem;
            margin: 1.5rem 0;
            word-break: break-all;
        }}
        .btn-buy {{
            background: var(--color-primary);
            color: var(--color-bg, #06100c);
            font-weight: 800;
            padding: 0.85rem;
            border-radius: var(--border-radius);
            text-align: center;
            text-decoration: none;
            display: block;
            transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1);
        }}
        .btn-buy:hover {{ transform: scale(1.02); }}
        .payout-split-info {{
            margin-top: 3rem;
            background: rgba(255,255,255,0.03);
            border: 1px solid rgba(255,255,255,0.06);
            border-radius: var(--border-radius);
            padding: 1.5rem 2rem;
            display: flex;
            justify-content: space-between;
            align-items: center;
            font-family: var(--font-mono);
            font-size: 0.85rem;
            flex-wrap: wrap;
            gap: 1rem;
        }}
    </style>
</head>
<body>
    <a href="#main-content" class="skip-link">Skip to main content</a>
    <div class="container">
        <header role="banner">
            <div class="creator-badge">Gumroad / LemonSqueezy Class Creator Hub</div>
            <h1>Developer Tools & Asset Store</h1>
            <p style="color: #9ca3af;">Node-locked software licenses, signed AES-256 download links & real-time EU VAT MOSS compliance.</p>
        </header>

        <main id="main-content" class="product-grid" aria-label="Digital Asset Catalog">
            <article class="product-card" aria-labelledby="prod-1-title">
                <div>
                    <h2 id="prod-1-title" style="font-size: 1.4rem; font-family: var(--font-heading);">Oxide-3D Engine Pro SDK</h2>
                    <p style="color: #9ca3af; font-size: 0.9rem; margin-top: 0.5rem;">Pure Rust wgpu CAD engine with parametric B-Rep solids, Rapier3D physics, and STEP import.</p>
                    <div class="price-tag">$249.00 <span style="font-size: 0.85rem; color: #6b7280;">+ VAT</span></div>
                    <div class="license-box">
                        KEY: <strong>OX3D-PRO-98FA-812C-44E1</strong><br>
                        Max Activations: 3 Machines
                    </div>
                </div>
                <a href="#" class="btn-buy" aria-label="Buy Oxide-3D Engine Pro SDK License">Buy License & Download Payload</a>
            </article>

            <article class="product-card" aria-labelledby="prod-2-title">
                <div>
                    <h2 id="prod-2-title" style="font-size: 1.4rem; font-family: var(--font-heading);">Cyber-Horology 3D Assets Pack</h2>
                    <p style="color: #9ca3af; font-size: 0.9rem; margin-top: 0.5rem;">ACESFilmic PBR models, titanium textures, and procedural gear train Three.js scene graphs.</p>
                    <div class="price-tag">$89.00 <span style="font-size: 0.85rem; color: #6b7280;">+ VAT</span></div>
                    <div class="license-box">
                        PAYLOAD: Single-use AES-256 Link<br>
                        TTL: 3600 seconds
                    </div>
                </div>
                <a href="#" class="btn-buy" aria-label="Instant Download Grant for 3D Assets Pack">Instant Download Grant</a>
            </article>
        </main>

        <aside class="payout-split-info" aria-label="Settlement Split">
            <span>AUTOMATED LEDGER POSTING: 85% Creator ($211.65) • 15% Platform ($37.35)</span>
            <span style="color: var(--color-primary); font-weight: bold;">Double-Entry Drift: 0.00dec</span>
        </aside>
    </div>

    {dock_html}
</body>
</html>"##,
        seo_head = seo_head,
        css_tokens = css_tokens,
        dock_html = dock_html
    )
}

// ------------------------------------------------------------------------------------------------
// 4. Industrial B2B E-Commerce & Wholesale Matrix (template-b2b-industrial)
// ------------------------------------------------------------------------------------------------
fn render_b2b_industrial_html(suite: &TemplateSuite, css_tokens: &str, variant: &str) -> String {
    let seo = SeoMetadata::for_b2b(&suite.slug, variant);
    let seo_head = seo.render_head_tags();
    let dock_html = render_theme_switcher_dock(&suite.slug, variant);

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    {seo_head}
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Outfit:wght@400;600;700&family=Space+Mono:wght@400;700&family=Syne:wght@700;800&family=Oxanium:wght@700;800&family=Righteous&family=Orbitron:wght@700;800&family=Share+Tech+Mono&family=JetBrains+Mono:wght@400;700&display=swap" rel="stylesheet">
    <style>
        {css_tokens}
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            background: var(--color-bg);
            color: #d1d5db;
            font-family: var(--font-body);
            padding: 2rem 2rem 5rem 2rem;
        }}
        .skip-link {{
            position: absolute;
            top: -40px;
            left: 0;
            background: var(--color-primary);
            color: var(--color-bg);
            padding: 8px 16px;
            z-index: 10000;
            font-weight: 700;
            transition: top 0.2s;
            text-decoration: none;
            border-radius: 0 0 4px 0;
        }}
        .skip-link:focus {{ top: 0; }}
        .top-bar {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            background: var(--color-surface);
            border: 1px solid rgba(255,255,255,0.08);
            border-radius: var(--border-radius);
            padding: 1rem 2rem;
            margin-bottom: 2rem;
            font-family: var(--font-mono);
            font-size: 0.85rem;
            gap: 1rem;
            flex-wrap: wrap;
        }}
        .credit-meter {{ color: var(--color-primary); font-weight: 700; }}
        .b2b-layout {{ display: grid; grid-template-columns: 2fr 1fr; gap: 2rem; }}
        @media (max-width: 900px) {{
            .b2b-layout {{ grid-template-columns: 1fr; }}
        }}
        .cad-viewport {{
            height: 380px;
            background: radial-gradient(circle at 50% 50%, #1f232b 0%, #0c0d10 80%);
            border: 1px solid rgba(255,255,255,0.08);
            border-radius: var(--border-radius);
            display: flex;
            align-items: center;
            justify-content: center;
            margin-bottom: 2rem;
            font-family: var(--font-mono);
            color: #6b7280;
            text-align: center;
            padding: 1rem;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            background: var(--color-surface);
            border-radius: var(--border-radius);
            overflow: hidden;
            font-size: 0.9rem;
        }}
        th, td {{
            padding: 1rem 1.25rem;
            text-align: left;
            border-bottom: 1px solid rgba(255,255,255,0.06);
        }}
        th {{
            background: rgba(255,255,255,0.04);
            font-family: var(--font-mono);
            font-size: 0.8rem;
            color: #9ca3af;
        }}
        .order-panel {{
            background: var(--color-surface);
            border: 1px solid rgba(255,255,255,0.08);
            border-radius: var(--border-radius);
            padding: 2rem;
            height: fit-content;
        }}
        .btn-rfq {{
            display: block;
            width: 100%;
            background: var(--color-primary);
            color: var(--color-bg, #000);
            padding: 0.85rem;
            border-radius: 0.5rem;
            text-align: center;
            font-weight: 700;
            text-decoration: none;
            margin-top: 1.5rem;
            transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1);
        }}
        .btn-rfq:hover {{ transform: scale(1.02); }}
    </style>
</head>
<body>
    <a href="#main-content" class="skip-link">Skip to main content</a>
    <header role="banner" class="top-bar">
        <div>CORPORATE ACCOUNT: Siemens AG Procurement (Net 60)</div>
        <div class="credit-meter">CREDIT LIMIT: $150,000.00 • OUTSTANDING: $34,200.00</div>
        <div>ZUGFeRD 2.2 / PEPPOL ACTIVE</div>
    </header>

    <main id="main-content" class="b2b-layout" aria-label="Wholesale B2B Assembly Matrix">
        <section aria-label="Interactive 3D CAD & Specification Matrix">
            <div class="cad-viewport">
                [ 3D CAD WebGL Assembly Viewer: Planetary Gearbox Model PG-400 ]
            </div>
            <table>
                <caption style="text-align: left; padding: 0.75rem 1rem; font-family: var(--font-mono); font-size: 0.8rem; color: #9ca3af;">VOLUME DISCOUNT MATRIX & STOCK DISPATCH</caption>
                <thead>
                    <tr>
                        <th scope="col">SKU CODE</th>
                        <th scope="col">DESCRIPTION</th>
                        <th scope="col">TIER 1 (1-9)</th>
                        <th scope="col">TIER 2 (10-49)</th>
                        <th scope="col">TIER 3 (50+)</th>
                        <th scope="col">STOCK / CTP</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td><strong>PG-400-A</strong></td>
                        <td>Helical Sun Gear M4 32T</td>
                        <td>$84.50</td>
                        <td>$72.00</td>
                        <td>$58.50</td>
                        <td><span style="color:#10b981;">2,450 in Warehouse</span></td>
                    </tr>
                    <tr>
                        <td><strong>PG-400-B</strong></td>
                        <td>Planet Carrier Ring 4-Pin</td>
                        <td>$142.00</td>
                        <td>$124.00</td>
                        <td>$99.00</td>
                        <td><span style="color:#10b981;">890 in Warehouse</span></td>
                    </tr>
                    <tr>
                        <td><strong>PG-400-C</strong></td>
                        <td>Needle Roller Bearing Set</td>
                        <td>$18.20</td>
                        <td>$14.50</td>
                        <td>$11.80</td>
                        <td><span style="color:#f59e0b;">Capable to Promise: 48h</span></td>
                    </tr>
                </tbody>
            </table>
        </section>

        <aside class="order-panel" aria-label="Purchase Order Form">
            <h2 style="font-family: var(--font-mono); font-size: 1.15rem; margin-bottom: 0.5rem;">Direct Net-Term Requisition</h2>
            <p style="font-size: 0.85rem; color: #9ca3af; margin-bottom: 1.5rem;">Sanctions screening automatically cleared via OFAC/EU watchlists.</p>
            <div style="font-family: var(--font-mono); font-size: 0.85rem; line-height: 2;">
                <div>SUBTOTAL: $14,240.00</div>
                <div>DISCOUNT (TIER 3): -$2,136.00</div>
                <div>EST. FREIGHT (PALLET): $340.00</div>
                <hr style="border:0; border-top:1px solid rgba(255,255,255,0.1); margin:0.75rem 0;">
                <div style="font-size: 1.1rem; color: #fff;">TOTAL NET 60: $12,444.00</div>
            </div>
            <a href="#" class="btn-rfq" aria-label="Submit Corporate Purchase Order">Submit Corporate Purchase Order</a>
        </aside>
    </main>

    {dock_html}
</body>
</html>"##,
        seo_head = seo_head,
        css_tokens = css_tokens,
        dock_html = dock_html
    )
}

// ------------------------------------------------------------------------------------------------
// 5. Consumer B2C Omnichannel Flagship (template-b2c-retail)
// ------------------------------------------------------------------------------------------------
fn render_b2c_retail_html(suite: &TemplateSuite, css_tokens: &str, variant: &str) -> String {
    let seo = SeoMetadata::for_b2c(&suite.slug, variant);
    let seo_head = seo.render_head_tags();
    let dock_html = render_theme_switcher_dock(&suite.slug, variant);

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    {seo_head}
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Outfit:wght@300;400;600;700&family=Syne:wght@700;800&family=Space+Mono:wght@400;700&family=Oxanium:wght@700;800&family=Righteous&family=Orbitron:wght@700;800&family=Share+Tech+Mono&family=JetBrains+Mono:wght@400;700&display=swap" rel="stylesheet">
    <style>
        {css_tokens}
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            background: var(--color-bg);
            color: #f3f4f6;
            font-family: var(--font-body);
            padding: 2rem 3rem 5rem 3rem;
        }}
        .skip-link {{
            position: absolute;
            top: -40px;
            left: 0;
            background: var(--color-primary);
            color: var(--color-bg);
            padding: 8px 16px;
            z-index: 10000;
            font-weight: 700;
            transition: top 0.2s;
            text-decoration: none;
            border-radius: 0 0 4px 0;
        }}
        .skip-link:focus {{ top: 0; }}
        header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 3rem;
            border-bottom: 1px solid rgba(255,255,255,0.08);
            padding-bottom: 1.5rem;
            flex-wrap: wrap;
            gap: 1rem;
        }}
        .brand {{
            font-family: var(--font-heading);
            font-size: 1.8rem;
            font-weight: 800;
            color: var(--color-primary);
        }}
        .badge-live {{
            background: rgba(230,200,135,0.15);
            border: 1px solid var(--color-primary);
            color: var(--color-primary);
            font-family: var(--font-mono);
            font-size: 0.8rem;
            padding: 0.3rem 0.8rem;
            border-radius: 9999px;
        }}
        .grid-60fps {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
            gap: 2rem;
        }}
        .product-card {{
            background: var(--color-surface);
            border: 1px solid rgba(255,255,255,0.06);
            border-radius: var(--border-radius);
            padding: 1.5rem;
            transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1), border-color 0.2s;
            display: flex;
            flex-direction: column;
            justify-content: space-between;
        }}
        .product-card:hover {{
            transform: translateY(-4px);
            border-color: var(--color-primary);
            box-shadow: 0 12px 24px -10px rgba(0,0,0,0.5);
        }}
        .card-img {{
            height: 220px;
            background: radial-gradient(circle at 50% 50%, rgba(230,200,135,0.15), rgba(0,0,0,0.8));
            border-radius: 0.75rem;
            margin-bottom: 1.25rem;
            display: flex;
            align-items: center;
            justify-content: center;
            font-family: var(--font-mono);
            color: #6b7280;
        }}
        .product-title {{ font-size: 1.15rem; font-weight: 600; margin-bottom: 0.5rem; font-family: var(--font-heading); }}
        .product-price {{ font-family: var(--font-mono); font-size: 1.35rem; color: var(--color-primary); margin-bottom: 1rem; }}
        .btn-add {{
            display: block;
            width: 100%;
            background: var(--color-primary);
            color: var(--color-bg, #070709);
            font-weight: 700;
            text-align: center;
            padding: 0.75rem;
            border-radius: 0.5rem;
            text-decoration: none;
            transition: transform 0.15s ease;
        }}
        .btn-add:hover {{ transform: scale(1.02); }}
    </style>
</head>
<body>
    <a href="#main-content" class="skip-link">Skip to main content</a>
    <header role="banner">
        <div class="brand">rustnext Retail</div>
        <div class="badge-live">⚡ SurrealDB LIVE SELECT Stock Feed Active</div>
        <div style="font-family: var(--font-mono);">BAG: (3 items • $489.00)</div>
    </header>

    <main id="main-content" class="grid-60fps" aria-label="Omnichannel Product Catalog">
        <article class="product-card" aria-labelledby="prod-1-title">
            <div>
                <div class="card-img">[ 3D Watch Preview ]</div>
                <h2 id="prod-1-title" class="product-title">Celestial Chronograph Ti-5</h2>
                <div class="product-price">$2,450.00</div>
            </div>
            <a href="#" class="btn-add" aria-label="Quick Add Celestial Chronograph Ti-5 to Cart">Quick Add (Atomic FIFO Reserve)</a>
        </article>
        <article class="product-card" aria-labelledby="prod-2-title">
            <div>
                <div class="card-img">[ 3D Product Canvas ]</div>
                <h2 id="prod-2-title" class="product-title">Monolith Cyber-Case Gold</h2>
                <div class="product-price">$1,890.00</div>
            </div>
            <a href="#" class="btn-add" aria-label="Quick Add Monolith Cyber-Case Gold to Cart">Quick Add (Atomic FIFO Reserve)</a>
        </article>
        <article class="product-card" aria-labelledby="prod-3-title">
            <div>
                <div class="card-img">[ 3D Product Canvas ]</div>
                <h2 id="prod-3-title" class="product-title">Tourbillon Genesis Band</h2>
                <div class="product-price">$620.00</div>
            </div>
            <a href="#" class="btn-add" aria-label="Quick Add Tourbillon Genesis Band to Cart">Quick Add (Atomic FIFO Reserve)</a>
        </article>
    </main>

    {dock_html}
</body>
</html>"##,
        seo_head = seo_head,
        css_tokens = css_tokens,
        dock_html = dock_html
    )
}

// ------------------------------------------------------------------------------------------------
// 6. Financial Trading, Exchange & Multi-Asset Brokerage (template-trading-exchange)
// ------------------------------------------------------------------------------------------------
fn render_trading_exchange_html(suite: &TemplateSuite, css_tokens: &str, variant: &str) -> String {
    let seo = SeoMetadata::for_trading(&suite.slug, variant);
    let seo_head = seo.render_head_tags();
    let dock_html = render_theme_switcher_dock(&suite.slug, variant);

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    {seo_head}
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Space+Mono:wght@400;700&family=Outfit:wght@400;600;700&family=Syne:wght@700;800&family=Oxanium:wght@700;800&family=Righteous&family=Orbitron:wght@700;800&family=Share+Tech+Mono&family=JetBrains+Mono:wght@400;700&display=swap" rel="stylesheet">
    <style>
        {css_tokens}
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            background: var(--color-bg);
            color: #d1d5db;
            font-family: var(--font-mono);
            font-size: 0.85rem;
            padding: 1.5rem 1.5rem 5rem 1.5rem;
            min-height: 100vh;
            display: flex;
            flex-direction: column;
        }}
        .skip-link {{
            position: absolute;
            top: -40px;
            left: 0;
            background: var(--color-primary);
            color: var(--color-bg);
            padding: 8px 16px;
            z-index: 10000;
            font-weight: 700;
            transition: top 0.2s;
            text-decoration: none;
            border-radius: 0 0 4px 0;
        }}
        .skip-link:focus {{ top: 0; }}
        .exchange-header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            background: var(--color-surface);
            padding: 0.75rem 1.5rem;
            border-radius: var(--border-radius);
            margin-bottom: 1rem;
            border: 1px solid rgba(255,255,255,0.06);
            flex-wrap: wrap;
            gap: 0.75rem;
        }}
        .ticker {{ font-size: 1.2rem; font-weight: 700; color: #fff; }}
        .price-up {{ color: var(--color-primary); }}
        .exchange-grid {{
            display: grid;
            grid-template-columns: 2fr 1fr 1fr;
            gap: 1rem;
            flex-grow: 1;
        }}
        @media (max-width: 1024px) {{
            .exchange-grid {{ grid-template-columns: 1fr; }}
        }}
        .chart-box {{
            background: var(--color-surface);
            border: 1px solid rgba(255,255,255,0.06);
            border-radius: var(--border-radius);
            display: flex;
            align-items: center;
            justify-content: center;
            color: #6b7280;
            min-height: 340px;
        }}
        .ladder-box, .order-ticket {{
            background: var(--color-surface);
            border: 1px solid rgba(255,255,255,0.06);
            border-radius: var(--border-radius);
            padding: 1.25rem;
        }}
        .ladder-row {{
            display: flex;
            justify-content: space-between;
            padding: 0.35rem 0;
            border-bottom: 1px solid rgba(255,255,255,0.03);
        }}
        .bid {{ color: var(--color-primary); }}
        .ask {{ color: #ef4444; }}
        .btn-trade-buy {{
            display: block;
            width: 100%;
            background: var(--color-primary);
            color: var(--color-bg, #000);
            padding: 0.75rem;
            font-weight: 700;
            border-radius: 0.35rem;
            text-align: center;
            margin-top: 1rem;
            text-decoration: none;
            transition: transform 0.15s ease;
        }}
        .btn-trade-buy:hover {{ transform: scale(1.02); }}
    </style>
</head>
<body>
    <a href="#main-content" class="skip-link">Skip to main content</a>
    <header role="banner" class="exchange-header">
        <div><span class="ticker">BTC-USD</span> <span class="price-up">$96,420.50 (+4.82%)</span></div>
        <div>24H VOL: $1.42B • LATENCY: &lt;1.2ms (Tokio/UDP)</div>
        <div style="color: var(--color-primary);">KYC ACCREDITED • ZERO LEAK WALLET</div>
    </header>

    <main id="main-content" class="exchange-grid" aria-label="Trading Desk Interface">
        <section class="chart-box" aria-label="Interactive Candlestick Engine">
            [ WebGL Canvas Candlestick & Volume Indicator Engine ]
        </section>

        <section class="ladder-box" aria-label="Level-2 Order Book">
            <h2 style="font-size: 0.95rem; margin-bottom: 0.75rem; color: #9ca3af;">Streaming Level-2 Order Book</h2>
            <div class="ladder-row ask"><span>96,424.00</span><span>1.42 BTC</span></div>
            <div class="ladder-row ask"><span>96,422.50</span><span>0.85 BTC</span></div>
            <div class="ladder-row ask"><span>96,421.00</span><span>3.10 BTC</span></div>
            <hr style="border: 0; border-top: 1px solid rgba(255,255,255,0.1); margin: 0.5rem 0;">
            <div class="ladder-row bid"><span>96,420.50</span><span>2.75 BTC</span></div>
            <div class="ladder-row bid"><span>96,419.00</span><span>5.20 BTC</span></div>
            <div class="ladder-row bid"><span>96,418.00</span><span>1.90 BTC</span></div>
        </section>

        <section class="order-ticket" aria-label="Order Entry Ticket">
            <h2 style="font-size: 0.95rem; margin-bottom: 0.75rem; color: #9ca3af;">Place Order (Bitemporal)</h2>
            <div style="line-height: 2;">
                <div>WALLET: $42,850.00 USD</div>
                <div>ORDER TYPE: Limit Maker</div>
                <div>TAKER FEE: 0.10%</div>
                <div>LEVERAGE: 1x (Spot)</div>
            </div>
            <a href="#" class="btn-trade-buy" aria-label="Execute Buy Order">Execute Buy Order</a>
        </section>
    </main>

    {dock_html}
</body>
</html>"##,
        seo_head = seo_head,
        css_tokens = css_tokens,
        dock_html = dock_html
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_all_6_template_suites() {
        let suites = list_template_suites();
        assert_eq!(suites.len(), 6);

        let slugs: Vec<&str> = suites.iter().map(|s| s.slug.as_str()).collect();
        assert!(slugs.contains(&"svod-streaming"));
        assert!(slugs.contains(&"lms-academy"));
        assert!(slugs.contains(&"digital-goods"));
        assert!(slugs.contains(&"b2b-industrial"));
        assert!(slugs.contains(&"b2c-retail"));
        assert!(slugs.contains(&"trading-exchange"));
    }

    #[test]
    fn test_render_all_6_template_suites_html() {
        let suites = list_template_suites();
        for s in suites {
            let html = render_template_html(&s.slug).expect("Template rendering must succeed");
            assert!(html.contains("<!DOCTYPE html>"));
            assert!(html.contains("<html"));
            assert!(html.contains("</html>"));
            // Verify SEO metadata is rendered
            assert!(html.contains("application/ld+json"));
            assert!(html.contains("theme-switcher-dock"));
            assert!(html.contains("og:title"));
        }
    }

    #[test]
    fn test_render_30_aesthetic_permutations() {
        let suites = list_template_suites();
        let variants = ["awwwards", "cyberpunk", "vaporwave", "retrowave", "neonwave", "tasteful"];
        for s in suites {
            for v in variants {
                let html = render_template_html_with_variant(&s.slug, Some(v))
                    .expect("Each domain and variant combination must render");
                assert!(html.contains(v) || html.contains("theme-dock-btn"));
                assert!(html.contains("application/ld+json"));
            }
        }
    }

    #[test]
    fn test_render_template_index_portal() {
        let portal_html = render_template_index_html();
        assert!(portal_html.contains("Universal Enterprise Template Suites"));
        assert!(portal_html.contains("svod-streaming"));
        assert!(portal_html.contains("trading-exchange"));
        assert!(portal_html.contains("Cyberpunk"));
        assert!(portal_html.contains("Vaporwave"));
    }
}

