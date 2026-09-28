pub mod atp_ctp;
pub mod dom;
pub mod einvoice;
pub mod invoice_matching;
pub mod landed_cost;
pub mod metered;
pub mod pricing;
pub mod sanctions;
pub mod taxes;
pub mod woocommerce_ingest;
pub mod zatca;

pub use atp_ctp::{AtpCtpEngine, InventoryPosition, PromiseAvailability};
pub use dom::{DomRouter, FulfillmentNode, ShippingRateEstimate};
pub use einvoice::{EInvoiceDocument, EInvoiceGenerator, EInvoiceStandard};
pub use invoice_matching::{
    GrnItemLine, InvoiceMatchingEngine, MatchVariance, OcrInvoiceLine, PoItemRef,
    ThreeWayMatchResult,
};
pub use landed_cost::distribute_landed_cost;
pub use metered::{MeteredRatingEngine, RatingModel, UsageEvent};
pub use pricing::{PricingEngine, PricingRule};
pub use sanctions::{SanctionEntry, SanctionsScreener};
pub use taxes::{TaxEngine, TaxLineResult, TaxRow, TaxScheduleResult, TaxType};
pub use woocommerce_ingest::{
    IngestedItem, IngestedSalesInvoice, MigrationSummary, WooCustomer, WooMigrationEngine,
    WooOrder, WooOrderLineItem, WooProduct,
};
pub use zatca::ZatcaPhase2Engine;

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_pricing_rules_volume_breaks() {
        let rules = vec![
            PricingRule {
                name: "Standard Retail".into(),
                customer_group: None,
                item_group: None,
                min_qty: dec!(1.0),
                max_qty: Some(dec!(99.0)),
                priority: 1,
                discount_percentage: dec!(0.0),
                discount_amount: dec!(0.0),
            },
            PricingRule {
                name: "Wholesale Tier 1".into(),
                customer_group: None,
                item_group: None,
                min_qty: dec!(100.0),
                max_qty: None,
                priority: 2,
                discount_percentage: dec!(10.0),
                discount_amount: dec!(0.0),
            },
        ];

        let base_rate = dec!(50.00);

        // 50 units -> No discount (50.00)
        let price_50 = PricingEngine::resolve_price(base_rate, dec!(50.0), None, None, &rules);
        assert_eq!(price_50, dec!(50.00));

        // 150 units -> 10% discount (45.00)
        let price_150 = PricingEngine::resolve_price(base_rate, dec!(150.0), None, None, &rules);
        assert_eq!(price_150, dec!(45.00));
    }

    #[test]
    fn test_compounding_multi_tier_taxes() {
        let base_amount = dec!(100.00);
        let tax_rows = vec![
            // Tax 1: 10% on Net Total -> 10.00
            TaxRow {
                account_head: "2210 - Output VAT 10%".into(),
                rate: dec!(10.0),
                charge_type: TaxType::OnNetTotal,
            },
            // Tax 2: 5% Compounded on previous -> (100 + 10) * 0.05 = 5.50
            TaxRow {
                account_head: "2220 - Surcharge 5%".into(),
                rate: dec!(5.0),
                charge_type: TaxType::CompoundedOnPrevious,
            },
        ];

        let result = TaxEngine::calculate_taxes(base_amount, &tax_rows);

        assert_eq!(result.net_total, dec!(100.00));
        assert_eq!(result.tax_lines.len(), 2);
        assert_eq!(result.tax_lines[0].tax_amount, dec!(10.00));
        assert_eq!(result.tax_lines[1].tax_amount, dec!(5.50));
        assert_eq!(result.total_tax, dec!(15.50));
        assert_eq!(result.grand_total, dec!(115.50));
    }
}
