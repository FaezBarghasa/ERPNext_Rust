use frappe_net::{provision_tenant, ConnectionPoolManager, TenantId};
use std::time::Duration;

#[tokio::test]
async fn test_tenant_database_isolation() {
    let pool_mgr = ConnectionPoolManager::new(Duration::from_secs(60));

    let tenant_a = TenantId("alpha-corp".to_string());
    let tenant_b = TenantId("beta-industries".to_string());

    // 1. Provision separate tenants
    let client_a = provision_tenant(&pool_mgr, &tenant_a.0)
        .await
        .expect("Provision A failed");
    let client_b = provision_tenant(&pool_mgr, &tenant_b.0)
        .await
        .expect("Provision B failed");

    // 2. Insert records with identical IDs on same table name in both tenants
    client_a
        .query("CREATE customer:acme SET name = 'Customer in Alpha Corp', balance = 1000;")
        .await
        .unwrap()
        .check()
        .unwrap();

    client_b
        .query("CREATE customer:acme SET name = 'Customer in Beta Corp', balance = 2000;")
        .await
        .unwrap()
        .check()
        .unwrap();

    // 3. Query tenant A and assert tenant B data is not visible
    let mut res_a = client_a
        .query("SELECT * FROM customer:acme;")
        .await
        .unwrap()
        .check()
        .unwrap();
    let rows_a: Option<serde_json::Value> = res_a.take(0).unwrap();
    let row_a = rows_a.expect("Customer in A not found");
    assert_eq!(row_a["name"], "Customer in Alpha Corp");
    assert_eq!(row_a["balance"], 1000);

    // 4. Query tenant B and assert tenant A data is not visible
    let mut res_b = client_b
        .query("SELECT * FROM customer:acme;")
        .await
        .unwrap()
        .check()
        .unwrap();
    let rows_b: Option<serde_json::Value> = res_b.take(0).unwrap();
    let row_b = rows_b.expect("Customer in B not found");
    assert_eq!(row_b["name"], "Customer in Beta Corp");
    assert_eq!(row_b["balance"], 2000);
}
