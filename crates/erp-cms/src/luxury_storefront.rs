//! LuxeGen & Medusa's Bloom Luxury Commercial Storefront Template Engine (`erp-cms::luxury_storefront`).
//!
//! Statically embeds and compiles the Awwwards-grade luxury e-commerce frontend
//! featuring Three.js WebGL 3D canvases, GSAP kinetic typography, interactive
//! 3D product configurator, and atomic checkout integration.

use compact_str::CompactString;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Catalog item specification for luxury e-commerce.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LuxuryProduct {
    pub id: CompactString,
    pub title: CompactString,
    pub category: CompactString,
    pub price_usd: Decimal,
    pub stock_qty: Decimal,
    pub description: CompactString,
    pub badge: CompactString,
    pub model_type: CompactString,
}

/// Returns the standard luxury storefront product catalog.
#[must_use]
pub fn get_luxury_catalog() -> Vec<LuxuryProduct> {
    vec![
        LuxuryProduct {
            id: "CHRONOS-01".into(),
            title: "Chronos-01 Kinetic Tourbillon".into(),
            category: "timepieces".into(),
            price_usd: dec!(14500.00),
            stock_qty: dec!(4),
            description: "Titanium-grade constant-force escapement with 72-hour power reserve and sapphire bridge architecture.".into(),
            badge: "Limited Edition • 50 Pcs".into(),
            model_type: "tourbillon".into(),
        },
        LuxuryProduct {
            id: "NEURAL-X".into(),
            title: "Neural-X Synapse Headband".into(),
            category: "neural".into(),
            price_usd: dec!(3850.00),
            stock_qty: dec!(12),
            description: "Sub-millisecond dry-electrode EEG & HRV biometric telemetry receiver with edge neural decoding.".into(),
            badge: "FIFO Stock • In Stock".into(),
            model_type: "synapse".into(),
        },
        LuxuryProduct {
            id: "AETHEL-M1".into(),
            title: "Aethel-Core M1 Acoustic Chamber".into(),
            category: "audio".into(),
            price_usd: dec!(2400.00),
            stock_qty: dec!(9),
            description: "Planar magnetic acoustic driver with vapor-deposited pure beryllium diaphragms and gold-plated acoustic grid.".into(),
            badge: "Audiophile Grade".into(),
            model_type: "transducer".into(),
        },
        LuxuryProduct {
            id: "VALKYRIE-04".into(),
            title: "Valkyrie Graphene Cyber-Shell".into(),
            category: "apparel".into(),
            price_usd: dec!(1950.00),
            stock_qty: dec!(15),
            description: "Ultra-lightweight monolithic graphene membrane with dynamic thermoregulation and radar-diffusive weave.".into(),
            badge: "Performance Techwear".into(),
            model_type: "cybershell".into(),
        },
        LuxuryProduct {
            id: "SOL-MATRIX".into(),
            title: "Sol-Matrix Photovoltaic Chronometer".into(),
            category: "timepieces".into(),
            price_usd: dec!(8200.00),
            stock_qty: dec!(6),
            description: "Synthetic diamond photovoltaic dial harnessing ambient spectrum radiation with perpetual atomic sync.".into(),
            badge: "Perpetual Escapement".into(),
            model_type: "solmatrix".into(),
        },
        LuxuryProduct {
            id: "VDA-DRONE".into(),
            title: "VDA-5050 Autonomous Courier Drone".into(),
            category: "neural".into(),
            price_usd: dec!(5600.00),
            stock_qty: dec!(8),
            description: "Indoor 3D LiDAR automated navigation platform with VDA 5050 industrial fleet interoperability.".into(),
            badge: "Robotics Fleet Ready".into(),
            model_type: "drone".into(),
        },
    ]
}

/// Raw embedded CSS for the luxury storefront.
pub const LUXURY_STOREFRONT_CSS: &str = include_str!("../../../frontend/templates/luxegen-storefront/style.css");

/// Raw embedded JavaScript application engine for the luxury storefront.
pub const LUXURY_STOREFRONT_JS: &str = include_str!("../../../frontend/templates/luxegen-storefront/app.js");

/// Raw embedded base HTML template for the luxury storefront.
pub const LUXURY_STOREFRONT_HTML: &str = include_str!("../../../frontend/templates/luxegen-storefront/index.html");

/// Compiles a standalone, zero-IPC self-contained HTML document with inlined CSS & JS.
#[must_use]
pub fn render_luxury_storefront_html() -> String {
    let html = LUXURY_STOREFRONT_HTML;
    // Replace linked stylesheet and script with inlined assets for self-contained zero-IPC delivery
    let with_css = html.replace(
        r#"<link rel="stylesheet" href="style.css">"#,
        &format!("<style>\n{LUXURY_STOREFRONT_CSS}\n</style>"),
    );
    with_css.replace(
        r#"<script src="app.js"></script>"#,
        &format!("<script>\n{LUXURY_STOREFRONT_JS}\n</script>"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_luxury_catalog_integrity() {
        let catalog = get_luxury_catalog();
        assert_eq!(catalog.len(), 6);
        assert_eq!(catalog[0].id, "CHRONOS-01");
        assert!(catalog[0].price_usd > Decimal::ZERO);
    }

    #[test]
    fn test_rendered_storefront_contains_critical_nodes() {
        let rendered = render_luxury_storefront_html();
        assert!(rendered.contains("hero-canvas"));
        assert!(rendered.contains("configurator-canvas"));
        assert!(rendered.contains("LuxeGen"));
        assert!(rendered.contains("Three.js"));
    }
}
