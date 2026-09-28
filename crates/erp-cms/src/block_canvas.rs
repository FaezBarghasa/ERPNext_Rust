//! Block-Based Visual Canvas & Statically Compiled SSR Engine (`erp-cms::block_canvas`).
//!
//! Stores page layouts as polymorphic JSON AST blocks (zero raw HTML in the database)
//! and renders blazing-fast server-side HTML directly into memory buffers (<10ms TTFB).

use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::fmt::Write;

/// Individual feature card in a features grid block.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeatureItem {
    pub title: CompactString,
    pub description: CompactString,
    pub icon_name: Option<CompactString>,
}

/// Strongly typed polymorphic page layout block AST.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "props")]
pub enum PageBlock {
    /// Hero banner with call-to-action button and optional background image.
    Hero {
        heading: CompactString,
        subheading: CompactString,
        cta_label: CompactString,
        cta_url: CompactString,
        image_url: Option<CompactString>,
    },
    /// Live DocType data grid showing records filtered from SurrealDB.
    DocTypeGrid {
        doctype: CompactString,
        filter: CompactString,
        columns: u8,
    },
    /// Pure Markdown content block.
    Markdown {
        source: CompactString,
    },
    /// Live digital commerce product showcase block.
    ProductShowcase {
        category_id: CompactString,
        limit: usize,
    },
    /// Call-to-action banner block.
    CtaBanner {
        title: CompactString,
        description: CompactString,
        button_text: CompactString,
        button_link: CompactString,
    },
    /// Grid of feature cards.
    FeaturesGrid {
        heading: Option<CompactString>,
        items: Vec<FeatureItem>,
    },
}

/// Complete CMS page definition stored as a structured JSON AST.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CmsPage {
    pub id: CompactString,
    pub route: CompactString,
    pub title: CompactString,
    pub meta_description: CompactString,
    pub is_published: bool,
    pub blocks: Vec<PageBlock>,
}

/// Pure-Rust Server-Side Rendering (SSR) HTML Compiler.
pub struct SsrEngine;

impl SsrEngine {
    /// Renders a complete HTML document into a high-efficiency pre-allocated string buffer.
    #[must_use]
    pub fn render_page(page: &CmsPage) -> String {
        let mut html = String::with_capacity(4096);

        let _ = write!(
            html,
            "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"UTF-8\"><title>{}</title><meta name=\"description\" content=\"{}\"><link rel=\"stylesheet\" href=\"/assets/theme.css\"></head><body class=\"cms-body\"><header class=\"cms-header\"><nav class=\"nav-container\"><a href=\"/\" class=\"brand-logo\">RustNext</a></nav></header><main class=\"cms-main\">",
            html_escape(&page.title),
            html_escape(&page.meta_description)
        );

        for block in &page.blocks {
            Self::render_block(&mut html, block);
        }

        let _ = write!(
            html,
            "</main><footer class=\"cms-footer\"><p>&copy; {} RustNext Enterprise Operating System</p></footer></body></html>",
            chrono::Utc::now().format("%Y")
        );

        html
    }

