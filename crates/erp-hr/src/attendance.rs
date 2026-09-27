/// Point-in-polygon (Jordan curve) geo-fence check (Stage 4.5.2).
pub fn inside_fence(pt: (f64,f64), poly: &[(f64,f64)]) -> bool {
    let (x, y) = pt; let mut inside = false; let n = poly.len();
    let mut j = n.saturating_sub(1);
    for i in 0..n { let (xi, yi) = poly[i]; let (xj, yj) = poly[j];
        if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi { inside = !inside; } j = i; }
    inside
}
#[cfg(test)] mod t { use super::*;
    #[test] fn fence(){ let sq = [(0.0,0.0),(1.0,0.0),(1.0,1.0),(0.0,1.0)];
        assert!(inside_fence((0.5,0.5), &sq)); assert!(!inside_fence((2.0,2.0), &sq)); } }
