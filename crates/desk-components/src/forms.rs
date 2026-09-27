/// depends_on evaluator stub (Stage 6.1.2): "field==value" conjunctions with &&.
pub fn eval_depends(expr: &str, values: &std::collections::HashMap<String,String>) -> bool {
    if expr.trim().is_empty() { return true; }
    expr.split("&&").all(|c| { let c = c.trim(); match c.split_once("==") {
        Some((k,v)) => values.get(k.trim()).map(|x| x==v.trim().trim_matches('"')).unwrap_or(false),
        None => false } })
}
#[cfg(test)] mod t { use super::*; use std::collections::HashMap;
    #[test] fn dep(){ let mut m = HashMap::new(); m.insert("status".into(),"Open".into());
        assert!(eval_depends("status==Open",&m)); assert!(!eval_depends("status==Closed",&m)); } }
