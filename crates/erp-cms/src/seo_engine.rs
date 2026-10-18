//! Technical SEO & Semantic Social Graph Engine (`erp_cms::seo_engine`).
//!
//! Generates valid Schema.org JSON-LD graph structures, OpenGraph/Twitter Card metadata,
//! canonical links, XML sitemaps, and robots.txt directives in pure Rust.

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// High-level SEO configuration and metadata container.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeoMetadata {
    pub title: CompactString,
    pub description: CompactString,
    pub canonical_url: CompactString,
    pub og_type: CompactString,
    pub og_image: CompactString,
    pub site_name: CompactString,
    pub json_ld: String,
}

impl SeoMetadata {
    /// Renders standard HTML head tags for SEO, OpenGraph, Twitter, and Schema.org JSON-LD.
    #[must_use]
    pub fn render_head_tags(&self) -> String {
        format!(
            r##"<title>{title}</title>
    <meta name="description" content="{desc}">
    <link rel="canonical" href="{canonical}">
    <meta name="robots" content="index, follow, max-image-preview:large, max-snippet:-1, max-video-preview:-1">
    
    <!-- OpenGraph / Facebook -->
    <meta property="og:type" content="{og_type}">
    <meta property="og:site_name" content="{site_name}">
    <meta property="og:title" content="{title}">
    <meta property="og:description" content="{desc}">
    <meta property="og:url" content="{canonical}">
    <meta property="og:image" content="{og_image}">
    <meta property="og:image:width" content="1200">
    <meta property="og:image:height" content="630">
    
    <!-- Twitter Cards -->
    <meta name="twitter:card" content="summary_large_image">
    <meta name="twitter:site" content="@rustnext_erp">
    <meta name="twitter:creator" content="@rustnext_erp">
    <meta name="twitter:title" content="{title}">
    <meta name="twitter:description" content="{desc}">
    <meta name="twitter:image" content="{og_image}">
    
    <!-- Schema.org JSON-LD -->
    <script type="application/ld+json">
    {json_ld}
    </script>"##,
            title = self.title,
            desc = self.description,
            canonical = self.canonical_url,
            og_type = self.og_type,
            site_name = self.site_name,
            og_image = self.og_image,
            json_ld = self.json_ld
        )
    }

    /// Constructs generic website SEO metadata.
    #[must_use]
    pub fn new_website(title: &str, description: &str, canonical_url: &str) -> Self {
        let json_ld = serde_json::json!({
            "@context": "https://schema.org",
            "@type": "WebSite",
            "name": title,
            "description": description,
            "url": canonical_url,
        })
        .to_string();

        Self {
            title: title.into(),
            description: description.into(),
            canonical_url: canonical_url.into(),
            og_type: "website".into(),
            og_image: "https://rustnext.enterprise.io/assets/og-default.webp".into(),
            site_name: "rustnext Enterprise".into(),
            json_ld,
        }
    }

    /// Constructs SEO metadata for the SVoD Video Streaming Platform (`VideoObject`, `Movie`, `Series`).
    #[must_use]
    pub fn for_svod(slug: &str, variant: &str) -> Self {
        let title = format!("Cinema 4K Ultra SVoD Streaming — {variant} Edition | rustnext").into();
        let description = "Stream award-winning cinema in Ultra-HD 4K with HLS low-latency delivery, real-time subtitle vector search, and synchronized watch parties.".into();
        let canonical_url =
            format!("https://rustnext.enterprise.io/templates/{slug}?variant={variant}").into();

        let json_ld = serde_json::json!({
            "@context": "https://schema.org",
            "@type": "VideoObject",
            "name": "Chronicles of Rust: Planetary Operating Substrate",
            "description": description,
            "thumbnailUrl": [
                "https://rustnext.enterprise.io/assets/cinema/thumb-1080p.webp",
                "https://rustnext.enterprise.io/assets/cinema/thumb-720p.webp"
            ],
            "uploadDate": "2026-09-28T08:00:00+00:00",
            "duration": "PT2H15M",
            "contentUrl": "https://rustnext.enterprise.io/stream/media-1080p.m3u8",
            "embedUrl": "https://rustnext.enterprise.io/embed/svod-streaming",
            "interactionStatistic": {
                "@type": "InteractionCounter",
                "interactionService": {
                    "@type": "WebSite",
                    "name": "rustnext Cinema",
                    "url": "https://rustnext.enterprise.io"
                },
                "interactionType": { "@type": "WatchAction" },
                "userInteractionCount": 142850
            },
            "productionCompany": {
                "@type": "Organization",
                "name": "rustnext Studios",
                "url": "https://rustnext.enterprise.io"
            }
        })
        .to_string();

        Self {
            title,
            description,
            canonical_url,
            og_type: "video.movie".into(),
            og_image: "https://rustnext.enterprise.io/assets/cinema/og-svod.webp".into(),
            site_name: "rustnext Cinema".into(),
            json_ld,
        }
    }

