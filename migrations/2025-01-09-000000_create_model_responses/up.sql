-- Create model_responses table for storing AI model responses
CREATE TABLE model_responses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),

    -- Model information
    provider VARCHAR(50) NOT NULL,
    model_name VARCHAR(100) NOT NULL,

    -- Request data
    request_messages JSONB NOT NULL,
    request_temperature REAL,
    request_max_tokens INTEGER,
    request_model VARCHAR(100),

    -- Response data
    response_outputs JSONB NOT NULL,

    -- Metadata
    duration_ms INTEGER,
    error_message TEXT,

    -- Indexes for common queries
    CONSTRAINT valid_provider CHECK (provider IN ('gemini', 'anthropic', 'openai', 'huggingface', 'groq', 'perplexity', 'other'))
);

-- Create indexes
CREATE INDEX idx_model_responses_created_at ON model_responses(created_at DESC);
CREATE INDEX idx_model_responses_provider ON model_responses(provider);
CREATE INDEX idx_model_responses_model_name ON model_responses(model_name);
CREATE INDEX idx_model_responses_provider_model ON model_responses(provider, model_name);
