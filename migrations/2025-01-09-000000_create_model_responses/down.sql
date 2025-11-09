-- Drop model_responses table
DROP INDEX IF EXISTS idx_model_responses_provider_model;
DROP INDEX IF EXISTS idx_model_responses_model_name;
DROP INDEX IF EXISTS idx_model_responses_provider;
DROP INDEX IF EXISTS idx_model_responses_created_at;
DROP TABLE model_responses;