    /// Constructs SEO metadata for the Digital Learning & LMS Academy (`Course`, `EducationalOccupationalCredential`).
    #[must_use]
    pub fn for_lms(slug: &str, variant: &str) -> Self {
        let title = format!(
            "Advanced Systems Engineering & Distributed Kernels — {variant} LMS | rustnext Academy"
        )
        .into();
        let description = "Master pure-Rust distributed systems, micro-topologies, and cryptographic verification with Merkle-anchored graduation diplomas.".into();
        let canonical_url =
            format!("https://rustnext.enterprise.io/templates/{slug}?variant={variant}").into();

        let json_ld = serde_json::json!({
            "@context": "https://schema.org",
            "@type": "Course",
            "name": "Mastering Planetary Scale Distributed Kernels",
            "description": description,
            "provider": {
                "@type": "Organization",
                "name": "rustnext Engineering Academy",
                "url": "https://rustnext.enterprise.io"
            },
            "educationalCredentialAwarded": {
                "@type": "EducationalOccupationalCredential",
                "name": "Certified Pure-Rust Systems Architect (PRSA)",
                "credentialCategory": "Professional Degree",
                "recognizedBy": {
                    "@type": "Organization",
                    "name": "Frappe-Rust Foundation"
                }
            },
            "hasCourseInstance": {
                "@type": "CourseInstance",
                "courseMode": "Online",
                "duration": "P12W",
                "instructor": {
                    "@type": "Person",
                    "name": "Dr. Sarah Chen, Systems Lead",
                    "jobTitle": "Principal Kernel Engineer"
                }
            },
            "offers": {
                "@type": "Offer",
                "category": "Tuition",
                "price": "1490.00",
                "priceCurrency": "USD",
                "availability": "https://schema.org/InStock"
            }
        })
        .to_string();

        Self {
            title,
            description,
            canonical_url,
            og_type: "website".into(),
            og_image: "https://rustnext.enterprise.io/assets/lms/og-lms.webp".into(),
            site_name: "rustnext Academy".into(),
            json_ld,
        }
    }

    /// Constructs SEO metadata for the Digital Products & Creator Hub (`SoftwareApplication`, `Product`, `Offer`).
    #[must_use]
    pub fn for_digital_goods(slug: &str, variant: &str) -> Self {
        let title = format!("Creator Hub & Developer Assets — {variant} Vault | rustnext").into();
        let description = "Instant software licenses, single-use encrypted download links, automated creator payouts, and verified developer SDKs.".into();
        let canonical_url =
            format!("https://rustnext.enterprise.io/templates/{slug}?variant={variant}").into();

        let json_ld = serde_json::json!({
            "@context": "https://schema.org",
            "@type": "SoftwareApplication",
            "name": "Synthetix 3D Asset Shader Suite",
            "operatingSystem": "Cross-Platform (Linux, macOS, Windows)",
            "applicationCategory": "DeveloperApplication",
            "aggregateRating": {
                "@type": "AggregateRating",
                "ratingValue": "4.96",
                "reviewCount": "1280"
            },
            "offers": {
                "@type": "Offer",
                "price": "89.00",
                "priceCurrency": "USD",
                "priceValidUntil": "2027-12-31",
                "availability": "https://schema.org/InStock"
            },
            "author": {
                "@type": "Organization",
                "name": "Digital Artisans Guild"
            }
        })
        .to_string();

        Self {
            title,
            description,
            canonical_url,
            og_type: "product".into(),
            og_image: "https://rustnext.enterprise.io/assets/goods/og-goods.webp".into(),
            site_name: "rustnext Creator Hub".into(),
            json_ld,
        }
    }

