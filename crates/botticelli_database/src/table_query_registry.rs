//! Implementation of TableQueryRegistry for narrative integration.

use crate::{
    format_as_csv, format_as_json, format_as_markdown, TableQueryExecutor,
    table_query_view::TableQueryViewBuilder,
};
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
        skip(self),
        fields(
            table_name,
            columns_count = columns.map(|c| c.len()),
            has_where = where_clause.is_some(),
            limit,
            offset,
            format
        )
    )]
    async fn query_table(
        &self,
        table_name: &str,
        columns: Option<&[String]>,
        where_clause: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
        order_by: Option<&str>,
        format: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Building table query view");

        // Build query view using builder pattern
        let mut builder = TableQueryViewBuilder::default();
        builder.table_name(table_name.to_string());

        if let Some(cols) = columns {
            builder.columns(cols.to_vec());
        }

        if let Some(where_str) = where_clause {
            builder.where_clause(where_str.to_string());
        }

        if let Some(lim) = limit {
            builder.limit(lim as i64);
        }

        if let Some(off) = offset {
            builder.offset(off as i64);
        }

        if let Some(order) = order_by {
            builder.order_by(order.to_string());
        }

        let view = builder
            .build()
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                error!(error = %e, "Failed to build table query view");
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Invalid table query parameters: {}", e),
                ))
            })?;

        // Execute query
        let rows = self.executor.query_table(&view).map_err(|e| {
            error!(error = %e, "Table query execution failed");
            Box::new(e) as Box<dyn std::error::Error + Send + Sync>
        })?;

        debug!(row_count = rows.len(), "Query executed successfully");

        // Format results based on requested format
        let formatted = match format.to_lowercase().as_str() {
            "json" => format_as_json(&rows),
            "markdown" | "md" => format_as_markdown(&rows),
            "csv" => format_as_csv(&rows),
            _ => {
                error!(format = %format, "Unknown format requested, defaulting to JSON");
                format_as_json(&rows)
            }
        };

        debug!(output_length = formatted.len(), "Results formatted");
        Ok(formatted)
    }
}
