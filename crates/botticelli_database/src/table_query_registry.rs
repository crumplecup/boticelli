//! Implementation of TableQueryRegistry for narrative integration.

use crate::{TableQueryExecutor, format_as_csv, format_as_json, format_as_markdown};
use async_trait::async_trait;
use botticelli_interface::TableQueryRegistry;
use tracing::{debug, error, instrument};

/// Implementation of TableQueryRegistry using TableQueryExecutor.
pub struct DatabaseTableQueryRegistry {
    executor: TableQueryExecutor,
}

impl DatabaseTableQueryRegistry {
    /// Creates a new table query registry.
    pub fn new(executor: TableQueryExecutor) -> Self {
        Self { executor }
    }
}

#[async_trait]
impl TableQueryRegistry for DatabaseTableQueryRegistry {
    #[instrument(
        skip(self, query),
        fields(
            table_name = %query.table_name(),
            columns_count = query.columns().as_ref().map(|c| c.len()),
            has_where = query.filter().is_some(),
            limit = query.limit(),
            offset = query.offset(),
            format = %query.format()
        )
    )]
    async fn query_table(
        &self,
        query: &botticelli_interface::TableQueryView,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Executing table query");

        // Execute query
        let rows = self.executor.query_table(query).map_err(|e| {
            error!(error = %e, "Table query execution failed");
            Box::new(e) as Box<dyn std::error::Error + Send + Sync>
        })?;

        debug!(row_count = rows.len(), "Query executed successfully");

        // Format results based on requested format
        let formatted = match query.format().to_lowercase().as_str() {
            "json" => format_as_json(&rows),
            "markdown" | "md" => format_as_markdown(&rows),
            "csv" => format_as_csv(&rows),
            _ => {
                error!(format = %query.format(), "Unknown format requested, defaulting to JSON");
                format_as_json(&rows)
            }
        };

        debug!(output_length = formatted.len(), "Results formatted");
        Ok(formatted)
    }

    #[instrument(
        skip(self, query),
        fields(
            table_name = %query.table_name(),
            columns_count = query.columns().as_ref().map(|c| c.len()),
            has_where = query.filter().is_some(),
            limit = query.limit(),
            offset = query.offset(),
            format = %query.format()
        )
    )]
    async fn query_and_delete_table(
        &self,
        query: &botticelli_interface::TableQueryView,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Executing destructive table query");

        // Execute query and delete
        let rows = self.executor.query_and_delete_table(query).map_err(|e| {
            error!(error = %e, "Destructive table query execution failed");
            Box::new(e) as Box<dyn std::error::Error + Send + Sync>
        })?;

        debug!(
            row_count = rows.len(),
            "Query and delete executed successfully"
        );

        // Format results based on requested format
        let formatted = match query.format().to_lowercase().as_str() {
            "json" => format_as_json(&rows),
            "markdown" | "md" => format_as_markdown(&rows),
            "csv" => format_as_csv(&rows),
            _ => {
                error!(format = %query.format(), "Unknown format requested, defaulting to JSON");
                format_as_json(&rows)
            }
        };

        debug!(output_length = formatted.len(), "Results formatted");
        Ok(formatted)
    }
}