    /// Constructs SEO metadata for the Industrial B2B & Wholesale Matrix (`Product`, `UnitPriceSpecification`).
    #[must_use]
    pub fn for_b2b(slug: &str, variant: &str) -> Self {
        let title =
            format!("Industrial Procurement & 3D CAD Matrix — {variant} B2B | rustnext").into();
        let description = "Enterprise wholesale procurement, interactive 3D WebGL CAD exploded assemblies, Siemens Net 60 corporate terms, and ZUGFeRD 2.2 e-invoices.".into();
        let canonical_url =
            format!("https://rustnext.enterprise.io/templates/{slug}?variant={variant}").into();

        let json_ld = serde_json::json!({
            "@context": "https://schema.org",
            "@type": "Product",
            "name": "Servo-Actuated Precision Harmonic Drive Gearbox",
            "image": "https://rustnext.enterprise.io/assets/industrial/cad-drive.webp",
            "description": description,
            "sku": "IND-HDG-9942",
            "mpn": "9942-AX-PRO",
            "brand": {
                "@type": "Brand",
                "name": "Kinetics Heavy Industries"
            },
            "offers": {
                "@type": "Offer",
                "url": canonical_url,
                "priceCurrency": "USD",
                "price": "2450.00",
                "itemAvailability": "https://schema.org/InStock",
                "priceSpecification": {
                    "@type": "UnitPriceSpecification",
                    "price": "2450.00",
                    "priceCurrency": "USD",
                    "unitCode": "C62",
                    "billingDuration": "P60D"
                },
                "seller": {
                    "@type": "Organization",
                    "name": "rustnext Industrial Wholesale"
                }
            }
        })
        .to_string();

        Self {
            title,
            description,
            canonical_url,
            og_type: "product".into(),
            og_image: "https://rustnext.enterprise.io/assets/industrial/og-b2b.webp".into(),
            site_name: "rustnext Industrial B2B".into(),
            json_ld,
        }
    }

    /// Constructs SEO metadata for the Consumer B2C Omnichannel Flagship (`Product`, `AggregateOffer`, `Brand`).
    #[must_use]
    pub fn for_b2c(slug: &str, variant: &str) -> Self {
        let title =
            format!("Haute Horlogerie & Luxury Retail — {variant} Flagship | rustnext").into();
        let description = "High-velocity consumer retail flagship featuring 60+ FPS virtualized SKU grid, SurrealDB live stock feeds, and atomic ACID checkout.".into();
        let canonical_url =
            format!("https://rustnext.enterprise.io/templates/{slug}?variant={variant}").into();

        let json_ld = serde_json::json!({
            "@context": "https://schema.org",
            "@type": "Product",
            "name": "Chronosphere Tourbillon Calibre V",
            "image": "https://rustnext.enterprise.io/assets/retail/watch-tourbillon.webp",
            "description": description,
            "brand": {
                "@type": "Brand",
                "name": "LuxeGen Haute Horlogerie"
            },
            "offers": {
                "@type": "AggregateOffer",
                "lowPrice": "14500.00",
                "highPrice": "28900.00",
                "priceCurrency": "CHF",
                "offerCount": "12",
                "availability": "https://schema.org/InStock"
            },
            "aggregateRating": {
                "@type": "AggregateRating",
                "ratingValue": "5.0",
                "reviewCount": "48"
            }
        })
        .to_string();

        Self {
            title,
            description,
            canonical_url,
            og_type: "product".into(),
            og_image: "https://rustnext.enterprise.io/assets/retail/og-b2c.webp".into(),
            site_name: "LuxeGen Flagship".into(),
            json_ld,
        }
    }

