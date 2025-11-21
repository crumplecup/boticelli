//! Database-specific functionality for TUI.
//!
//! This module contains all database operations used by the TUI.
//! It's feature-gated with the `database` feature.

use crate::ContentRow;
use botticelli_error::BotticelliResult;
use diesel::PgConnection;

/// Load content from a database table.
#[tracing::instrument(skip(conn))]
pub fn load_content(
    conn: &mut PgConnection,
    table_name: &str,
) -> BotticelliResult<Vec<ContentRow>> {
    use botticelli_database::list_content;

    let items = list_content(conn, table_name, None, 1000)?;

    let content_items = items
        .into_iter()
        .filter_map(|item| {
            let id = item.get("id").and_then(|v| v.as_i64())?;
            let review_status = item
                .get("review_status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let rating = item
                .get("rating")
                .and_then(|v| v.as_i64())
                .map(|r| r as i32);
            let tags = item
                .get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|t| t.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            // Create a preview from the JSON content
            let preview = serde_json::to_string(&item)
                .unwrap_or_default()
                .chars()
                .take(50)
                .collect::<String>();

            let source_narrative = item
                .get("source_narrative")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let source_act = item
                .get("source_act")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            Some(ContentRow {
                id,
                review_status,
                rating,
                tags,
                preview,
                content: item,
                source_narrative,
                source_act,
            })
        })
        .collect();

    Ok(content_items)
}

/// Save content metadata updates to database.
#[tracing::instrument(skip(conn))]
pub fn save_content_metadata(
    conn: &mut PgConnection,
    table_name: &str,
    item_id: i64,
    tags: &[String],
    rating: Option<i32>,
    status: &str,
) -> BotticelliResult<()> {
    use botticelli_database::{update_content_metadata, update_review_status};

    update_content_metadata(conn, table_name, item_id, Some(tags), rating)?;
    update_review_status(conn, table_name, item_id, status)?;

    Ok(())
}

/// Delete content item from database.
#[tracing::instrument(skip(conn))]
pub fn delete_content_item(
    conn: &mut PgConnection,
    table_name: &str,
    item_id: i64,
) -> BotticelliResult<()> {
    use botticelli_database::delete_content;

    delete_content(conn, table_name, item_id)?;
    Ok(())
}

/// Promote content to another table.
#[tracing::instrument(skip(conn))]
pub fn promote_content_item(
    conn: &mut PgConnection,
    source_table: &str,
    target_table: &str,
    item_id: i64,
) -> BotticelliResult<i64> {
    use botticelli_database::promote_content;

    let new_id = promote_content(conn, source_table, target_table, item_id)?;
    Ok(new_id)
}
