use rust_decimal::Decimal;

/// Straight-line depreciation schedule generator using fixed-point decimal arithmetic.
#[must_use]
pub fn straight_line_depreciation(
    cost: Decimal,
    salvage: Decimal,
    life_periods: usize,
) -> Vec<Decimal> {
    if life_periods == 0 {
        return vec![];
    }
    let total_depreciable = cost - salvage;
    let n = Decimal::from(life_periods);
    let per_period = total_depreciable / n;

    let mut schedule = vec![per_period; life_periods];
    let distributed: Decimal = per_period * n;
    let remainder = total_depreciable - distributed;

    // Distribute fractional remainder onto last period to ensure exact zero-loss
    if let Some(last) = schedule.last_mut() {
        *last += remainder;
    }

    schedule
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_depreciation_zero_loss() {
        let schedule = straight_line_depreciation(dec!(1000.00), dec!(100.00), 3);
        assert_eq!(schedule.len(), 3);
        let total: Decimal = schedule.iter().sum();
        assert_eq!(total, dec!(900.00));
    }
}
