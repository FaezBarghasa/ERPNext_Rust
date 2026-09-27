/// Straight-line depreciation in minor units (Stage 4.1.4).
pub fn straight_line(cost: i64, salvage: i64, life_periods: usize) -> Vec<i64> {
    if life_periods == 0 { return vec![]; }
    let per = (cost - salvage) / life_periods as i64; let mut v = vec![per; life_periods];
    let diff = (cost - salvage) - per * life_periods as i64; // remainder to last period
    if let Some(last) = v.last_mut() { *last += diff; } v
}
#[cfg(test)] mod t { use super::*; #[test] fn sl(){ let v = straight_line(1000,0,4); assert_eq!(v.iter().sum::<i64>(),1000); } }
