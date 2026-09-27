//! Real-Time Statistical Fraud Guard & Benford's Law Anomaly Detection.

use rust_decimal::Decimal;

pub struct BenfordGuard;

impl BenfordGuard {
    /// Expected probability of leading digit d under Benford's Law: P(d) = log10(1 + 1/d)
    #[must_use]
    pub fn expected_probability(digit: u8) -> f64 {
        if !(1..=9).contains(&digit) {
            return 0.0;
        }
        (1.0 + 1.0 / (digit as f64)).log10()
    }

    /// Computes Chi-Square goodness-of-fit statistic against Benford distribution.
    /// Chi-Square > 15.51 (at df=8, p=0.05) indicates statistically significant anomaly.
    #[must_use]
    pub fn compute_chi_square(amounts: &[Decimal]) -> f64 {
        let mut counts = [0usize; 9];
        let mut total = 0usize;

        for amt in amounts {
            let s = amt.abs().to_string();
            if let Some(first_char) = s.chars().find(|c| c.is_ascii_digit() && *c != '0') {
                if let Some(d) = first_char.to_digit(10) {
                    let digit = d as usize;
                    if (1..=9).contains(&digit) {
                        counts[digit - 1] += 1;
                        total += 1;
                    }
                }
            }
        }

        if total < 30 {
            return 0.0; // Insufficient sample size for reliable chi-square
        }

        let mut chi_square = 0.0;
        for i in 0..9 {
            let observed = counts[i] as f64;
            let expected = (total as f64) * Self::expected_probability((i + 1) as u8);
            if expected > 0.0 {
                chi_square += (observed - expected).powi(2) / expected;
            }
        }

        chi_square
    }

    #[must_use]
    pub fn is_anomalous(amounts: &[Decimal]) -> bool {
        Self::compute_chi_square(amounts) > 15.51
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_benford_law_probabilities() {
        let p1 = BenfordGuard::expected_probability(1);
        assert!((p1 - 0.3010).abs() < 1e-3);
        let p9 = BenfordGuard::expected_probability(9);
        assert!((p9 - 0.0457).abs() < 1e-3);

        // Natural distribution test (approximate Benford sample)
        let mut natural_amounts = Vec::new();
        for _ in 0..30 {
            natural_amounts.push(dec!(125.0));
        }
        for _ in 0..17 {
            natural_amounts.push(dec!(230.0));
        }
        for _ in 0..12 {
            natural_amounts.push(dec!(310.0));
        }
        for _ in 0..10 {
            natural_amounts.push(dec!(450.0));
        }
        for _ in 0..8 {
            natural_amounts.push(dec!(520.0));
        }
        for _ in 0..7 {
            natural_amounts.push(dec!(610.0));
        }
        for _ in 0..6 {
            natural_amounts.push(dec!(790.0));
        }
        for _ in 0..5 {
            natural_amounts.push(dec!(840.0));
        }
        for _ in 0..5 {
            natural_amounts.push(dec!(910.0));
        }

        assert!(!BenfordGuard::is_anomalous(&natural_amounts));

        // Fraudulent synthetic distribution (all starting with 9)
        let fraud_amounts: Vec<Decimal> = (0..100).map(|_| dec!(999.50)).collect();
        assert!(BenfordGuard::is_anomalous(&fraud_amounts));
    }
}
