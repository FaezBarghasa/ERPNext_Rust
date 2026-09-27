/// Weighted lead score 0..100 (Stage 4.6.1).
pub fn score(freq: f64, deal_value: f64, source_w: f64) -> f64 { (freq.min(10.0)*5.0 + (deal_value/1000.0).min(30.0) + source_w.min(20.0)).min(100.0) }
#[cfg(test)] mod t { use super::*; #[test] fn s(){ assert!((0.0..=100.0).contains(&score(5.0, 20000.0, 10.0))); } }
