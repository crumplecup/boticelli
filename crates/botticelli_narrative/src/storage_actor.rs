//! Storage actor for asynchronous table operations.
//!
//! This module provides an actor-based abstraction for content storage,
//! handling table creation, schema inference, and row insertion through
//! an asynchronous message-passing interface.

use actix::prelude::*;
use botticelli_database::{
    ContentGenerationRepository, NewContentGenerationRow, PostgresContentGenerationRepository,
    UpdateContentGenerationRow, create_content_table, create_inferred_table, infer_schema,
    reflect_table_schema,
};
use botticelli_error::{BotticelliError, BotticelliResult};
use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use serde_json::Value as JsonValue;
use std::time::Instant;

/// Storage actor handling all database operations for content generation.
pub struct StorageActor {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl StorageActor {
    /// Create a new storage actor with a connection pool.
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self { pool }
    }

    /// Get a connection from the pool.
    fn get_conn(&self) -> BotticelliResult<diesel::r2d2::PooledConnection<ConnectionManager<PgConnection>>> {
        self.pool.get().map_err(|e| {
            botticelli_error::BackendError::new(format!("Failed to get connection from pool: {}", e))
        })
    }
}

impl Actor for StorageActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        tracing::info!("StorageActor started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        tracing::info!("StorageActor stopped");
    }
}

/// Message to start tracking a content generation.
#[derive(Debug, Message)]
#[rtype(result = "BotticelliResult<()>")]
pub struct StartGeneration {
    pub table_name: String,
    pub narrative_file: String,
    pub narrative_name: String,
}

impl Handler<StartGeneration> for StorageActor {
    type Result = BotticelliResult<()>;

    fn handle(&mut self, msg: StartGeneration, _ctx: &mut Self::Context) -> Self::Result {
        let mut conn = self.get_conn()?;
        let mut repo = PostgresContentGenerationRepository::new(&mut conn);

        let new_gen = NewContentGenerationRow {
            table_name: msg.table_name.clone(),
            narrative_file: msg.narrative_file,
            narrative_name: msg.narrative_name,
            status: "running".to_string(),
            created_by: None,
        };

        repo.start_generation(new_gen).map_err(|e| {
            tracing::debug!(
                error = %e,
                table = %msg.table_name,
                "Could not start tracking (may already exist)"
            );
            e
        })?;

        tracing::info!(table = %msg.table_name, "Started tracking content generation");
        Ok(())
    }
}

/// Message to create a table from a template.
#[derive(Debug, Message)]
#[rtype(result = "BotticelliResult<()>")]
pub struct CreateTableFromTemplate {
    pub table_name: String,
    pub template: String,
    pub narrative_name: Option<String>,
    pub description: Option<String>,
}

impl Handler<CreateTableFromTemplate> for StorageActor {
    type Result = BotticelliResult<()>;

    fn handle(&mut self, msg: CreateTableFromTemplate, _ctx: &mut Self::Context) -> Self::Result {
        let mut conn = self.get_conn()?;

        tracing::debug!(
            template = %msg.template,
            table = %msg.table_name,
            "Creating table from template"
        );

        create_content_table(
            &mut conn,
            &msg.table_name,
            &msg.template,
            msg.narrative_name.as_deref(),
            msg.description.as_deref(),
        )?;

        tracing::info!(table = %msg.table_name, "Table created from template");
        Ok(())
    }
}

/// Message to create a table with inferred schema.
#[derive(Debug, Message)]
#[rtype(result = "BotticelliResult<()>")]
pub struct CreateTableFromInference {
    pub table_name: String,
    pub json_sample: JsonValue,
    pub narrative_name: Option<String>,
    pub description: Option<String>,
}

impl Handler<CreateTableFromInference> for StorageActor {
    type Result = BotticelliResult<()>;

    fn handle(&mut self, msg: CreateTableFromInference, _ctx: &mut Self::Context) -> Self::Result {
        let mut conn = self.get_conn()?;

        tracing::debug!(table = %msg.table_name, "Inferring schema from JSON");

        let schema = infer_schema(&msg.json_sample)?;

        tracing::info!(
            field_count = schema.field_count(),
            table = %msg.table_name,
            "Inferred schema from JSON"
        );

        create_inferred_table(
            &mut conn,
            &msg.table_name,
            &schema,
            msg.narrative_name.as_deref(),
            msg.description.as_deref(),
        )?;

        tracing::info!(table = %msg.table_name, "Inferred table created successfully");
        Ok(())
    }
}

