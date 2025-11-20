//! View structs for table query construction.

use derive_builder::Builder;

/// View for querying table data with flexible filtering and pagination.
#[derive(Debug, Clone, Default, Builder, derive_getters::Getters)]
#[builder(setter(into, strip_option), default)]
pub struct TableQueryView {
    /// Table name to query
    #[builder(setter(into))]
    table_name: String,
    
    /// Specific columns to select (None = all columns)
    columns: Option<Vec<String>>,
    
    /// WHERE clause for filtering
    where_clause: Option<String>,
    
    /// Maximum number of rows to return
    limit: Option<i64>,
    
    /// Number of rows to skip
    offset: Option<i64>,
    
    /// ORDER BY clause
    order_by: Option<String>,
    
    /// Sample size for TABLESAMPLE (not yet implemented)
    sample: Option<i64>,
}

/// View for counting rows in a table with optional filtering.
#[derive(Debug, Clone, Default, Builder, derive_getters::Getters)]
#[builder(setter(into, strip_option), default)]
pub struct TableCountView {
    /// Table name to count
    #[builder(setter(into))]
    table_name: String,
    
    /// WHERE clause for filtering
    where_clause: Option<String>,
}
