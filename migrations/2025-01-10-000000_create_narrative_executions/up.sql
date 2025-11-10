-- Create narrative_executions table
CREATE TABLE narrative_executions (
    id SERIAL PRIMARY KEY,
    narrative_name TEXT NOT NULL,
    narrative_description TEXT,
    started_at TIMESTAMP NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP,
    status TEXT NOT NULL DEFAULT 'running',
    error_message TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create act_executions table
CREATE TABLE act_executions (
    id SERIAL PRIMARY KEY,
    execution_id INTEGER NOT NULL REFERENCES narrative_executions(id) ON DELETE CASCADE,
    act_name TEXT NOT NULL,
    sequence_number INTEGER NOT NULL,
    model TEXT,
    temperature REAL,
    max_tokens INTEGER,
    response TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create act_inputs table
CREATE TABLE act_inputs (
    id SERIAL PRIMARY KEY,
    act_execution_id INTEGER NOT NULL REFERENCES act_executions(id) ON DELETE CASCADE,
    input_order INTEGER NOT NULL,
    input_type TEXT NOT NULL,
    text_content TEXT,
    mime_type TEXT,
    source_type TEXT,
    source_url TEXT,
    source_base64 TEXT,
    source_binary BYTEA,
    source_size_bytes BIGINT,
    content_hash TEXT,
    filename TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create indexes for common queries
CREATE INDEX idx_narrative_executions_name ON narrative_executions(narrative_name);
CREATE INDEX idx_narrative_executions_status ON narrative_executions(status);
CREATE INDEX idx_narrative_executions_started_at ON narrative_executions(started_at);

CREATE INDEX idx_act_executions_execution_id ON act_executions(execution_id);
CREATE INDEX idx_act_executions_sequence ON act_executions(execution_id, sequence_number);

CREATE INDEX idx_act_inputs_act_execution_id ON act_inputs(act_execution_id);
CREATE INDEX idx_act_inputs_order ON act_inputs(act_execution_id, input_order);
CREATE INDEX idx_act_inputs_content_hash ON act_inputs(content_hash);
