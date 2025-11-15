-- Restore old media storage columns to act_inputs
-- This allows rolling back to the old storage system if needed

ALTER TABLE act_inputs ADD COLUMN IF NOT EXISTS source_type TEXT;
ALTER TABLE act_inputs ADD COLUMN IF NOT EXISTS source_url TEXT;
ALTER TABLE act_inputs ADD COLUMN IF NOT EXISTS source_base64 TEXT;
ALTER TABLE act_inputs ADD COLUMN IF NOT EXISTS source_binary BYTEA;
ALTER TABLE act_inputs ADD COLUMN IF NOT EXISTS source_size_bytes BIGINT;
ALTER TABLE act_inputs ADD COLUMN IF NOT EXISTS content_hash TEXT;

-- Recreate index
CREATE INDEX IF NOT EXISTS idx_act_inputs_content_hash ON act_inputs(content_hash);

-- Remove comment
COMMENT ON COLUMN act_inputs.media_ref_id IS 'Reference to media stored in media_references table';