/// Message to insert content into a table.
#[derive(Debug, Message)]
#[rtype(result = "BotticelliResult<()>")]
pub struct InsertContent {
    pub table_name: String,
    pub json_data: JsonValue,
    pub narrative_name: String,
    pub act_name: String,
    pub model: Option<String>,
}

impl Handler<InsertContent> for StorageActor {
    type Result = BotticelliResult<()>;

    fn handle(&mut self, msg: InsertContent, _ctx: &mut Self::Context) -> Self::Result {
        let mut conn = self.get_conn()?;

        // Query schema to get column types
        let schema = reflect_table_schema(&mut conn, &msg.table_name)?;
        let column_types: std::collections::HashMap<_, _> = schema
            .columns
            .iter()
            .map(|col| (col.name.as_str(), col.data_type.as_str()))
            .collect();

        // Build INSERT statement dynamically
        let obj = msg.json_data
            .as_object()
            .ok_or_else(|| botticelli_error::BackendError::new("JSON must be an object"))?;

        let mut columns = Vec::new();
        let mut values = Vec::new();

        // Add content fields from JSON
        for (key, value) in obj {
            columns.push(key.clone());
            let col_type = column_types.get(key.as_str()).copied().unwrap_or("text");
            values.push(json_value_to_sql(value, col_type));
        }

        // Add metadata columns
        columns.push("source_narrative".to_string());
        values.push(format!("'{}'", msg.narrative_name));

        columns.push("source_act".to_string());
        values.push(format!("'{}'", msg.act_name));

        if let Some(m) = &msg.model {
            columns.push("generation_model".to_string());
            values.push(format!("'{}'", m));
        }

        columns.push("created_at".to_string());
        values.push("NOW()".to_string());

        // Execute INSERT
        let query = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            msg.table_name,
            columns.join(", "),
            values.join(", ")
        );

        tracing::debug!(sql = %query, "Executing INSERT");

        diesel::sql_query(&query).execute(&mut conn).map_err(|e| {
            botticelli_error::BackendError::new(format!("Failed to insert content: {}", e))
        })?;

        tracing::debug!(
            table = %msg.table_name,
            act = %msg.act_name,
            "Content inserted successfully"
        );

        Ok(())
    }
}

/// Message to complete a content generation.
#[derive(Debug, Message)]
#[rtype(result = "BotticelliResult<()>")]
pub struct CompleteGeneration {
    pub table_name: String,
    pub row_count: Option<i32>,
    pub duration_ms: i32,
    pub status: String,
    pub error_message: Option<String>,
}

impl Handler<CompleteGeneration> for StorageActor {
    type Result = BotticelliResult<()>;

    fn handle(&mut self, msg: CompleteGeneration, _ctx: &mut Self::Context) -> Self::Result {
        let mut conn = self.get_conn()?;
        let mut repo = PostgresContentGenerationRepository::new(&mut conn);

        let update = UpdateContentGenerationRow {
            completed_at: Some(Utc::now()),
            row_count: msg.row_count,
            generation_duration_ms: Some(msg.duration_ms),
            status: Some(msg.status.clone()),
            error_message: msg.error_message.clone(),
        };

        repo.complete_generation(&msg.table_name, update).map_err(|e| {
            tracing::warn!(
                error = %e,
                table = %msg.table_name,
                "Failed to update tracking record"
            );
            e
        })?;

        tracing::info!(
            table = %msg.table_name,
            row_count = ?msg.row_count,
            duration_ms = msg.duration_ms,
            status = %msg.status,
            "Updated tracking: generation complete"
        );

        Ok(())
    }
}

/// Convert a JSON value to SQL literal string.
fn json_value_to_sql(value: &JsonValue, col_type: &str) -> String {
    match value {
        JsonValue::Null => "NULL".to_string(),
        JsonValue::Bool(b) => b.to_string(),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::String(s) => {
            if col_type == "jsonb" || col_type == "json" {
                format!("'{}'::jsonb", s.replace('\'', "''"))
            } else {
                format!("'{}'", s.replace('\'', "''"))
            }
        }
        JsonValue::Array(_) | JsonValue::Object(_) => {
            format!("'{}'::jsonb", value.to_string().replace('\'', "''"))
        }
    }
}
