/// Gross-to-net infinite-capacity MRP (Stage 4.4.3).
pub fn net_requirements(gross: i64, on_hand: i64, scheduled: i64) -> i64 { (gross - on_hand - scheduled).max(0) }
/// Finite-capacity: fit demand into discrete time-block capacities; returns backlog.
pub fn finite_schedule(demands: &[i64], capacities: &[i64]) -> (Vec<i64>, i64) {
    let mut out = vec![0; demands.len().max(capacities.len())]; let mut backlog = 0;
    for (i, d) in demands.iter().enumerate() {
        let cap = capacities.get(i).copied().unwrap_or(0);
        let produced = (*d + backlog).min(cap); backlog = *d + backlog - produced; out[i] = produced;
    } (out, backlog)
}
#[cfg(test)] mod t { use super::*;
    #[test] fn net(){ assert_eq!(net_requirements(100,30,20),50); }
    #[test] fn finite(){ let (p,b) = finite_schedule(&[10,10],&[8,8]); assert_eq!(b,4); assert_eq!(p,[8,8]); } }
