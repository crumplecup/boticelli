-- Drop indexes first
DROP INDEX IF EXISTS idx_content_generations_narrative_file;
DROP INDEX IF EXISTS idx_content_generations_status;
DROP INDEX IF EXISTS idx_content_generations_generated_at;

-- Drop the table
DROP TABLE IF EXISTS content_generations;
