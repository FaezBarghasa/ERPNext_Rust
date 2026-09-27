pub const STAGES: &[&str] = &["Lead","Opportunity","Quotation","Sales Order","Customer"];
pub fn next_stage(s: &str) -> Option<&str> { STAGES.iter().position(|&x| x==s).and_then(|i| STAGES.get(i+1).copied()) }
#[cfg(test)] mod t { use super::*; #[test] fn adv(){ assert_eq!(next_stage("Lead"), Some("Opportunity")); } }
