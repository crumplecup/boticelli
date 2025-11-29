-- Drop indices
DROP INDEX IF EXISTS idx_actor_server_state_actor;
DROP INDEX IF EXISTS idx_actor_server_state_next_run;
DROP INDEX IF EXISTS idx_actor_server_executions_started;
DROP INDEX IF EXISTS idx_actor_server_executions_task;

-- Drop tables
DROP TABLE IF EXISTS actor_server_executions;
DROP TABLE IF EXISTS actor_server_state;