    /// Constructs SEO metadata for the Financial Trading & Brokerage Hub (`FinancialProduct`).
    #[must_use]
    pub fn for_trading(slug: &str, variant: &str) -> Self {
        let title =
            format!("Institutional Trading Exchange & Prime Terminal — {variant} Desk | rustnext")
                .into();
        let description = "High-frequency financial trading terminal with real-time Level-2 order book depth ladder, microsecond trade tape, and Zero-Knowledge solvent balance proofs.".into();
        let canonical_url =
            format!("https://rustnext.enterprise.io/templates/{slug}?variant={variant}").into();

        let json_ld = serde_json::json!({
            "@context": "https://schema.org",
            "@type": "FinancialProduct",
            "name": "Institutional Spot & Derivatives Prime Clearing",
            "description": description,
            "provider": {
                "@type": "Organization",
                "name": "rustnext Prime Brokerage Ltd",
                "url": "https://rustnext.enterprise.io"
            },
            "feesAndCommissionsSpecification": "Zero Maker Fee, 0.015% Taker Fee with Volume Rebates",
            "annualPercentageRate": 0.0
        }).to_string();

        Self {
            title,
            description,
            canonical_url,
            og_type: "website".into(),
            og_image: "https://rustnext.enterprise.io/assets/trading/og-trading.webp".into(),
            site_name: "rustnext Exchange".into(),
            json_ld,
        }
    }
}

/// Generates an enterprise-grade XML sitemap indexing all 10 domains and their UI/UX aesthetic variants (100+ paths).
#[must_use]
pub fn generate_sitemap_xml() -> String {
    let domains = [
        "svod-streaming",
        "lms-academy",
        "digital-goods",
        "b2b-industrial",
        "b2c-retail",
        "trading-exchange",
        "saas-cloud",
        "healthcare-clinical",
        "real-estate-spatial",
        "gastronomy-hospitality",
    ];

    let variants = [
        "swiss",
        "bento",
        "neobrutalist",
        "editorial",
        "cyberpunk",
        "split-screen",
        "cockpit",
        "playful",
        "scrollytelling",
        "cinematic-3d",
        "vaporwave",
        "retrowave",
        "neonwave",
        "tasteful",
    ];

    let mut urls = String::new();

    // Portal home
    urls.push_str(
        r#"  <url>
    <loc>https://rustnext.enterprise.io/templates</loc>
    <lastmod>2026-09-28</lastmod>
    <changefreq>daily</changefreq>
    <priority>1.0</priority>
  </url>
"#,
    );

    for domain in domains {
        // Base domain
        urls.push_str(&format!(
            r#"  <url>
    <loc>https://rustnext.enterprise.io/templates/{domain}</loc>
    <lastmod>2026-09-28</lastmod>
    <changefreq>weekly</changefreq>
    <priority>0.9</priority>
  </url>
"#
        ));

        // Variants
        for variant in variants {
            urls.push_str(&format!(
                r#"  <url>
    <loc>https://rustnext.enterprise.io/templates/{domain}?variant={variant}</loc>
    <lastmod>2026-09-28</lastmod>
    <changefreq>weekly</changefreq>
    <priority>0.8</priority>
  </url>
"#
            ));
        }
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
{urls}</urlset>"#
    )
}

/// Generates standard robots.txt directives optimizing search crawler crawl budget.
#[must_use]
pub fn generate_robots_txt() -> String {
    r#"# rustnext Planetary ERP & Storefront Engine
User-agent: *
Allow: /
Allow: /templates
Allow: /templates/
Allow: /storefront
Allow: /site/
Allow: /feed.xml
Disallow: /api/
Disallow: /admin/
Disallow: /.well-known/

# Sitemaps
Sitemap: https://rustnext.enterprise.io/sitemap.xml
"#
    .to_string()
}

/// Represents an entry in a sitemap.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SitemapEntry {
    pub loc: String,
    pub lastmod: Option<String>,
    pub changefreq: Option<String>,
    pub priority: Option<f32>,
}