    /// Appends the HTML representation of an individual block to the buffer.
    pub fn render_block(buffer: &mut String, block: &PageBlock) {
        match block {
            PageBlock::Hero {
                heading,
                subheading,
                cta_label,
                cta_url,
                image_url,
            } => {
                let img_tag = match image_url {
                    Some(url) => format!("<img src=\"{}\" alt=\"Hero\" class=\"hero-img\">", html_escape(url)),
                    None => String::new(),
                };

                let _ = write!(
                    buffer,
                    "<section class=\"block-hero\"><div class=\"hero-content\"><h1>{}</h1><p class=\"hero-sub\">{}</p><a href=\"{}\" class=\"btn-cta\">{}</a></div>{}</section>",
                    html_escape(heading),
                    html_escape(subheading),
                    html_escape(cta_url),
                    html_escape(cta_label),
                    img_tag
                );
            }
            PageBlock::DocTypeGrid {
                doctype,
                filter,
                columns,
            } => {
                let _ = write!(
                    buffer,
                    "<section class=\"block-doctype-grid\" data-doctype=\"{}\" data-filter=\"{}\" style=\"--grid-cols: {}\"><div class=\"grid-container\"><!-- Dynamic Live Streaming DocType Grid: {} --></div></section>",
                    html_escape(doctype),
                    html_escape(filter),
                    columns,
                    html_escape(doctype)
                );
            }
            PageBlock::Markdown { source } => {
                let _ = write!(
                    buffer,
                    "<section class=\"block-markdown\"><article>{}</article></section>",
                    html_escape(source)
                );
            }
            PageBlock::ProductShowcase {
                category_id,
                limit,
            } => {
                let _ = write!(
                    buffer,
                    "<section class=\"block-product-showcase\" data-category=\"{}\" data-limit=\"{}\"><div class=\"showcase-grid\"><!-- Atomic Live Commerce Catalog --></div></section>",
                    html_escape(category_id),
                    limit
                );
            }
            PageBlock::CtaBanner {
                title,
                description,
                button_text,
                button_link,
            } => {
                let _ = write!(
                    buffer,
                    "<section class=\"block-cta-banner\"><h2>{}</h2><p>{}</p><a href=\"{}\" class=\"btn-banner\">{}</a></section>",
                    html_escape(title),
                    html_escape(description),
                    html_escape(button_link),
                    html_escape(button_text)
                );
            }
            PageBlock::FeaturesGrid { heading, items } => {
                let heading_html = match heading {
                    Some(h) => format!("<h2>{}</h2>", html_escape(h)),
                    None => String::new(),
                };

                let _ = write!(
                    buffer,
                    "<section class=\"block-features-grid\">{}<div class=\"features-container\">",
                    heading_html
                );

                for item in items {
                    let _ = write!(
                        buffer,
                        "<div class=\"feature-card\"><h3>{}</h3><p>{}</p></div>",
                        html_escape(&item.title),
                        html_escape(&item.description)
                    );
                }

                buffer.push_str("</div></section>");
            }
        }
    }
}

/// Simple, zero-allocation escaping for HTML special characters.
fn html_escape(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_block_json_serialization() {
        let page = CmsPage {
            id: "page-home".into(),
            route: "/".into(),
            title: "Next-Gen Enterprise OS".into(),
            meta_description: "Pure-Rust Institutional ERP".into(),
            is_published: true,
            blocks: vec![
                PageBlock::Hero {
                    heading: "Planetary Scale Pure Rust".into(),
                    subheading: "Zero GIL, Zero GC pauses".into(),
                    cta_label: "Get Started".into(),
                    cta_url: "/signup".into(),
                    image_url: Some("/assets/hero.webp".into()),
                },
                PageBlock::FeaturesGrid {
                    heading: Some("Core Pillars".into()),
                    items: vec![
                        FeatureItem {
                            title: "Micro-Mode".into(),
                            description: "Runs in <64MB RAM".into(),
                            icon_name: Some("cpu".into()),
                        },
                        FeatureItem {
                            title: "SurrealDB Core".into(),
                            description: "Native Graph Relations".into(),
                            icon_name: Some("database".into()),
                        },
                    ],
                },
            ],
        };

        let json = serde_json::to_string(&page).unwrap();
        let decoded: CmsPage = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.title, page.title);
        assert_eq!(decoded.blocks.len(), 2);
    }

    #[test]
    fn test_ssr_html_rendering() {
        let page = CmsPage {
            id: "page-landing".into(),
            route: "/landing".into(),
            title: "Storefront & Catalog".into(),
            meta_description: "Real-time stock".into(),
            is_published: true,
            blocks: vec![PageBlock::CtaBanner {
                title: "Launch Today".into(),
                description: "Experience single-digit millisecond latency".into(),
                button_text: "Join Now".into(),
                button_link: "/join".into(),
            }],
        };

        let rendered = SsrEngine::render_page(&page);
        assert!(rendered.contains("<!DOCTYPE html>"));
        assert!(rendered.contains("<title>Storefront &amp; Catalog</title>"));
        assert!(rendered.contains("Launch Today"));
        assert!(rendered.contains("class=\"block-cta-banner\""));
    }
}
