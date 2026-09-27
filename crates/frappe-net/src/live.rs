pub use frappe_storage::LiveDiff;
/// Register LIVE SELECT for a doc view; kill on navigate-away (Stage 2.1.3).
pub fn live_query(tenant_ns: &str, table: &str, id: &str) -> String { format!("LIVE SELECT * FROM {}:{} WHERE $tenant = '{}';", table, id, tenant_ns) }
#[cfg(test)] mod t { use super::*; #[test] fn q(){ assert!(live_query("ns","tab_si","1").starts_with("LIVE SELECT")); } }
