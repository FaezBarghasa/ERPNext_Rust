use std::collections::HashMap;
/// Multi-level BOM explosion with cycle guard (Stage 4.4.1).
pub fn explode(root: &str, edges: &HashMap<String, Vec<(String, i64)>>, qty: i64) -> HashMap<String, i64> {
    let mut out = HashMap::new(); let mut stack = vec![(root.to_string(), qty)]; let mut guard = 0;
    while let Some((n, q)) = stack.pop() {
        guard += 1; if guard > 100_000 { break; }
        match edges.get(&n) { None => *out.entry(n).or_insert(0) += q,
            Some(children) => for (c, per) in children { stack.push((c.clone(), q * per)); } }
    } out
}
pub fn requires_edge(bom: &str, item: &str, qty: i64) -> String { format!("RELATE tab_bom:{}->requires->tab_item:{} SET qty = {};", bom, item, qty) }
#[cfg(test)] mod t { use super::*; #[test] fn expl(){ let mut e = HashMap::new();
    e.insert("drone".into(), vec![("motor".into(),4),("avionics".into(),1)]);
    e.insert("avionics".into(), vec![("chip".into(),2)]);
    let r = explode("drone",&e,1); assert_eq!(r["motor"],4); assert_eq!(r["chip"],2); } }
