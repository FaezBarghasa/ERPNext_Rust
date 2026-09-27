/// FEFO batch pick: reject expired (Stage 4.3.3).
#[derive(Debug, Clone)] pub struct Batch { pub id: String, pub expiry_day: i64, pub qty: i64 }
pub fn pick_fefo(batches: &mut Vec<Batch>, today: i64, need: i64) -> i64 {
    batches.retain(|b| b.expiry_day >= today);
    batches.sort_by_key(|b| b.expiry_day);
    let mut left = need; let mut got = 0;
    for b in batches.iter_mut() { let t = left.min(b.qty); b.qty -= t; left -= t; got += t; if left==0 {break;} }
    got
}
#[cfg(test)] mod t { use super::*;
    #[test] fn fefo(){ let mut b = vec![Batch{id:"a".into(),expiry_day:10,qty:5},Batch{id:"x".into(),expiry_day:1,qty:99}];
        assert_eq!(pick_fefo(&mut b, 5, 5), 5); assert!(b.iter().all(|x| x.id != "x")); }
}
