#[derive(Debug, Clone)] pub struct GlLine { pub account: String, pub debit: i64, pub credit: i64, pub company: String, pub cost_center: String }
/// Zero-balance invariant: sum(debit) - sum(credit) == 0 (Stage 4.1.1). Minor units (cents).
pub fn verify_balanced(lines: &[GlLine]) -> bool { lines.iter().map(|l| l.debit - l.credit).sum::<i64>() == 0 }
pub fn insert_stmt(table: &str, l: &GlLine) -> String {
    format!("CREATE {} SET account='{}', debit={}, credit={}, company='{}', cost_center='{}';", table, l.account, l.debit, l.credit, l.company, l.cost_center)
}
#[cfg(test)] mod t { use super::*;
    #[test] fn balanced() { let l = vec![GlLine{account:"a".into(),debit:100,credit:0,company:"c".into(),cost_center:"m".into()},GlLine{account:"b".into(),debit:0,credit:100,company:"c".into(),cost_center:"m".into()}];
        assert!(verify_balanced(&l)); assert!(!verify_balanced(&l[..1])); }
}
