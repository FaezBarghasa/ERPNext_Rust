//! Embedded SurrealDB (in-memory) tenant backend (Stage 2.2.1).
use surrealdb::{Surreal, engine::local::Mem};

/// Open an in-memory instance, create NS+DB, and return the handle.
pub async fn open_tenant(
    tenant_ns: &str,
    db: &str,
) -> surrealdb::Result<Surreal<surrealdb::engine::local::Db>> {
    let s = Surreal::new::<Mem>(()).await?;
    s.use_ns(tenant_ns).use_db(db).await?;
    Ok(s)
}

#[cfg(test)]
mod t {
    use super::*;
    #[tokio::test]
    async fn ns_db_roundtrip() {
        let db = open_tenant("tenant_demo", "site_production").await.unwrap();
        db.query("CREATE tab_probe SET x = 1")
            .await
            .unwrap()
            .check()
            .unwrap();
        let mut r = db
            .query("SELECT * FROM tab_probe")
            .await
            .unwrap()
            .check()
            .unwrap();
        let rows: Vec<serde_json::Value> = r.take(0).unwrap();
        assert_eq!(rows.len(), 1);
    }
}
