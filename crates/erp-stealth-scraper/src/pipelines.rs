//! Native ERP Ingestion Pipelines (`erp_stealth_scraper::pipelines`).

use crate::{ScrapedPriceRecord, SupplierCatalogItem};
use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// Auto-generated ERP `Item Price` mutation payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErpItemPriceUpdate {
    pub item_code: CompactString,
    pub price_list: CompactString,
    pub price_list_rate: f64,
    pub currency: CompactString,
    pub competitor_url: CompactString,
    pub valid_from: CompactString,
}

/// Auto-generated ERP `Supplier Quotation` document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErpSupplierQuotationDraft {
    pub quotation_id: CompactString,
    pub supplier: CompactString,
    pub transaction_date: CompactString,
    pub items: Vec<ErpSupplierQuotationItem>,
    pub total_amount: f64,
    pub currency: CompactString,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErpSupplierQuotationItem {
    pub item_code: CompactString,
    pub qty: f64,
    pub rate: f64,
    pub amount: f64,
}

/// Real-time price anomaly alert for sales and procurement managers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PriceAlertEvent {
    pub item_code: CompactString,
    pub internal_price: f64,
    pub competitor_price: f64,
    pub difference_pct: f64,
    pub competitor_url: CompactString,
    pub alert_level: PriceAlertSeverity,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PriceAlertSeverity {
    Info,
    Warning,
    CriticalUnderpricing,
    MajorArbitrageOpportunity,
}

/// Pipeline processor converting raw scraped records into typed ERP transactions.
pub struct ScraperPipelineProcessor;

impl ScraperPipelineProcessor {
    /// Transforms a `ScrapedPriceRecord` into an `ErpItemPriceUpdate`.
    #[must_use]
    pub fn to_item_price_update(
        record: &ScrapedPriceRecord,
        item_code: &str,
        price_list: &str,
    ) -> ErpItemPriceUpdate {
        ErpItemPriceUpdate {
            item_code: item_code.into(),
            price_list: price_list.into(),
            price_list_rate: record.price,
            currency: record.currency.clone(),
            competitor_url: record.url.clone(),
            valid_from: chrono::Utc::now().to_rfc3339().into(),
        }
    }

    /// Evaluates competitor price against internal baseline and emits alert if deviation > threshold.
    #[must_use]
    pub fn evaluate_price_alert(
        item_code: &str,
        internal_price: f64,
        record: &ScrapedPriceRecord,
        threshold_pct: f64,
    ) -> Option<PriceAlertEvent> {
        if internal_price <= 0.0 {
            return None;
        }

        let diff_pct = ((record.price - internal_price) / internal_price) * 100.0;
        if diff_pct.abs() >= threshold_pct {
            let alert_level = if diff_pct <= -20.0 {
                PriceAlertSeverity::CriticalUnderpricing
            } else if diff_pct >= 25.0 {
                PriceAlertSeverity::MajorArbitrageOpportunity
            } else {
                PriceAlertSeverity::Warning
            };

            Some(PriceAlertEvent {
                item_code: item_code.into(),
                internal_price,
                competitor_price: record.price,
                difference_pct: diff_pct,
                competitor_url: record.url.clone(),
                alert_level,
            })
        } else {
            None
        }
    }

    /// Converts scraped supplier catalog items into a draft `Supplier Quotation`.
    #[must_use]
    pub fn to_supplier_quotation(
        supplier_name: &str,
        items: &[SupplierCatalogItem],
    ) -> ErpSupplierQuotationDraft {
        let quotation_items: Vec<ErpSupplierQuotationItem> = items
            .iter()
            .map(|i| ErpSupplierQuotationItem {
                item_code: i.sku.clone(),
                qty: 1.0,
                rate: i.price,
                amount: i.price,
            })
            .collect();

        let total_amount = quotation_items.iter().map(|i| i.amount).sum();
        let currency = items
            .first()
            .map(|i| i.currency.clone())
            .unwrap_or_else(|| "USD".into());

        ErpSupplierQuotationDraft {
            quotation_id: format!("SQ-AUTOSCRAPE-{}", chrono::Utc::now().timestamp()).into(),
            supplier: supplier_name.into(),
            transaction_date: chrono::Utc::now().to_rfc3339().into(),
            items: quotation_items,
            total_amount,
            currency,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_price_alert_detection() {
        let record = ScrapedPriceRecord {
            url: "https://competitor.com/item1".into(),
            price: 70.0,
            currency: "USD".into(),
            sku: Some("SKU-100".into()),
            timestamp_utc: 12345678,
        };

        // Internal price 100.0, competitor 70.0 (-30%) -> CriticalUnderpricing
        let alert =
            ScraperPipelineProcessor::evaluate_price_alert("ITEM-001", 100.0, &record, 10.0);
        assert!(alert.is_some());
        let a = alert.unwrap();
        assert_eq!(a.alert_level, PriceAlertSeverity::CriticalUnderpricing);
        assert_eq!(a.difference_pct, -30.0);
    }
}
