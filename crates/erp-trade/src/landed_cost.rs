/// Distribute charges by valuation amount (Stage 4.2.3).
pub fn by_value(values: &[i64], charges: i64) -> Vec<i64> {
    let total: i64 = values.iter().sum(); if total == 0 { return vec![0; values.len()]; }
    let mut out: Vec<i64> = values.iter().map(|v| v * charges / total).collect();
    let diff = charges - out.iter().sum::<i64>();
    if let Some(last) = out.last_mut() { *last += diff; } out
}
#[cfg(test)] mod t { use super::*; #[test] fn sums(){ let v = by_value(&[60,40],100); assert_eq!(v.iter().sum::<i64>(),100); } }
