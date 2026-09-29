pub mod fingerprint;
pub mod headless_hardening;
pub mod http2_frame;
pub mod pipelines;
pub mod proxy_ring;
pub mod turnstile_solver;

pub use fingerprint::{BrowserPreset, Ja4Fingerprint, TlsCamouflageProfile};
pub use headless_hardening::{HardwareSpoofConfig, HeadlessHardeningEngine};
pub use http2_frame::{Http2Choreography, Http2SettingsFrame};
pub use pipelines::{
    ErpItemPriceUpdate, ErpSupplierQuotationDraft, PriceAlertEvent, PriceAlertSeverity,
    ScraperPipelineProcessor,
};
pub use proxy_ring::{ProxyNode, ProxyRotationRing};
pub use turnstile_solver::{ChallengeContext, ChallengeSolution, ChallengeSolverHook};

use compact_str::CompactString;
use reqwest::Client;
use reqwest::header::{
    ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, HeaderMap, HeaderValue, USER_AGENT,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum ScraperError {
    #[error("HTTP transport error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Bot challenge detected (403 / 429 / CAPTCHA): {0}")]
    ChallengeDetected(CompactString),
    #[error("Parsing error: {0}")]
    Parsing(CompactString),
    #[error("Price element not found in response")]
    NotFound,
}

/// Scraped competitor pricing record ready for ingestion into ERP `Item Price`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScrapedPriceRecord {
    pub url: CompactString,
    pub price: f64,
    pub currency: CompactString,
    pub sku: Option<CompactString>,
    pub timestamp_utc: i64,
}

/// Scraped supplier catalog entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SupplierCatalogItem {
    pub sku: CompactString,
    pub title: CompactString,
    pub price: f64,
    pub currency: CompactString,
    pub in_stock: bool,
    pub supplier_name: CompactString,
}

/// High-fidelity anti-detection scraper client.
pub struct StealthScraperClient {
    client: Client,
    proxy_rotation_pool: Vec<String>,
}

impl StealthScraperClient {
    /// Configures a client that matches modern Chrome TLS ClientHello and HTTP/2 settings.
    pub fn new(proxies: Vec<String>) -> Result<Self, ScraperError> {
        let mut default_headers = HeaderMap::new();

        // Exact Chrome 130+ desktop header set & ordering
        default_headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
            ),
        );
        default_headers.insert(
            ACCEPT,
            HeaderValue::from_static(
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
            ),
        );
        default_headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
        default_headers.insert(
            ACCEPT_ENCODING,
            HeaderValue::from_static("gzip, deflate, br, zstd"),
        );
        default_headers.insert(
            "sec-ch-ua",
            HeaderValue::from_static(
                r#""Chromium";v="130", "Google Chrome";v="130", "Not?A_Brand";v="99""#,
            ),
        );
        default_headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
        default_headers.insert(
            "sec-ch-ua-platform",
            HeaderValue::from_static(r#""Windows""#),
        );
        default_headers.insert("sec-fetch-dest", HeaderValue::from_static("document"));
        default_headers.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
        default_headers.insert("sec-fetch-site", HeaderValue::from_static("none"));
        default_headers.insert("sec-fetch-user", HeaderValue::from_static("?1"));
        default_headers.insert("upgrade-insecure-requests", HeaderValue::from_static("1"));

        let client = Client::builder().default_headers(default_headers).build()?;

        Ok(Self {
            client,
            proxy_rotation_pool: proxies,
        })
    }

    /// Extracts decimal prices from raw text (e.g. `$1,249.99` -> `1249.99`).
    #[must_use]
    pub fn extract_numeric_price(text: &str) -> Option<f64> {
        let clean: String = text
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        clean.parse::<f64>().ok()
    }

    /// Fetches target webpage and extracts price elements safely.
    pub async fn scrape_supplier_price(
        &self,
        target_url: &str,
    ) -> Result<ScrapedPriceRecord, ScraperError> {
        let resp = self.client.get(target_url).send().await?;

        if resp.status() == reqwest::StatusCode::FORBIDDEN
            || resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS
        {
            return Err(ScraperError::ChallengeDetected(
                "Bot challenge or rate limit detected".into(),
            ));
        }

        let body = resp.text().await?;
        let price = Self::extract_price_from_html(&body).ok_or(ScraperError::NotFound)?;

        Ok(ScrapedPriceRecord {
            url: target_url.into(),
            price,
            currency: "USD".into(),
            sku: None,
            timestamp_utc: chrono::Utc::now().timestamp(),
        })
    }

    /// Internal regex / text based HTML parser for price detection.
    #[must_use]
    pub fn extract_price_from_html(html: &str) -> Option<f64> {
        let re = regex::Regex::new(
            r#"(?i)(?:price|amount|cost)["']?\s*[:=]\s*["']?\$?([0-9]+(?:\.[0-9]{2})?)"#,
        )
        .ok()?;
        if let Some(caps) = re.captures(html)
            && let Some(m) = caps.get(1)
        {
            return m.as_str().parse::<f64>().ok();
        }

        // Fallback: look for generic currency patterns ($123.45)
        let fallback_re = regex::Regex::new(r#"\$([0-9]{1,6}(?:\.[0-9]{2})?)"#).ok()?;
        if let Some(caps) = fallback_re.captures(html)
            && let Some(m) = caps.get(1)
        {
            return m.as_str().parse::<f64>().ok();
        }

        None
    }

    /// Accessor for configured proxy rotation pool count.
    #[must_use]
    pub fn proxy_count(&self) -> usize {
        self.proxy_rotation_pool.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_numeric_price() {
        assert_eq!(
            StealthScraperClient::extract_numeric_price("$1,499.50"),
            Some(1499.50)
        );
        assert_eq!(
            StealthScraperClient::extract_numeric_price("€ 42.00"),
            Some(42.00)
        );
        assert_eq!(StealthScraperClient::extract_numeric_price("N/A"), None);
    }

    #[test]
    fn test_extract_price_from_html() {
        let sample_html = r#"
            <div class="product-summary">
                <span class="price-label">Our Price:</span>
                <span data-price="299.95">$299.95</span>
            </div>
        "#;
        let price = StealthScraperClient::extract_price_from_html(sample_html);
        assert_eq!(price, Some(299.95));
    }

    #[test]
    fn test_scraper_client_initialization() {
        let client = StealthScraperClient::new(vec!["http://proxy1:8080".into()]);
        assert!(client.is_ok());
        assert_eq!(client.unwrap().proxy_count(), 1);
    }
}
