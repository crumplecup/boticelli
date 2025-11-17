-- Track all content generation attempts
CREATE TABLE content_generations (
    id SERIAL PRIMARY KEY,
    table_name TEXT NOT NULL,
    narrative_file TEXT NOT NULL,
    narrative_name TEXT NOT NULL,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    row_count INTEGER,
    generation_duration_ms INTEGER,
    status TEXT NOT NULL CHECK (status IN ('running', 'success', 'failed')),
    error_message TEXT,
    created_by TEXT,
    
    -- Ensure table names are unique
    CONSTRAINT content_generations_table_name_key UNIQUE (table_name)
);

-- Indexes for common query patterns
CREATE INDEX idx_content_generations_generated_at ON content_generations(generated_at DESC);
CREATE INDEX idx_content_generations_status ON content_generations(status);
CREATE INDEX idx_content_generations_narrative_file ON content_generations(narrative_file);

-- Comment for documentation
COMMENT ON TABLE content_generations IS 'Tracks metadata for all content generation executions, including success, failure, and timing information';
