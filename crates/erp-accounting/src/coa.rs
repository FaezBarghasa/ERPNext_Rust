/// Chart of Accounts graph rollup (Stage 4.1.2).
pub fn child_edge(parent: &str, child: &str) -> String { format!("RELATE tab_account:{}->child_of->tab_account:{};", child, parent) }
pub fn rollup_query() -> &'static str {
    "SELECT id, name, math::sum(->child_of->tab_gl_entry.debit) - math::sum(->child_of->tab_gl_entry.credit) AS balance FROM tab_account;"
}
#[cfg(test)] mod t { use super::*; #[test] fn edge_ok(){ assert!(child_edge("assets","current").contains("child_of")); } }
