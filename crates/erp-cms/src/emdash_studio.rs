//! Emdash Editorial Publishing Studio & Media Processing Pipeline (`erp-cms::emdash_studio`).
//!
//! Provides WordPress and Emdash parity:
//! - Block-based rich text content modeling (Heading, Paragraph, Code, Quote, Callout, Table, Media)
//! - Editorial publication workflow state transitions with validation
//! - Media library smart focal-point cropping and responsive WebP variant generation
//! - Automated XML sitemap generation and OpenGraph social metadata integration.

use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::media_library::MediaError;
use crate::seo_engine::SeoMetadata;

/// Styling categories for Callout blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalloutStyle {
    Info,
    Success,
    Warning,
    Danger,
    Note,
}

impl CalloutStyle {
    #[must_use]
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Info => "callout-info",
            Self::Success => "callout-success",
            Self::Warning => "callout-warning",
            Self::Danger => "callout-danger",
            Self::Note => "callout-note",
        }
    }
}

/// Rich text editorial block AST.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "block_type")]
pub enum EmdashBlock {
    Heading {
        level: u8,
        text: CompactString,
    },
    Paragraph {
        text: CompactString,
    },
    Code {
        language: CompactString,
        code: CompactString,
    },
    Quote {
        quote: CompactString,
        citation: Option<CompactString>,
    },
    Callout {
        style: CalloutStyle,
        title: Option<CompactString>,
        text: CompactString,
    },
    Table {
        headers: Vec<CompactString>,
        rows: Vec<Vec<CompactString>>,
    },
    Media {
        asset_id: CompactString,
        url: CompactString,
        caption: Option<CompactString>,
        alt: CompactString,
    },
}

impl EmdashBlock {
    #[must_use]
    pub fn render_html(&self) -> String {
        match self {
            Self::Heading { level, text } => {
                let lvl = (*level).clamp(1, 6);
                format!("<h{lvl}>{text}</h{lvl}>\n")
            }
            Self::Paragraph { text } => {
                format!("<p>{text}</p>\n")
            }
            Self::Code { language, code } => {
                format!(
                    "<pre><code class=\"language-{language}\">{}</code></pre>\n",
                    html_escape(code)
                )
            }
            Self::Quote { quote, citation } => {
                let cite = citation
                    .as_ref()
                    .map(|c| format!("<footer>— <cite>{c}</cite></footer>"))
                    .unwrap_or_default();
                format!("<blockquote><p>{quote}</p>{cite}</blockquote>\n")
            }
            Self::Callout { style, title, text } => {
                let title_tag = title
                    .as_ref()
                    .map(|t| format!("<h4>{t}</h4>"))
                    .unwrap_or_default();
                format!(
                    "<aside class=\"callout {}\">{title_tag}<p>{text}</p></aside>\n",
                    style.css_class()
                )
            }
            Self::Table { headers, rows } => {
                let mut out = String::from("<table>\n<thead>\n<tr>\n");
                for h in headers {
                    out.push_str(&format!("<th>{h}</th>"));
                }
                out.push_str("\n</tr>\n</thead>\n<tbody>\n");
                for row in rows {
                    out.push_str("<tr>\n");
                    for cell in row {
                        out.push_str(&format!("<td>{cell}</td>"));
                    }
                    out.push_str("\n</tr>\n");
                }
                out.push_str("</tbody>\n</table>\n");
                out
            }
            Self::Media {
                url, caption, alt, ..
            } => {
                let cap = caption
                    .as_ref()
                    .map(|c| format!("<figcaption>{c}</figcaption>"))
                    .unwrap_or_default();
                format!(
                    "<figure><img src=\"{url}\" alt=\"{alt}\" loading=\"lazy\" />{cap}</figure>\n"
                )
            }
        }
    }

