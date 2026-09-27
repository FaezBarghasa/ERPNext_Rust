/// Gross = Base + HRA + DA; PF = min(12% Base, 1800) (Stage 4.5.1). Minor units.
pub fn gross_cents(base: i64, hra: i64, da: i64) -> i64 { base + hra + da }
pub fn provident_fund(base: i64) -> i64 { ((base as f64 * 0.12) as i64).min(1800) }
#[cfg(test)] mod t { use super::*;
    #[test] fn pf_cap(){ assert_eq!(provident_fund(100_000), 1800); assert_eq!(gross_cents(100,50,25),175); } }
