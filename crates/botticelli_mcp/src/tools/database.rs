//! Database query tools for MCP server.

use crate::tools::McpTool;
use crate::{McpError, McpResult};
use async_trait::async_trait;
use serde_json::{json, Value};
use tracing::{debug, instrument};

/// Tool for querying content from database tables.
pub struct QueryContentTool;

#[async_trait]
impl McpTool for QueryContentTool {
    fn name(&self) -> &str {
        "query_content"
    }

    fn description(&self) -> &str {
        "Query content from database tables. Returns a list of content items with their metadata."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "table": {
                    "type": "string",
                    "description": "The table name to query (e.g., 'blog_posts', 'tweets')"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results to return (default: 10, max: 100)",
                    "default": 10,
                    "minimum": 1,
                    "maximum": 100
                }
            },
            "required": ["table"]
        })
    }

    #[instrument(skip(self, input), fields(table, limit))]
    async fn execute(&self, input: Value) -> McpResult<Value> {
        let table = input
            .get("table")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidInput("Missing 'table' field".to_string()))?;

        let limit = input
            .get("limit")
            .and_then(|v| v.as_i64())
            .unwrap_or(10)
            .clamp(1, 100);

        debug!(table = %table, limit, "Querying content");

        // TODO: Actually query the database once we add the database dependency
        // For now, return a mock response
        Ok(json!({
            "status": "not_yet_implemented",
            "message": "Database tool stub - will query actual database once dependency is added",
            "requested": {
                "table": table,
                "limit": limit
            },
            "note": "This tool will query PostgreSQL and return actual content in the next iteration"
        }))
    }
}