    #[must_use]
    pub fn word_count(&self) -> usize {
        match self {
            Self::Heading { text, .. } | Self::Paragraph { text } => {
                text.split_whitespace().count()
            }
            Self::Quote { quote, .. } => quote.split_whitespace().count(),
            Self::Callout { text, .. } => text.split_whitespace().count(),
            Self::Table { headers, rows } => {
                let h_count: usize = headers.iter().map(|h| h.split_whitespace().count()).sum();
                let r_count: usize = rows
                    .iter()
                    .flat_map(|r| r.iter())
                    .map(|c| c.split_whitespace().count())
                    .sum();
                h_count + r_count
            }
            Self::Code { code, .. } => code.split_whitespace().count(),
            Self::Media { caption, alt, .. } => {
                let cap_cnt = caption
                    .as_ref()
                    .map(|c| c.split_whitespace().count())
                    .unwrap_or(0);
                cap_cnt + alt.split_whitespace().count()
            }
        }
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Errors occurring during editorial state transitions.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum EditorialWorkflowError {
    #[error("Invalid workflow transition from '{from}' to '{to}'")]
    InvalidTransition {
        from: &'static str,
        to: &'static str,
    },
    #[error("Missing required metadata for publishing: {0}")]
    MissingPublishingMetadata(&'static str),
}

/// Workflow states of an Emdash editorial document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorialWorkflowState {
    Draft,
    InReview,
    Scheduled { publish_timestamp: u64 },
    Published { published_timestamp: u64 },
    Archived,
}

impl EditorialWorkflowState {
    #[must_use]
    pub fn state_name(&self) -> &'static str {
        match self {
            Self::Draft => "Draft",
            Self::InReview => "InReview",
            Self::Scheduled { .. } => "Scheduled",
            Self::Published { .. } => "Published",
            Self::Archived => "Archived",
        }
    }

    /// Validates and executes state transitions.
    pub fn transition_to(
        &self,
        target: EditorialWorkflowState,
    ) -> Result<EditorialWorkflowState, EditorialWorkflowError> {
        let is_valid = match (self, &target) {
            // Draft can move to InReview or Archived
            (Self::Draft, Self::InReview | Self::Archived) => true,

            // InReview can move back to Draft, advance to Scheduled/Published, or Archived
            (
                Self::InReview,
                Self::Draft | Self::Scheduled { .. } | Self::Published { .. } | Self::Archived,
            ) => true,

            // Scheduled can be rescheduled, published, reverted to InReview, or archived
            (
                Self::Scheduled { .. },
                Self::InReview | Self::Scheduled { .. } | Self::Published { .. } | Self::Archived,
            ) => true,

            // Published can be archived or reverted to Draft (unpublish)
            (Self::Published { .. }, Self::Archived | Self::Draft) => true,

            // Archived can be restored to Draft
            (Self::Archived, Self::Draft) => true,

            _ => false,
        };

        if is_valid {
            Ok(target)
        } else {
            Err(EditorialWorkflowError::InvalidTransition {
                from: self.state_name(),
                to: target.state_name(),
            })
        }
    }
}

/// Editorial article / post entity in Emdash CMS.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmdashDocument {
    pub id: CompactString,
    pub title: CompactString,
    pub slug: CompactString,
    pub author: CompactString,
    pub blocks: Vec<EmdashBlock>,
    pub workflow_state: EditorialWorkflowState,
    pub tags: Vec<CompactString>,
    pub seo: SeoMetadata,
    #[serde(default)]
    pub version: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

impl EmdashDocument {
    #[must_use]
    pub fn new(
        id: impl Into<CompactString>,
        title: impl Into<CompactString>,
        slug: impl Into<CompactString>,
        author: impl Into<CompactString>,
    ) -> Self {
        let title_str = title.into();
        let slug_str = slug.into();
        let canonical = format!("https://rustnext.enterprise.io/editorial/{slug_str}");
        let seo = SeoMetadata::new_website(title_str.as_str(), title_str.as_str(), &canonical);

        Self {
            id: id.into(),
            title: title_str,
            slug: slug_str,
            author: author.into(),
            blocks: Vec::new(),
            workflow_state: EditorialWorkflowState::Draft,
            tags: Vec::new(),
            seo,
            version: 1,
            created_at: 0,
            updated_at: 0,
        }
    }

    pub fn add_block(&mut self, block: EmdashBlock) {
        self.blocks.push(block);
    }

    #[must_use]
    pub fn word_count(&self) -> usize {
        self.blocks.iter().map(EmdashBlock::word_count).sum()
    }

    /// Average adult reading speed: 200 words per minute.
    #[must_use]
    pub fn reading_time_minutes(&self) -> u32 {
        let words = self.word_count();
        ((words as f32 / 200.0).ceil() as u32).max(1)
    }

