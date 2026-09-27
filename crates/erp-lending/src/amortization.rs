/// EMI reducing-balance: E = P*r*(1+r)^n / ((1+r)^n - 1) (Stage 4.6.3). Cents.
pub fn emi_cents(principal: i64, annual_pct: f64, months: u32) -> i64 {
    if months == 0 { return 0; } if annual_pct == 0.0 { return principal / months as i64; }
    let r = annual_pct / 1200.0; let f = (1.0 + r).powi(months as i32);
    (principal as f64 * r * f / (f - 1.0)).round() as i64
}
#[cfg(test)] mod t { use super::*; #[test] fn emi(){ assert!(emi_cents(100000, 12.0, 12) > 8000); } }
