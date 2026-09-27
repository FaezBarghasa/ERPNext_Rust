use rust_decimal::Decimal;

/// Computes weighted lead priority score (0..100) based on deal size and engagement.
#[must_use]
pub fn calculate_lead_score(
    engagement_count: u32,
    estimated_deal_value: Decimal,
    source_weight: Decimal,
) -> Decimal {
    let engagement_score = Decimal::from(engagement_count.min(10)) * Decimal::from(5);
    let value_score = (estimated_deal_value / Decimal::from(1000)).min(Decimal::from(30));
    let source_score = source_weight.min(Decimal::from(20));

    let total = engagement_score + value_score + source_score;
    total.min(Decimal::from(100))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_lead_score_bounds() {
        let score = calculate_lead_score(5, dec!(20000.0), dec!(10.0));
        assert!(score >= dec!(0.0) && score <= dec!(100.0));
    }
}