    #[must_use]
    pub fn render_html(&self) -> String {
        let mut out = format!(
            "<article class=\"emdash-article\" id=\"{}\">\n<header>\n<h1>{}</h1>\n<div class=\"meta\">By <span class=\"author\">{}</span> • <span class=\"read-time\">{} min read</span></div>\n</header>\n<div class=\"article-body\">\n",
            self.id,
            self.title,
            self.author,
            self.reading_time_minutes()
        );

        for block in &self.blocks {
            out.push_str(&block.render_html());
        }

        out.push_str("</div>\n</article>\n");
        out
    }
}

/// Calculated pixel bounding box for image cropping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CropBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Computes an optimal crop bounding box honoring normalized focal point (0.0..1.0).
pub fn calculate_focal_crop(
    image_width: u32,
    image_height: u32,
    target_width: u32,
    target_height: u32,
    focal_point: (f32, f32),
) -> Result<CropBox, MediaError> {
    if !(0.0..=1.0).contains(&focal_point.0) || !(0.0..=1.0).contains(&focal_point.1) {
        return Err(MediaError::InvalidFocalPoint);
    }

    if image_width == 0 || image_height == 0 || target_width == 0 || target_height == 0 {
        return Ok(CropBox {
            x: 0,
            y: 0,
            width: image_width,
            height: image_height,
        });
    }

    let target_aspect = target_width as f64 / target_height as f64;
    let img_aspect = image_width as f64 / image_height as f64;

    let (crop_w, crop_h) = if img_aspect > target_aspect {
        // Image is wider than target: fit height, calculate width
        let h = image_height;
        let w = ((image_height as f64 * target_aspect).round() as u32).min(image_width);
        (w, h)
    } else {
        // Image is taller than target: fit width, calculate height
        let w = image_width;
        let h = ((image_width as f64 / target_aspect).round() as u32).min(image_height);
        (w, h)
    };

    let focal_x = (image_width as f32 * focal_point.0).round() as u32;
    let focal_y = (image_height as f32 * focal_point.1).round() as u32;

    let crop_x = focal_x
        .saturating_sub(crop_w / 2)
        .min(image_width.saturating_sub(crop_w));
    let crop_y = focal_y
        .saturating_sub(crop_h / 2)
        .min(image_height.saturating_sub(crop_h));

    Ok(CropBox {
        x: crop_x,
        y: crop_y,
        width: crop_w,
        height: crop_h,
    })
}

/// Responsive WebP image variant specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebPVariantMetadata {
    pub name: CompactString,
    pub width: u32,
    pub height: u32,
    pub quality: u8,
    pub estimated_size_bytes: u64,
}

/// Generates standard responsive WebP derivative profiles for responsive HTML `<picture>` tags.
#[must_use]
pub fn generate_responsive_webp_variants(
    original_width: u32,
    original_height: u32,
) -> Vec<WebPVariantMetadata> {
    if original_width == 0 || original_height == 0 {
        return Vec::new();
    }

    let aspect = original_height as f64 / original_width as f64;
    let target_widths = [
        ("thumb", 300, 75),
        ("sm", 640, 80),
        ("md", 1024, 82),
        ("lg", 1600, 85),
    ];

    let mut variants = Vec::new();

    for (name, target_w, quality) in target_widths {
        if target_w <= original_width {
            let h = (target_w as f64 * aspect).round() as u32;
            let pixels = (target_w as u64) * (h as u64);
            // WebP average compression ~0.12 bytes per pixel
            let est_size = (pixels as f64 * 0.12).round() as u64;

            variants.push(WebPVariantMetadata {
                name: name.into(),
                width: target_w,
                height: h,
                quality,
                estimated_size_bytes: est_size,
            });
        }
    }

    variants
}

