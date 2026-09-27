pub fn contains_edge(parent: &str, child: &str) -> String { format!("RELATE tab_warehouse:{}->contains->tab_warehouse:{};", parent, child) }
#[cfg(test)] mod t { use super::*; #[test] fn e(){ assert!(contains_edge("hq","aisle_1").contains("contains")); } }
