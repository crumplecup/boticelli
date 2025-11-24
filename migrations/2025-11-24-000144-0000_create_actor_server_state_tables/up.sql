-- Create actor server state table for persistent task tracking
CREATE TABLE actor_server_state (
    task_id VARCHAR(255) PRIMARY KEY,
    actor_name VARCHAR(255) NOT NULL,
    last_run TIMESTAMPTZ,
    next_run TIMESTAMPTZ NOT NULL,
    consecutive_failures INTEGER DEFAULT 0,
    is_paused BOOLEAN DEFAULT FALSE,
    metadata JSONB DEFAULT '{}',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create actor server executions table for execution history
CREATE TABLE actor_server_executions (
    id BIGSERIAL PRIMARY KEY,
    task_id VARCHAR(255) NOT NULL,
    actor_name VARCHAR(255) NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    success BOOLEAN DEFAULT FALSE,
    error_message TEXT,
    skills_succeeded INTEGER DEFAULT 0,
    skills_failed INTEGER DEFAULT 0,
    skills_skipped INTEGER DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indices for common queries
CREATE INDEX idx_actor_server_executions_task ON actor_server_executions(task_id);
CREATE INDEX idx_actor_server_executions_started ON actor_server_executions(started_at DESC);
CREATE INDEX idx_actor_server_state_next_run ON actor_server_state(next_run) WHERE NOT is_paused;
CREATE INDEX idx_actor_server_state_actor ON actor_server_state(actor_name);