/// Generates an XML sitemap of all published Emdash editorial articles.
#[must_use]
pub fn generate_editorial_xml_sitemap(documents: &[&EmdashDocument], base_url: &str) -> String {
    let clean_base = base_url.trim_end_matches('/');
    let mut urls = String::new();

    for doc in documents {
        if matches!(doc.workflow_state, EditorialWorkflowState::Published { .. }) {
            urls.push_str(&format!(
                r#"  <url>
    <loc>{clean_base}/editorial/{}</loc>
    <lastmod>2026-10-17</lastmod>
    <changefreq>weekly</changefreq>
    <priority>0.8</priority>
  </url>
"#,
                doc.slug
            ));
        }
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
{urls}</urlset>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emdash_block_rendering_and_word_count() {
        let heading = EmdashBlock::Heading {
            level: 1,
            text: "Emdash CMS Substrate Architecture".into(),
        };
        let para = EmdashBlock::Paragraph {
            text: "Pure Rust headless publishing engine designed for microsecond rendering.".into(),
        };
        let callout = EmdashBlock::Callout {
            style: CalloutStyle::Success,
            title: Some("Notice".into()),
            text: "Zero JavaScript runtime required for standard blog readers.".into(),
        };

        assert!(heading.render_html().contains("<h1>"));
        assert!(para.render_html().contains("<p>"));
        assert!(callout.render_html().contains("callout-success"));

        assert_eq!(heading.word_count(), 4);
        assert_eq!(para.word_count(), 9);
        assert_eq!(callout.word_count(), 8);
    }

    #[test]
    fn test_workflow_state_transitions() {
        let draft = EditorialWorkflowState::Draft;
        assert_eq!(draft.state_name(), "Draft");

        // Draft -> InReview (Valid)
        let in_review = draft
            .transition_to(EditorialWorkflowState::InReview)
            .unwrap();
        assert_eq!(in_review.state_name(), "InReview");

        // InReview -> Published (Valid)
        let published = in_review
            .transition_to(EditorialWorkflowState::Published {
                published_timestamp: 1_700_000_000,
            })
            .unwrap();
        assert_eq!(published.state_name(), "Published");

        // Published -> Draft (Unpublish - Valid)
        let unpub = published
            .transition_to(EditorialWorkflowState::Draft)
            .unwrap();
        assert_eq!(unpub, EditorialWorkflowState::Draft);

        // Draft -> Published directly is INVALID
        assert!(
            draft
                .transition_to(EditorialWorkflowState::Published {
                    published_timestamp: 1_700_000_000,
                })
                .is_err()
        );
    }

    #[test]
    fn test_emdash_document_lifecycle() {
        let mut doc = EmdashDocument::new(
            "post_001",
            "High Throughput CMS in Rust",
            "high-throughput-cms-rust",
            "Faez Barghasa",
        );

        doc.add_block(EmdashBlock::Paragraph {
            text: "This is a detailed analysis of high performance systems.".into(),
        });

        assert_eq!(doc.reading_time_minutes(), 1);
        let html = doc.render_html();
        assert!(html.contains("High Throughput CMS in Rust"));
        assert!(html.contains("Faez Barghasa"));
        assert!(html.contains("<article class=\"emdash-article\" id=\"post_001\">"));
    }

    #[test]
    fn test_focal_point_crop_calculation() {
        // Image 1920x1080 (16:9), target 1080x1080 (1:1), focal point centered (0.5, 0.5)
        let crop = calculate_focal_crop(1920, 1080, 1080, 1080, (0.5, 0.5)).unwrap();
        assert_eq!(crop.height, 1080);
        assert_eq!(crop.width, 1080);
        assert_eq!(crop.y, 0);
        assert_eq!(crop.x, (1920 - 1080) / 2); // 420

        // Invalid focal point
        assert!(calculate_focal_crop(1920, 1080, 500, 500, (1.5, 0.5)).is_err());
    }

    #[test]
    fn test_webp_variant_generation() {
        let variants = generate_responsive_webp_variants(1920, 1080);
        assert_eq!(variants.len(), 4);
        assert_eq!(variants[0].name.as_str(), "thumb");
        assert_eq!(variants[0].width, 300);
        assert_eq!(variants[3].name.as_str(), "lg");
        assert_eq!(variants[3].width, 1600);
    }

    #[test]
    fn test_editorial_xml_sitemap() {
        let mut doc1 = EmdashDocument::new("doc1", "Doc 1", "doc-1", "Admin");
        doc1.workflow_state = EditorialWorkflowState::Published {
            published_timestamp: 1_700_000_000,
        };

        let mut doc2 = EmdashDocument::new("doc2", "Doc 2", "doc-2", "Admin");
        doc2.workflow_state = EditorialWorkflowState::Draft;

        let sitemap = generate_editorial_xml_sitemap(&[&doc1, &doc2], "https://example.com");
        assert!(sitemap.contains("<loc>https://example.com/editorial/doc-1</loc>"));
        assert!(!sitemap.contains("doc-2"));
    }
}
