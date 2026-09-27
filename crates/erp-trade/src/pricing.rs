#[derive(Debug)] pub struct PriceRule { pub min_qty: i64, pub pct_off: f64, pub flat_cents_off: i64 }
/// Best applicable rule by min_qty; returns unit price in cents.
pub fn apply_rules(list_cents: i64, qty: i64, rules: &[PriceRule]) -> i64 {
    let mut best: Option<&PriceRule> = None;
    for r in rules { if qty >= r.min_qty && best.map(|b| r.min_qty > b.min_qty).unwrap_or(true) { best = Some(r); } }
    match best { None => list_cents,
        Some(r) => { let p = (list_cents as f64 * (1.0 - r.pct_off / 100.0)) as i64 - r.flat_cents_off; p.max(0) } }
}
#[cfg(test)] mod t { use super::*; #[test] fn rule(){ let r = vec![PriceRule{min_qty:10,pct_off:10.0,flat_cents_off:0}];
    assert_eq!(apply_rules(1000, 10, &r), 900); assert_eq!(apply_rules(1000, 5, &r), 1000); } }
