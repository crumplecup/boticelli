-- Create metadata table for tracking content generation tables
CREATE TABLE content_generation_tables (
    table_name TEXT PRIMARY KEY,
    template_source TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    narrative_file TEXT,
    description TEXT
);

-- Create index for lookups by template source
CREATE INDEX idx_content_generation_tables_template ON content_generation_tables(template_source);

