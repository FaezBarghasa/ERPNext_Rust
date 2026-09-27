use std::collections::HashMap;
pub fn progress(done: usize, total: usize) -> f64 { if total==0 {0.0} else {done as f64/total as f64*100.0} }
pub fn cert_eligible(scores: &HashMap<String,f64>, pass: f64) -> bool { !scores.is_empty() && scores.values().all(|&s| s>=pass) }
#[cfg(test)] mod t { use super::*; #[test] fn p(){ assert_eq!(progress(1,2),50.0); } }
