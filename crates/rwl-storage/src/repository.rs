//! Generic Asynchronous SurrealDB Repository Subsystem (`rwl-storage::repository`).
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
    #[error("Invalid SQL identifier: {0}")]
    InvalidIdentifier(String),
}

/// Validates that table names and entity IDs contain only safe alphanumeric and underscore characters.
pub fn validate_identifier(ident: &str) -> Result<(), RepositoryError> {
    if ident.is_empty() || !ident.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(RepositoryError::InvalidIdentifier(ident.to_string()));
    }
    Ok(())
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
        validate_identifier(table)?;
        let clean_id = id.replace([':', ' ', '-'], "_");
        validate_identifier(&clean_id)?;

        let val = serde_json::to_value(data)?;
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
        validate_identifier(table)?;
        let clean_id = id.replace([':', ' ', '-'], "_");
        validate_identifier(&clean_id)?;

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
        validate_identifier(table)?;
        let sql = format!("SELECT * FROM {table};");
        let mut response = self.db.query(sql).await?.check()?;
        let records: Vec<serde_json::Value> = response.take(0)?;
        let mut result = Vec::with_capacity(records.len());
        for val in records {
            let entity = serde_json::from_value(val).map_err(RepositoryError::Serialization)?;
            result.push(entity);
        }
        Ok(result)
    }

    /// Fetches records matching a custom WHERE clause.
    pub async fn query_where<T: DeserializeOwned + Send + Sync>(
        &self,
        table: &str,
        where_clause: &str,
    ) -> Result<Vec<T>, RepositoryError> {
        validate_identifier(table)?;
        if where_clause.contains(';')
            || where_clause.contains('\0')
            || where_clause.contains("--")
            || where_clause.contains("/*")
        {
            return Err(RepositoryError::InvalidIdentifier(
                "Unsafe characters or SQL comments prohibited in WHERE clause".into(),
            ));
        }

        let sql = format!("SELECT * FROM {table} WHERE {where_clause};");
        let mut response = self.db.query(sql).await?.check()?;
        let records: Vec<serde_json::Value> = response.take(0)?;
        let mut result = Vec::with_capacity(records.len());
        for val in records {
            let entity = serde_json::from_value(val).map_err(RepositoryError::Serialization)?;
            result.push(entity);
        }
        Ok(result)
    }

    /// Fetches records matching a parameterized WHERE clause with typed bindings.
    pub async fn query_where_bound<T: DeserializeOwned + Send + Sync>(
        &self,
        table: &str,
        where_clause: &str,
        bindings: &[(&str, serde_json::Value)],
    ) -> Result<Vec<T>, RepositoryError> {
        validate_identifier(table)?;
        if where_clause.contains(';')
            || where_clause.contains('\0')
            || where_clause.contains("--")
            || where_clause.contains("/*")
        {
            return Err(RepositoryError::InvalidIdentifier(
                "Unsafe characters or SQL comments prohibited in WHERE clause".into(),
            ));
        }

        let sql = format!("SELECT * FROM {table} WHERE {where_clause};");
        let mut query = self.db.query(sql);
        for (k, v) in bindings {
            query = query.bind((*k, v.clone()));
        }
        let mut response = query.await?.check()?;
        let records: Vec<serde_json::Value> = response.take(0)?;
        let mut result = Vec::with_capacity(records.len());
        for val in records {
            let entity = serde_json::from_value(val).map_err(RepositoryError::Serialization)?;
            result.push(entity);
        }
        Ok(result)
    }

    /// Deletes a single record at `table:id`.
    pub async fn delete(&self, table: &str, id: &str) -> Result<(), RepositoryError> {
        validate_identifier(table)?;
        let clean_id = id.replace([':', ' ', '-'], "_");
        validate_identifier(&clean_id)?;

        let sql = format!("DELETE {table}:{clean_id};");
        self.db.query(sql).await?.check()?;
        Ok(())
    }

    /// Counts records in a given table.
    pub async fn count(&self, table: &str) -> Result<u64, RepositoryError> {
        validate_identifier(table)?;
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

    #[tokio::test]
    async fn test_surreal_repository_injection_defense() {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("sec_ns").use_db("sec_db").await.unwrap();
        let repo = SurrealRepository::new(db);

        // 1. Malicious table identifier with spaces or special characters
        let res = repo.count("customers; DROP TABLE user").await;
        assert!(matches!(res, Err(RepositoryError::InvalidIdentifier(_))));

        // 2. Semicolon query chaining in WHERE clause rejected
        let res2: Result<Vec<MockCustomer>, RepositoryError> = repo
            .query_where("customers", "1 = 1; DELETE customers")
            .await;
        assert!(matches!(res2, Err(RepositoryError::InvalidIdentifier(_))));

        // 3. Bound query with parameter binding works securely
        let customer = MockCustomer {
            id: "cust_200".to_string(),
            name: "Alice Security".to_string(),
            email: "alice@defense.rs".to_string(),
        };
        repo.upsert("customers", &customer.id, &customer)
            .await
            .unwrap();

        let bound: Vec<MockCustomer> = repo
            .query_where_bound(
                "customers",
                "email = $email",
                &[(
                    "email",
                    serde_json::Value::String("alice@defense.rs".into()),
                )],
            )
            .await
            .unwrap();
        assert_eq!(bound.len(), 1);
        assert_eq!(bound[0].name, "Alice Security");

        // 4. Comment injection and null byte rejection
        let res_comment: Result<Vec<MockCustomer>, RepositoryError> = repo
            .query_where("customers", "email = 'x' -- comment")
            .await;
        assert!(matches!(
            res_comment,
            Err(RepositoryError::InvalidIdentifier(_))
        ));

        let res_block_comment: Result<Vec<MockCustomer>, RepositoryError> = repo
            .query_where("customers", "email = 'x' /* block */")
            .await;
        assert!(matches!(
            res_block_comment,
            Err(RepositoryError::InvalidIdentifier(_))
        ));

        let res_null_byte: Result<Vec<MockCustomer>, RepositoryError> =
            repo.query_where("customers", "email = 'x\0'").await;
        assert!(matches!(
            res_null_byte,
            Err(RepositoryError::InvalidIdentifier(_))
        ));

        // 5. Schema mismatch in select_all returns Serialization error
        repo.client()
            .query("CREATE customers:corrupt CONTENT { id: 'corrupt', name: 12345, email: true };")
            .await
            .unwrap();
        let res_corrupt: Result<Vec<MockCustomer>, RepositoryError> =
            repo.select_all("customers").await;
        assert!(matches!(
            res_corrupt,
            Err(RepositoryError::Serialization(_))
        ));
    }
}