/// Generates an XML sitemap including default template routes and extra dynamic routes (such as published Emdash documents or Builder pages).
#[must_use]
pub fn generate_dynamic_sitemap_xml(extra_entries: &[SitemapEntry]) -> String {
    let mut base_sitemap = generate_sitemap_xml();
    if extra_entries.is_empty() {
        return base_sitemap;
    }

    let mut extra_xml = String::new();
    for entry in extra_entries {
        extra_xml.push_str("  <url>\n");
        extra_xml.push_str(&format!("    <loc>{}</loc>\n", entry.loc));
        if let Some(ref lastmod) = entry.lastmod {
            extra_xml.push_str(&format!("    <lastmod>{}</lastmod>\n", lastmod));
        }
        if let Some(ref changefreq) = entry.changefreq {
            extra_xml.push_str(&format!("    <changefreq>{}</changefreq>\n", changefreq));
        }
        if let Some(priority) = entry.priority {
            extra_xml.push_str(&format!("    <priority>{:.1}</priority>\n", priority));
        }
        extra_xml.push_str("  </url>\n");
    }

    if let Some(pos) = base_sitemap.rfind("</urlset>") {
        base_sitemap.insert_str(pos, &extra_xml);
        base_sitemap
    } else {
        base_sitemap
    }
}

/// RSS Feed Item for syndicated publication.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RssFeedItem {
    pub title: String,
    pub link: String,
    pub description: String,
    pub pub_date: String,
    pub author: Option<String>,
    pub guid: String,
}

/// Generates an RSS 2.0 XML syndicated publication feed.
#[must_use]
pub fn generate_rss_2_xml(
    title: &str,
    link: &str,
    description: &str,
    items: &[RssFeedItem],
) -> String {
    let mut items_xml = String::new();
    for item in items {
        items_xml.push_str("    <item>\n");
        items_xml.push_str(&format!(
            "      <title><![CDATA[{}]]></title>\n",
            item.title
        ));
        items_xml.push_str(&format!("      <link>{}</link>\n", item.link));
        items_xml.push_str(&format!(
            "      <guid isPermaLink=\"false\">{}</guid>\n",
            item.guid
        ));
        items_xml.push_str(&format!(
            "      <description><![CDATA[{}]]></description>\n",
            item.description
        ));
        items_xml.push_str(&format!("      <pubDate>{}</pubDate>\n", item.pub_date));
        if let Some(ref author) = item.author {
            items_xml.push_str(&format!("      <author>{}</author>\n", author));
        }
        items_xml.push_str("    </item>\n");
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title><![CDATA[{title}]]></title>
    <link>{link}</link>
    <description><![CDATA[{description}]]></description>
    <atom:link href="{link}/feed.xml" rel="self" type="application/rss+xml" />
    <language>en-us</language>
{items_xml}  </channel>
</rss>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_sitemap_xml_generation() {
        let entries = vec![
            SitemapEntry {
                loc: "https://rustnext.enterprise.io/site/announcing-v2".into(),
                lastmod: Some("2026-10-17".into()),
                changefreq: Some("daily".into()),
                priority: Some(1.0),
            },
            SitemapEntry {
                loc: "https://rustnext.enterprise.io/site/pricing".into(),
                lastmod: Some("2026-10-17".into()),
                changefreq: Some("weekly".into()),
                priority: Some(0.9),
            },
        ];

        let xml = generate_dynamic_sitemap_xml(&entries);
        assert!(xml.contains("<loc>https://rustnext.enterprise.io/site/announcing-v2</loc>"));
        assert!(xml.contains("<loc>https://rustnext.enterprise.io/site/pricing</loc>"));
        assert!(xml.contains("<priority>1.0</priority>"));
        assert!(xml.contains("<priority>0.9</priority>"));
        assert!(xml.contains("</urlset>"));
    }

    #[test]
    fn test_rss_feed_generation() {
        let items = vec![RssFeedItem {
            title: "Planetary Scale Release".into(),
            link: "https://rustnext.enterprise.io/site/release-note".into(),
            description: "High performance ERP written in pure Rust.".into(),
            pub_date: "Sat, 17 Oct 2026 12:00:00 GMT".into(),
            author: Some("editor@rustnext.io".into()),
            guid: "item-001".into(),
        }];

        let rss = generate_rss_2_xml(
            "rustnext Journal",
            "https://rustnext.enterprise.io",
            "Latest news and architectural blueprints",
            &items,
        );
        assert!(rss.contains("<title><![CDATA[Planetary Scale Release]]></title>"));
        assert!(rss.contains("<guid isPermaLink=\"false\">item-001</guid>"));
        assert!(rss.contains("https://rustnext.enterprise.io/feed.xml"));
    }
}
