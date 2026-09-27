use rust_decimal::Decimal;

/// Distributes landed cost charges across item valuation amounts proportionally.
#[must_use]
pub fn distribute_landed_cost(item_values: &[Decimal], total_charges: Decimal) -> Vec<Decimal> {
    let total_value: Decimal = item_values.iter().sum();
    if total_value == Decimal::ZERO {
        return vec![Decimal::ZERO; item_values.len()];
    }

    let mut distributed: Vec<Decimal> = item_values
        .iter()
        .map(|v| (v * total_charges) / total_value)
        .collect();

    let sum_distributed: Decimal = distributed.iter().sum();
    let diff = total_charges - sum_distributed;

    if let Some(last) = distributed.last_mut() {
        *last += diff;
    }

    distributed
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_landed_cost_distribution() {
        let values = vec![dec!(60.0), dec!(40.0)];
        let distributed = distribute_landed_cost(&values, dec!(100.0));
        assert_eq!(distributed[0], dec!(60.0));
        assert_eq!(distributed[1], dec!(40.0));
        let total: Decimal = distributed.iter().sum();
        assert_eq!(total, dec!(100.0));
    }
}
