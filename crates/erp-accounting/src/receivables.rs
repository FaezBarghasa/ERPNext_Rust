/// FIFO payment allocation edge + aging buckets (Stage 4.1.3).
pub fn settle_edge(pay: &str, inv: &str, amount_cents: i64) -> String {
    format!("RELATE tab_payment_entry:{}->settles->tab_sales_invoice:{} SET allocated_amount = {:.2};", pay, inv, amount_cents as f64 / 100.0)
}
pub fn aging_bucket(days_overdue: i64) -> &'static str {
    if days_overdue <= 30 { "0-30" } else if days_overdue <= 60 { "31-60" } else if days_overdue <= 90 { "61-90" } else { "90+" }
}
#[cfg(test)] mod t { use super::*; #[test] fn buckets(){ assert_eq!(aging_bucket(0),"0-30"); assert_eq!(aging_bucket(91),"90+"); } }
