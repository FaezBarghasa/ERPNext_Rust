pub fn deadline_minutes(priority: &str, open_min: u64) -> u64 {
    open_min + match priority { "Critical" => 60, "High" => 240, "Medium" => 1440, _ => 4320 }
}
#[cfg(test)] mod t { use super::*; #[test] fn sla(){ assert_eq!(deadline_minutes("Critical", 0), 60); } }
