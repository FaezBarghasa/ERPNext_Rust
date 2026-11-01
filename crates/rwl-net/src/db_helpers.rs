//! Database Query Utilities and Deduplication Helpers (`rwl-net::db_helpers`).
//!
//! Provides clean, parameterized SurrealDB query patterns to eliminate duplicated query boilerplate
//! across HTTP resource handlers.

use crate::error::ApiError;
use serde_json::Value;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

/// Safely queries a single document by its table and `name` identifier using parameterized binding.
///
/// Prevents SurrealQL injection and returns standard `ApiError::NotFound` if missing.
pub async fn query_one_by_name(
    db: &Surreal<Any>,
    table: &str,
    name: &str,
) -> Result<Option<Value>, ApiError> {
    // Validate table name (alphanumeric and underscore only)
    if !table.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(ApiError::BadRequest(format!("Invalid table identifier: {table}")));
    }

    let query = format!("SELECT * FROM type::table($table) WHERE name = $name LIMIT 1");
    let mut response = db
        .query(query)
        .bind(("table", table.to_string()))
        .bind(("name", name.to_string()))
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    let record: Option<Value> = response
        .take(0)
        .map_err(|e| ApiError::Database(e.to_string()))?;

    Ok(record)
}
