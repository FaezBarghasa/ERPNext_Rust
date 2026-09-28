//! Technical SEO & Semantic Social Graph Engine (`erp_cms::seo_engine`).
//!
//! Generates valid Schema.org JSON-LD graph structures, OpenGraph/Twitter Card metadata,
//! canonical links, XML sitemaps, and robots.txt directives in pure Rust.

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// High-level SEO configuration and metadata container.
#[derive(Clone, Debug, Serialize, Deserialize)]
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
        let title =
            format!("Creator Studio & Developer Assets — {variant} Vault | rustnext").into();
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
            format!("Institutional Prime Brokerage & Terminal — {variant} Desk | rustnext").into();
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

/// Generates an enterprise-grade XML sitemap indexing all 6 domains and their 5 aesthetic variants (30 paths).
#[must_use]
pub fn generate_sitemap_xml() -> String {
    let domains = [
        "svod-streaming",
        "lms-academy",
        "digital-goods",
        "b2b-industrial",
        "b2c-retail",
        "trading-exchange",
    ];

    let variants = [
        "awwwards",
        "cyberpunk",
        "vaporwave",
        "retrowave",
        "neonwave",
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
Disallow: /api/
Disallow: /admin/
Disallow: /.well-known/

# Sitemaps
Sitemap: https://rustnext.enterprise.io/sitemap.xml
"#
    .to_string()
}
