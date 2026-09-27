#[derive(Debug, Clone)] pub struct TaxLine { pub rate_pct: f64, pub compound: bool }
// DAG calc: sequential lines; compound applies on running total (Stage 5.1).
pub fn calc_taxes(net_cents: i64, lines: &[TaxLine]) -> i64 {
    let mut base = net_cents as f64; let mut total_tax = 0.0;
    for l in lines { let t = if l.compound { base * l.rate_pct/100.0 } else { net_cents as f64 * l.rate_pct/100.0 };
        total_tax += t; base += if l.compound { t } else { 0.0 }; } total_tax.round() as i64
}
// ZATCA hash chain: Hash_n = SHA256(payload_n || Hash_{n-1}) — placeholder FNV demo (Stage 5.2).
pub fn chain_hash(prev: u64, payload: &str) -> u64 {
    let mut h = prev ^ 0xcbf29ce484222325u64;
    for b in payload.bytes() { h ^= b as u64; h = h.wrapping_mul(0x100000001b3); } h
}
#[cfg(test)] mod t { use super::*;
    #[test] fn compound(){ let t = calc_taxes(10000, &[TaxLine{rate_pct:10.0,compound:false},TaxLine{rate_pct:5.0,compound:true}]); assert!(t >= 1500); }
    #[test] fn chain(){ assert_ne!(chain_hash(0,"a"), chain_hash(0,"b")); } }
