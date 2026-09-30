//! Generic Asynchronous SurrealDB Repository Subsystem (`frappe-storage::repository`).
//!
//! Provides transactional, type-safe entity persistence, query builders,
//! and schema-agnostic CRUD operations for all enterprise domain models.

use serde::Serialize;
use serde::de::DeserializeOwned;
use surrealdb::{Connection, Surreal};
use thiserror::Error;

/// Repository operation errors.
#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] surrealdb::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Entity not found: {0}")]
    NotFound(String),
}

/// Universal entity repository wrapping a SurrealDB client connection.
#[derive(Clone, Debug)]
pub struct SurrealRepository<C: Connection> {
    db: Surreal<C>,
}

impl<C: Connection> SurrealRepository<C> {
    /// Creates a new repository with the given SurrealDB connection.
    #[must_use]
    pub fn new(db: Surreal<C>) -> Self {
        Self { db }
    }

    /// Access the underlying SurrealDB handle.
    #[must_use]
    pub fn client(&self) -> &Surreal<C> {
        &self.db
    }

    /// Creates or updates a record at `table:id` with the provided serializable entity.
    pub async fn upsert<T: Serialize + Send + Sync>(
        &self,
        table: &str,
        id: &str,
        data: &T,
    ) -> Result<(), RepositoryError> {
        let val = serde_json::to_value(data)?;
        let clean_id = id.replace([':', ' ', '-'], "_");
        let sql = format!("UPSERT {table}:{clean_id} MERGE $payload;");
        self.db.query(sql).bind(("payload", val)).await?.check()?;
        Ok(())
    }

    /// Fetches a single entity from `table:id`.
    pub async fn select_one<T: DeserializeOwned + Send + Sync>(
        &self,
        table: &str,
        id: &str,
    ) -> Result<Option<T>, RepositoryError> {
        let clean_id = id.replace([':', ' ', '-'], "_");
        let sql = format!("SELECT * FROM {table}:{clean_id};");
        let mut response = self.db.query(sql).await?.check()?;
        let records: Vec<serde_json::Value> = response.take(0)?;
        if let Some(val) = records.into_iter().next() {
            let entity = serde_json::from_value(val)?;
            Ok(Some(entity))
        } else {
            Ok(None)
        }
    }

    /// Fetches all records from the specified table.
    pub async fn select_all<T: DeserializeOwned + Send + Sync>(
        &self,
        table: &str,
    ) -> Result<Vec<T>, RepositoryError> {
        let sql = format!("SELECT * FROM {table};");
        let mut response = self.db.query(sql).await?.check()?;
        let records: Vec<serde_json::Value> = response.take(0)?;
        let mut result = Vec::with_capacity(records.len());
        for val in records {
            if let Ok(entity) = serde_json::from_value(val) {
                result.push(entity);
            }
        }
        Ok(result)
    }

    /// Fetches records matching a custom WHERE clause.
    pub async fn query_where<T: DeserializeOwned + Send + Sync>(
        &self,
        table: &str,
        where_clause: &str,
    ) -> Result<Vec<T>, RepositoryError> {
        let sql = format!("SELECT * FROM {table} WHERE {where_clause};");
        let mut response = self.db.query(sql).await?.check()?;
        let records: Vec<serde_json::Value> = response.take(0)?;
        let mut result = Vec::with_capacity(records.len());
        for val in records {
            if let Ok(entity) = serde_json::from_value(val) {
                result.push(entity);
            }
        }
        Ok(result)
    }

    /// Deletes a single record at `table:id`.
    pub async fn delete(&self, table: &str, id: &str) -> Result<(), RepositoryError> {
        let clean_id = id.replace([':', ' ', '-'], "_");
        let sql = format!("DELETE {table}:{clean_id};");
        self.db.query(sql).await?.check()?;
        Ok(())
    }

    /// Counts records in a given table.
    pub async fn count(&self, table: &str) -> Result<u64, RepositoryError> {
        let sql = format!("SELECT count() FROM {table} GROUP ALL;");
        let mut response = self.db.query(sql).await?.check()?;
        let res: Option<serde_json::Value> = response.take(0)?;
        if let Some(val) = res
            && let Some(count) = val.get("count").and_then(serde_json::Value::as_u64)
        {
            return Ok(count);
        }
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use surrealdb::engine::local::Mem;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    struct MockCustomer {
        id: String,
        name: String,
        email: String,
    }

    #[tokio::test]
    async fn test_surreal_repository_crud() {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test_ns").use_db("test_db").await.unwrap();
        let repo = SurrealRepository::new(db);

        let customer = MockCustomer {
            id: "cust_100".to_string(),
            name: "Faez Barghasa".to_string(),
            email: "faez@example.com".to_string(),
        };

        // 1. Upsert
        repo.upsert("customers", &customer.id, &customer)
            .await
            .expect("Upsert");

        // 2. Select One
        let fetched: Option<MockCustomer> = repo
            .select_one("customers", &customer.id)
            .await
            .expect("Select one");
        assert!(fetched.is_some());
        let fetched_val = fetched.unwrap();
        assert_eq!(fetched_val.name, "Faez Barghasa");
        assert_eq!(fetched_val.email, "faez@example.com");

        // 3. Select All
        let all: Vec<MockCustomer> = repo.select_all("customers").await.expect("Select all");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "Faez Barghasa");

        // 4. Query Where
        let filtered: Vec<MockCustomer> = repo
            .query_where("customers", "email = 'faez@example.com'")
            .await
            .expect("Query where");
        assert_eq!(filtered.len(), 1);

        // 5. Delete
        repo.delete("customers", &customer.id)
            .await
            .expect("Delete");
        let after_delete: Option<MockCustomer> = repo
            .select_one("customers", &customer.id)
            .await
            .expect("Select after delete");
        assert_eq!(after_delete, None);
    }
}
