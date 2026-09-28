use rust_decimal::Decimal;
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DecLine {
    pub account: String,
    pub debit: Decimal,
    pub credit: Decimal,
}
/// Zero-balance invariant with exact decimal math.
pub fn verify_balanced_dec(lines: &[DecLine]) -> bool {
    lines
        .iter()
        .fold(Decimal::ZERO, |a, l| a + l.debit - l.credit)
        == Decimal::ZERO
}
/// Straight-line depreciation in Decimal.
pub fn straight_line_dec(cost: Decimal, salvage: Decimal, n: usize) -> Vec<Decimal> {
    if n == 0 {
        return vec![];
    }
    let per = (cost - salvage) / Decimal::from(n);
    let mut v = vec![per; n];
    let diff = (cost - salvage) - per * Decimal::from(n);
    if let Some(l) = v.last_mut() {
        *l += diff;
    }
    v
}
#[cfg(test)]
mod t {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn dec_balanced() {
        let d = |s: &str| Decimal::from_str(s).unwrap();
        let l = vec![
            DecLine {
                account: "a".into(),
                debit: d("100.10"),
                credit: d("0"),
            },
            DecLine {
                account: "b".into(),
                debit: d("0"),
                credit: d("100.10"),
            },
        ];
        assert!(verify_balanced_dec(&l));
    }
    #[test]
    fn no_float_drift() {
        // 0.1 + 0.2 == 0.3 exactly in Decimal, unlike f64
        let d = |s: &str| Decimal::from_str(s).unwrap();
        assert_eq!(d("0.1") + d("0.2"), d("0.3"));
    }
}
