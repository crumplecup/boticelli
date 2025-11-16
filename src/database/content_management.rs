//! Content management for generated content tables.
//!
//! Provides functions for querying, updating, and managing content
//! in dynamically created generation tables.

use crate::{BoticelliResult, DatabaseError, DatabaseErrorKind};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use serde_json::Value as JsonValue;

/// List generated content from a table.
///
/// Queries a dynamically named content table and returns results as JSON.
/// Supports filtering by review_status and limiting results.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `table_name` - Name of the content table
/// * `status_filter` - Optional review status filter ("pending", "approved", "rejected")
/// * `limit` - Maximum number of results to return
///
/// # Returns
///
/// Vector of JSON objects representing table rows
pub fn list_content(
    conn: &mut PgConnection,
    table_name: &str,
    status_filter: Option<&str>,
    limit: usize,
) -> BoticelliResult<Vec<JsonValue>> {
    // Build query dynamically
    let mut query = format!(
        "SELECT row_to_json(t) FROM (SELECT * FROM {} WHERE 1=1",
        table_name
    );

    if let Some(status) = status_filter {
        query.push_str(&format!(" AND review_status = '{}'", status));
    }

    query.push_str(" ORDER BY generated_at DESC");
    query.push_str(&format!(" LIMIT {}", limit));
    query.push_str(") t");

    tracing::debug!(sql = %query, "Listing content");

    // Execute query and collect results
    let results: Vec<String> = diesel::sql_query(&query)
        .load::<StringRow>(conn)
        .map_err(|e| DatabaseError::new(DatabaseErrorKind::Query(e.to_string())))?
        .into_iter()
        .map(|row| row.row_to_json)
        .collect();

    // Parse JSON strings
    let json_results: Result<Vec<JsonValue>, _> = results
        .iter()
        .map(|s| serde_json::from_str(s))
        .collect();

    json_results.map_err(|e| DatabaseError::new(DatabaseErrorKind::Query(e.to_string())).into())
}

/// Get a specific content item by ID.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `table_name` - Name of the content table
/// * `id` - Content ID
///
/// # Returns
///
/// JSON object representing the row, or error if not found
pub fn get_content_by_id(
    conn: &mut PgConnection,
    table_name: &str,
    id: i64,
) -> BoticelliResult<JsonValue> {
    let query = format!(
        "SELECT row_to_json(t) FROM (SELECT * FROM {} WHERE id = {}) t",
        table_name, id
    );

    tracing::debug!(sql = %query, "Getting content by ID");

    let result: String = diesel::sql_query(&query)
        .get_result::<StringRow>(conn)
        .map_err(|e| DatabaseError::new(DatabaseErrorKind::Query(e.to_string())))?
        .row_to_json;

    serde_json::from_str(&result)
        .map_err(|e| DatabaseError::new(DatabaseErrorKind::Query(e.to_string())).into())
}

/// Update tags and rating for a content item.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `table_name` - Name of the content table
/// * `id` - Content ID
/// * `tags` - Optional tags to set (replaces existing)
/// * `rating` - Optional rating (1-5)
pub fn update_content_metadata(
    conn: &mut PgConnection,
    table_name: &str,
    id: i64,
    tags: Option<&[String]>,
    rating: Option<i32>,
) -> BoticelliResult<()> {
    let mut updates = Vec::new();

    if let Some(tag_list) = tags {
        let tags_sql = if tag_list.is_empty() {
            "NULL".to_string()
        } else {
            let escaped: Vec<String> = tag_list
                .iter()
                .map(|t| format!("'{}'", t.replace('\'', "''")))
                .collect();
            format!("ARRAY[{}]", escaped.join(", "))
        };
        updates.push(format!("tags = {}", tags_sql));
    }

    if let Some(r) = rating {
        if !(1..=5).contains(&r) {
            return Err(DatabaseError::new(DatabaseErrorKind::Query(
                "Rating must be between 1 and 5".to_string(),
            ))
            .into());
        }
        updates.push(format!("rating = {}", r));
    }

    if updates.is_empty() {
        return Ok(());
    }

    let query = format!(
        "UPDATE {} SET {} WHERE id = {}",
        table_name,
        updates.join(", "),
        id
    );

    tracing::debug!(sql = %query, "Updating content metadata");

    diesel::sql_query(&query)
        .execute(conn)
        .map_err(|e| DatabaseError::new(DatabaseErrorKind::Query(e.to_string())))?;

    Ok(())
}

/// Update review status for a content item.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `table_name` - Name of the content table
/// * `id` - Content ID
/// * `status` - New status ("pending", "approved", "rejected")
pub fn update_review_status(
    conn: &mut PgConnection,
    table_name: &str,
    id: i64,
    status: &str,
) -> BoticelliResult<()> {
    // Validate status
    if !["pending", "approved", "rejected"].contains(&status) {
        return Err(DatabaseError::new(DatabaseErrorKind::Query(
            "Status must be 'pending', 'approved', or 'rejected'".to_string(),
        ))
        .into());
    }

    let query = format!(
        "UPDATE {} SET review_status = '{}' WHERE id = {}",
        table_name, status, id
    );

    tracing::debug!(sql = %query, "Updating review status");

    diesel::sql_query(&query)
        .execute(conn)
        .map_err(|e| DatabaseError::new(DatabaseErrorKind::Query(e.to_string())))?;

    Ok(())
}

/// Delete a content item.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `table_name` - Name of the content table
/// * `id` - Content ID
pub fn delete_content(
    conn: &mut PgConnection,
    table_name: &str,
    id: i64,
) -> BoticelliResult<()> {
    let query = format!("DELETE FROM {} WHERE id = {}", table_name, id);

    tracing::debug!(sql = %query, "Deleting content");

    diesel::sql_query(&query)
        .execute(conn)
        .map_err(|e| DatabaseError::new(DatabaseErrorKind::Query(e.to_string())))?;

    Ok(())
}

/// Helper struct for deserializing row_to_json results.
#[derive(QueryableByName)]
struct StringRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    row_to_json: String,
}
