-- Remove old media storage columns from act_inputs
-- Run this migration ONLY after validating that all media has been migrated
-- and is accessible via the new storage system.

-- Remove indexes on old columns if they exist
DROP INDEX IF EXISTS idx_act_inputs_content_hash;

-- Remove old media storage columns
ALTER TABLE act_inputs DROP COLUMN IF EXISTS source_type;
ALTER TABLE act_inputs DROP COLUMN IF EXISTS source_url;
ALTER TABLE act_inputs DROP COLUMN IF EXISTS source_base64;
ALTER TABLE act_inputs DROP COLUMN IF EXISTS source_binary;
ALTER TABLE act_inputs DROP COLUMN IF EXISTS source_size_bytes;
ALTER TABLE act_inputs DROP COLUMN IF EXISTS content_hash;

-- Add comment
COMMENT ON COLUMN act_inputs.media_ref_id IS 'Foreign key to media_references table (replaces old source_* columns)';
