# Phase 2: Bot Command Execution - Detailed Implementation Plan

## Overview

Phase 2 adds the ability to execute bot commands (Discord, Slack, etc.) from narrative acts. This enables narratives to fetch real-time data from platforms, analyze it, and incorporate the results into content generation workflows.

## Goals

1. **Execute bot commands from narratives** - Turn `Input::BotCommand` into actual API calls
2. **Platform abstraction** - Support multiple platforms (Discord primary, others later)
3. **Caching & performance** - Cache results, respect rate limits
4. **Error handling** - Graceful failures with helpful context
5. **Security** - Read-only operations, proper permissions

## Architecture

### Trait Design

```rust
/// Executes bot commands for a specific platform.
///
/// Implementations handle the platform-specific API calls and return
/// structured JSON results that can be converted to text for LLM consumption.
#[async_trait]
pub trait BotCommandExecutor: Send + Sync {
    /// Returns the platform this executor handles (e.g., "discord", "slack").
    fn platform(&self) -> &str;
    
    /// Execute a command and return JSON result.
    ///
    /// # Arguments
    /// * `command` - Command string (e.g., "server.get_stats", "channels.list")
    /// * `args` - Command arguments as JSON values
    ///
    /// # Returns
    /// JSON value representing the command result
    ///
    /// # Errors
    /// Returns error if:
    /// - Command is not supported
    /// - API call fails
    /// - Authentication fails
    /// - Rate limit exceeded
    async fn execute(
        &self,
        command: &str,
        args: &HashMap<String, serde_json::Value>,
    ) -> BotCommandResult<serde_json::Value>;
    
    /// Check if this executor supports a command.
    fn supports_command(&self, command: &str) -> bool;
    
    /// List all supported commands.
    fn supported_commands(&self) -> Vec<String>;
    
    /// Get command documentation.
    fn command_help(&self, command: &str) -> Option<String>;
}

/// Result type for bot command execution.
pub type BotCommandResult<T> = Result<T, BotCommandError>;
```

### Error Handling

```rust
/// Errors that can occur during bot command execution.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, derive_more::Display)]
pub enum BotCommandErrorKind {
    /// Command not found or not supported.
    #[display("Command not found: {}", _0)]
    CommandNotFound(String),
    
    /// Platform not found (no executor registered).
    #[display("Platform not found: {}", _0)]
    PlatformNotFound(String),
    
    /// Missing required argument.
    #[display("Missing required argument '{}' for command '{}'", arg_name, command)]
    MissingArgument {
        command: String,
        arg_name: String,
    },
    
    /// Invalid argument type or value.
    #[display("Invalid argument '{}' for command '{}': {}", arg_name, command, reason)]
    InvalidArgument {
        command: String,
        arg_name: String,
        reason: String,
    },
    
    /// API call failed.
    #[display("API call failed for '{}': {}", command, reason)]
    ApiError {
        command: String,
        reason: String,
    },
    
    /// Authentication failed.
    #[display("Authentication failed for platform '{}': {}", platform, reason)]
    AuthenticationError {
        platform: String,
        reason: String,
    },
    
    /// Rate limit exceeded.
    #[display("Rate limit exceeded for '{}': retry after {} seconds", command, retry_after)]
    RateLimitExceeded {
        command: String,
        retry_after: u64,
    },
    
    /// Permission denied.
    #[display("Permission denied for '{}': {}", command, reason)]
    PermissionDenied {
        command: String,
        reason: String,
    },
    
    /// Resource not found (guild, channel, user, etc.).
    #[display("Resource not found for '{}': {}", command, resource_type)]
    ResourceNotFound {
        command: String,
        resource_type: String,
    },
    
    /// Serialization/deserialization error.
    #[display("Serialization error for '{}': {}", command, reason)]
    SerializationError {
        command: String,
        reason: String,
    },
}

/// Bot command error with location tracking.
#[derive(Debug, Clone, derive_more::Display, derive_more::Error)]
#[display("Bot Command Error: {} at line {} in {}", kind, line, file)]
pub struct BotCommandError {
    pub kind: BotCommandErrorKind,
    pub line: u32,
    pub file: &'static str,
}

impl BotCommandError {
    #[track_caller]
    pub fn new(kind: BotCommandErrorKind) -> Self {
        let location = std::panic::Location::caller();
        Self {
            kind,
            line: location.line(),
            file: location.file(),
        }
    }
}
```

### Command Registry

```rust
/// Registry of bot command executors for multiple platforms.
pub struct BotCommandRegistry {
    executors: HashMap<String, Arc<dyn BotCommandExecutor>>,
}

impl BotCommandRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            executors: HashMap::new(),
        }
    }
    
    /// Register an executor for a platform.
    pub fn register<E: BotCommandExecutor + 'static>(
        &mut self,
        executor: E,
    ) -> &mut Self {
        let platform = executor.platform().to_string();
        self.executors.insert(platform, Arc::new(executor));
        self
    }
    
    /// Get executor for a platform.
    pub fn get(&self, platform: &str) -> Option<&Arc<dyn BotCommandExecutor>> {
        self.executors.get(platform)
    }
    
    /// Execute a command on a platform.
    pub async fn execute(
        &self,
        platform: &str,
        command: &str,
        args: &HashMap<String, serde_json::Value>,
    ) -> BotCommandResult<serde_json::Value> {
        let executor = self
            .get(platform)
            .ok_or_else(|| BotCommandError::new(
                BotCommandErrorKind::PlatformNotFound(platform.to_string())
            ))?;
        
        executor.execute(command, args).await
    }
    
    /// List all registered platforms.
    pub fn platforms(&self) -> Vec<String> {
        self.executors.keys().cloned().collect()
    }
}
```

### Caching Layer

```rust
/// Cached result of a bot command execution.
#[derive(Debug, Clone)]
struct CachedResult {
    result: serde_json::Value,
    cached_at: std::time::Instant,
    ttl: std::time::Duration,
}

impl CachedResult {
    fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }
}

/// Wraps an executor with caching.
pub struct CachedBotCommandExecutor<E> {
    inner: E,
    cache: Arc<Mutex<HashMap<String, CachedResult>>>,
    default_ttl: std::time::Duration,
}

impl<E: BotCommandExecutor> CachedBotCommandExecutor<E> {
    pub fn new(executor: E, default_ttl: std::time::Duration) -> Self {
        Self {
            inner: executor,
            cache: Arc::new(Mutex::new(HashMap::new())),
            default_ttl,
        }
    }
    
    fn cache_key(&self, command: &str, args: &HashMap<String, serde_json::Value>) -> String {
        // Create stable cache key from command + sorted args
        let mut sorted_args: Vec<_> = args.iter().collect();
        sorted_args.sort_by_key(|(k, _)| *k);
        format!("{}:{:?}", command, sorted_args)
    }
}

#[async_trait]
impl<E: BotCommandExecutor> BotCommandExecutor for CachedBotCommandExecutor<E> {
    fn platform(&self) -> &str {
        self.inner.platform()
    }
    
    async fn execute(
        &self,
        command: &str,
        args: &HashMap<String, serde_json::Value>,
    ) -> BotCommandResult<serde_json::Value> {
        let cache_key = self.cache_key(command, args);
        
        // Check cache
        {
            let cache = self.cache.lock().unwrap();
            if let Some(cached) = cache.get(&cache_key) {
                if !cached.is_expired() {
                    tracing::debug!(
                        command,
                        age_secs = cached.cached_at.elapsed().as_secs(),
                        "Cache hit for bot command"
                    );
                    return Ok(cached.result.clone());
                }
            }
        }
        
        // Cache miss - execute command
        tracing::debug!(command, "Cache miss for bot command");
        let result = self.inner.execute(command, args).await?;
        
        // Cache result
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(
                cache_key,
                CachedResult {
                    result: result.clone(),
                    cached_at: std::time::Instant::now(),
                    ttl: self.default_ttl,
                },
            );
        }
        
        Ok(result)
    }
    
    fn supports_command(&self, command: &str) -> bool {
        self.inner.supports_command(command)
    }
    
    fn supported_commands(&self) -> Vec<String> {
        self.inner.supported_commands()
    }
    
    fn command_help(&self, command: &str) -> Option<String> {
        self.inner.command_help(command)
    }
}
```

## Discord Implementation

### Command Structure

Discord commands follow a hierarchical namespace:
- `server.*` - Server/guild operations
- `channels.*` - Channel operations
- `members.*` - Member operations
- `roles.*` - Role operations
- `messages.*` - Message operations

### Supported Commands (Initial Set)

```rust
pub struct DiscordBotCommandExecutor {
    client: Arc<DiscordClient>,
    commands: HashMap<String, CommandDefinition>,
}

struct CommandDefinition {
    name: String,
    description: String,
    required_args: Vec<String>,
    optional_args: Vec<String>,
    handler: CommandHandler,
}

type CommandHandler = Arc<
    dyn Fn(
        &DiscordClient,
        &HashMap<String, serde_json::Value>,
    ) -> Pin<Box<dyn Future<Output = BotCommandResult<serde_json::Value>> + Send>>
    + Send
    + Sync
>;
```

#### Server Commands

**`server.get_stats`** - Get server statistics
```rust
// Required args: guild_id
// Returns: JSON with member_count, channel_count, role_count, created_at, etc.
{
    "guild_id": "1234567890",
    "name": "Botticelli Community",
    "member_count": 1250,
    "channel_count": 45,
    "role_count": 15,
    "created_at": "2024-01-15T10:30:00Z",
    "boost_level": 2,
    "boost_count": 14,
    "icon_url": "https://cdn.discordapp.com/...",
    "description": "Official Botticelli community server"
}
```

**`server.get_info`** - Get detailed server information
```rust
// Required args: guild_id
// Returns: Full guild information including features, verification level, etc.
```

#### Channel Commands

**`channels.list`** - List all channels in a server
```rust
// Required args: guild_id
// Optional args: type (text, voice, category, etc.)
// Returns: Array of channel objects
[
    {
        "id": "1234567890",
        "name": "general",
        "type": "text",
        "position": 0,
        "topic": "General discussion",
        "nsfw": false,
        "parent_id": "9876543210"
    },
    // ...
]
```

**`channels.get_info`** - Get detailed channel information
```rust
// Required args: channel_id
// Returns: Channel object with full details
```

**`channels.get_stats`** - Get channel activity statistics
```rust
// Required args: channel_id
// Optional args: days (default: 7)
// Returns: Statistics for the channel
{
    "channel_id": "1234567890",
    "name": "general",
    "message_count": 1543,
    "active_members": 234,
    "top_posters": [
        {"user_id": "111", "username": "alice", "message_count": 45},
        {"user_id": "222", "username": "bob", "message_count": 38}
    ],
    "time_period_days": 7
}
```

#### Member Commands

**`members.list`** - List members in a server
```rust
// Required args: guild_id
// Optional args: limit (default: 100), role_id (filter by role)
// Returns: Array of member objects
```

**`members.get_info`** - Get detailed member information
```rust
// Required args: guild_id, user_id
// Returns: Member object with roles, join date, etc.
```

**`members.get_stats`** - Get member activity statistics
```rust
// Required args: guild_id, user_id
// Optional args: days (default: 30)
// Returns: Activity stats for the member
```

#### Role Commands

**`roles.list`** - List all roles in a server
```rust
// Required args: guild_id
// Returns: Array of role objects
[
    {
        "id": "1234567890",
        "name": "@everyone",
        "color": 0,
        "position": 0,
        "permissions": 104324161,
        "member_count": 1250
    },
    // ...
]
```

#### Message Commands

**`messages.recent`** - Get recent messages from a channel
```rust
// Required args: channel_id
// Optional args: limit (default: 50, max: 100)
// Returns: Array of message objects
```

### Implementation Example

```rust
impl DiscordBotCommandExecutor {
    pub fn new(client: Arc<DiscordClient>) -> Self {
        let mut executor = Self {
            client,
            commands: HashMap::new(),
        };
        
        executor.register_server_commands();
        executor.register_channel_commands();
        executor.register_member_commands();
        executor.register_role_commands();
        executor.register_message_commands();
        
        executor
    }
    
    fn register_server_commands(&mut self) {
        // server.get_stats
        self.commands.insert(
            "server.get_stats".to_string(),
            CommandDefinition {
                name: "server.get_stats".to_string(),
                description: "Get server statistics".to_string(),
                required_args: vec!["guild_id".to_string()],
                optional_args: vec![],
                handler: Arc::new(|client, args| {
                    Box::pin(async move {
                        let guild_id = args
                            .get("guild_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| BotCommandError::new(
                                BotCommandErrorKind::MissingArgument {
                                    command: "server.get_stats".to_string(),
                                    arg_name: "guild_id".to_string(),
                                }
                            ))?;
                        
                        let guild_id: u64 = guild_id
                            .parse()
                            .map_err(|_| BotCommandError::new(
                                BotCommandErrorKind::InvalidArgument {
                                    command: "server.get_stats".to_string(),
                                    arg_name: "guild_id".to_string(),
                                    reason: "Invalid guild ID format".to_string(),
                                }
                            ))?;
                        
                        let stats = client.get_guild_stats(guild_id).await?;
                        
                        Ok(serde_json::to_value(stats).map_err(|e| {
                            BotCommandError::new(BotCommandErrorKind::SerializationError {
                                command: "server.get_stats".to_string(),
                                reason: e.to_string(),
                            })
                        })?)
                    })
                }),
            },
        );
        
        // More server commands...
    }
}

#[async_trait]
impl BotCommandExecutor for DiscordBotCommandExecutor {
    fn platform(&self) -> &str {
        "discord"
    }
    
    async fn execute(
        &self,
        command: &str,
        args: &HashMap<String, serde_json::Value>,
    ) -> BotCommandResult<serde_json::Value> {
        tracing::info!(command, ?args, "Executing Discord bot command");
        
        let cmd_def = self
            .commands
            .get(command)
            .ok_or_else(|| BotCommandError::new(
                BotCommandErrorKind::CommandNotFound(command.to_string())
            ))?;
        
        // Validate required arguments
        for required_arg in &cmd_def.required_args {
            if !args.contains_key(required_arg) {
                return Err(BotCommandError::new(
                    BotCommandErrorKind::MissingArgument {
                        command: command.to_string(),
                        arg_name: required_arg.clone(),
                    }
                ));
            }
        }
        
        // Execute command
        (cmd_def.handler)(&self.client, args).await
    }
    
    fn supports_command(&self, command: &str) -> bool {
        self.commands.contains_key(command)
    }
    
    fn supported_commands(&self) -> Vec<String> {
        self.commands.keys().cloned().collect()
    }
    
    fn command_help(&self, command: &str) -> Option<String> {
        self.commands.get(command).map(|def| {
            format!(
                "{}\n\nRequired arguments: {}\nOptional arguments: {}",
                def.description,
                def.required_args.join(", "),
                def.optional_args.join(", ")
            )
        })
    }
}
```

## NarrativeExecutor Integration

### Builder Pattern

```rust
impl NarrativeExecutor {
    /// Add a bot command executor for a platform.
    pub fn with_bot_executor<E: BotCommandExecutor + 'static>(
        mut self,
        executor: E,
    ) -> Self {
        let platform = executor.platform().to_string();
        self.bot_executors.insert(platform, Arc::new(executor));
        self
    }
    
    /// Build executor with default Discord executor.
    #[cfg(feature = "discord")]
    pub fn with_discord_bot(
        self,
        discord_client: Arc<DiscordClient>,
        cache_ttl: std::time::Duration,
    ) -> Self {
        let executor = DiscordBotCommandExecutor::new(discord_client);
        let cached_executor = CachedBotCommandExecutor::new(executor, cache_ttl);
        self.with_bot_executor(cached_executor)
    }
}
```

### Input Processing

```rust
impl NarrativeExecutor {
    async fn process_input(&self, input: &Input) -> Result<String, NarrativeError> {
        match input {
            Input::Text(content) => Ok(content.clone()),
            
            Input::BotCommand {
                platform,
                command,
                args,
                required,
                cache_duration,
            } => {
                tracing::info!(platform, command, "Processing bot command input");
                
                let executor = self
                    .bot_executors
                    .get(platform)
                    .ok_or_else(|| NarrativeError::new(
                        NarrativeErrorKind::BotCommandError(format!(
                            "No executor registered for platform: {}",
                            platform
                        ))
                    ))?;
                
                match executor.execute(command, args).await {
                    Ok(result) => {
                        // Convert JSON result to pretty-printed string for LLM
                        let text = serde_json::to_string_pretty(&result)
                            .map_err(|e| NarrativeError::new(
                                NarrativeErrorKind::BotCommandError(
                                    format!("Failed to serialize result: {}", e)
                                )
                            ))?;
                        
                        tracing::debug!(
                            command,
                            result_length = text.len(),
                            "Bot command executed successfully"
                        );
                        
                        Ok(text)
                    }
                    Err(e) => {
                        if *required {
                            // Halt execution if command is required
                            Err(NarrativeError::new(
                                NarrativeErrorKind::BotCommandError(
                                    format!("Required command failed: {}", e)
                                )
                            ))
                        } else {
                            // Continue with error message as context
                            tracing::warn!(
                                command,
                                error = %e,
                                "Optional bot command failed, continuing"
                            );
                            Ok(format!("[Bot command '{}' failed: {}]", command, e))
                        }
                    }
                }
            }
            
            // ... other input types
        }
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    /// Mock executor for testing
    struct MockBotCommandExecutor {
        responses: HashMap<String, serde_json::Value>,
    }
    
    impl MockBotCommandExecutor {
        fn new() -> Self {
            let mut responses = HashMap::new();
            
            // Mock server.get_stats response
            responses.insert(
                "server.get_stats".to_string(),
                serde_json::json!({
                    "guild_id": "1234567890",
                    "name": "Test Server",
                    "member_count": 100,
                    "channel_count": 10
                }),
            );
            
            Self { responses }
        }
    }
    
    #[async_trait]
    impl BotCommandExecutor for MockBotCommandExecutor {
        fn platform(&self) -> &str {
            "mock"
        }
        
        async fn execute(
            &self,
            command: &str,
            _args: &HashMap<String, serde_json::Value>,
        ) -> BotCommandResult<serde_json::Value> {
            self.responses
                .get(command)
                .cloned()
                .ok_or_else(|| BotCommandError::new(
                    BotCommandErrorKind::CommandNotFound(command.to_string())
                ))
        }
        
        fn supports_command(&self, command: &str) -> bool {
            self.responses.contains_key(command)
        }
        
        fn supported_commands(&self) -> Vec<String> {
            self.responses.keys().cloned().collect()
        }
        
        fn command_help(&self, _command: &str) -> Option<String> {
            None
        }
    }
    
    #[tokio::test]
    async fn test_bot_command_execution() {
        let executor = MockBotCommandExecutor::new();
        let mut args = HashMap::new();
        args.insert("guild_id".to_string(), serde_json::json!("1234567890"));
        
        let result = executor.execute("server.get_stats", &args).await.unwrap();
        
        assert_eq!(result["member_count"], 100);
        assert_eq!(result["channel_count"], 10);
    }
    
    #[tokio::test]
    async fn test_bot_command_caching() {
        let inner = MockBotCommandExecutor::new();
        let cached = CachedBotCommandExecutor::new(
            inner,
            std::time::Duration::from_secs(60),
        );
        
        let mut args = HashMap::new();
        args.insert("guild_id".to_string(), serde_json::json!("1234567890"));
        
        // First call - cache miss
        let result1 = cached.execute("server.get_stats", &args).await.unwrap();
        
        // Second call - cache hit
        let result2 = cached.execute("server.get_stats", &args).await.unwrap();
        
        assert_eq!(result1, result2);
    }
    
    #[tokio::test]
    async fn test_bot_command_registry() {
        let mut registry = BotCommandRegistry::new();
        registry.register(MockBotCommandExecutor::new());
        
        let mut args = HashMap::new();
        args.insert("guild_id".to_string(), serde_json::json!("1234567890"));
        
        let result = registry
            .execute("mock", "server.get_stats", &args)
            .await
            .unwrap();
        
        assert_eq!(result["member_count"], 100);
    }
    
    #[tokio::test]
    async fn test_unknown_platform() {
        let registry = BotCommandRegistry::new();
        let args = HashMap::new();
        
        let result = registry.execute("unknown", "test", &args).await;
        
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().kind,
            BotCommandErrorKind::PlatformNotFound(_)
        ));
    }
    
    #[tokio::test]
    async fn test_unknown_command() {
        let executor = MockBotCommandExecutor::new();
        let args = HashMap::new();
        
        let result = executor.execute("unknown.command", &args).await;
        
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().kind,
            BotCommandErrorKind::CommandNotFound(_)
        ));
    }
}
```

### Integration Tests

```rust
#[cfg(all(test, feature = "discord"))]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Requires Discord bot token
    async fn test_discord_server_stats() {
        let token = std::env::var("DISCORD_BOT_TOKEN")
            .expect("DISCORD_BOT_TOKEN not set");
        
        let client = DiscordClient::new(&token).await.unwrap();
        let executor = DiscordBotCommandExecutor::new(Arc::new(client));
        
        let mut args = HashMap::new();
        args.insert(
            "guild_id".to_string(),
            serde_json::json!(std::env::var("TEST_GUILD_ID").unwrap()),
        );
        
        let result = executor.execute("server.get_stats", &args).await.unwrap();
        
        assert!(result.get("member_count").is_some());
        assert!(result.get("channel_count").is_some());
    }
}
```

## Security Considerations

### Read-Only Operations

All bot commands MUST be read-only:
- ❌ No message sending
- ❌ No role modifications
- ❌ No member kicks/bans
- ❌ No channel creation/deletion
- ✅ Only data retrieval

### Permission Validation

```rust
impl DiscordBotCommandExecutor {
    async fn validate_permissions(
        &self,
        guild_id: u64,
        required_permission: Permission,
    ) -> BotCommandResult<()> {
        let bot_member = self.client.get_bot_member(guild_id).await?;
        
        if !bot_member.has_permission(required_permission) {
            return Err(BotCommandError::new(
                BotCommandErrorKind::PermissionDenied {
                    command: "N/A".to_string(),
                    reason: format!("Missing permission: {:?}", required_permission),
                }
            ));
        }
        
        Ok(())
    }
}
```

### Rate Limiting

Respect Discord API rate limits:
```rust
impl DiscordClient {
    async fn call_api_with_rate_limit<T>(
        &self,
        endpoint: &str,
    ) -> Result<T, DiscordApiError> {
        // Check rate limit bucket
        let bucket = self.get_rate_limit_bucket(endpoint);
        
        if bucket.is_exhausted() {
            return Err(DiscordApiError::RateLimitExceeded {
                retry_after: bucket.reset_after(),
            });
        }
        
        // Make API call
        let response = self.http_client.get(endpoint).send().await?;
        
        // Update rate limit state from headers
        bucket.update_from_headers(response.headers());
        
        Ok(response.json().await?)
    }
}
```

### Input Sanitization

```rust
fn sanitize_guild_id(value: &serde_json::Value) -> BotCommandResult<u64> {
    let id_str = value
        .as_str()
        .ok_or_else(|| BotCommandError::new(
            BotCommandErrorKind::InvalidArgument {
                command: "N/A".to_string(),
                arg_name: "guild_id".to_string(),
                reason: "Must be a string".to_string(),
            }
        ))?;
    
    // Validate it's a valid Discord snowflake (numeric string)
    let id: u64 = id_str
        .parse()
        .map_err(|_| BotCommandError::new(
            BotCommandErrorKind::InvalidArgument {
                command: "N/A".to_string(),
                arg_name: "guild_id".to_string(),
                reason: "Invalid Discord ID format".to_string(),
            }
        ))?;
    
    // Validate range (Discord snowflakes are 64-bit)
    if id == 0 {
        return Err(BotCommandError::new(
            BotCommandErrorKind::InvalidArgument {
                command: "N/A".to_string(),
                arg_name: "guild_id".to_string(),
                reason: "ID cannot be zero".to_string(),
            }
        ));
    }
    
    Ok(id)
}
```

## Performance Optimizations

### Batch Operations

When possible, batch multiple related operations:
```rust
// Instead of N calls to get_channel_info
let channels = client.get_all_channels(guild_id).await?;

// Instead of N calls to get_member_info
let members = client.get_members_batch(guild_id, member_ids).await?;
```

### Parallel Execution

Execute independent commands in parallel:
```rust
let (stats_result, channels_result) = tokio::join!(
    executor.execute("server.get_stats", &stats_args),
    executor.execute("channels.list", &channels_args),
);
```

### Connection Pooling

Reuse HTTP connections:
```rust
let http_client = reqwest::Client::builder()
    .pool_max_idle_per_host(10)
    .pool_idle_timeout(std::time::Duration::from_secs(90))
    .build()?;
```

## Documentation Requirements

### Command Documentation

Each command needs:
1. **Description** - What the command does
2. **Required arguments** - With types and constraints
3. **Optional arguments** - With defaults
4. **Return format** - JSON schema
5. **Example usage** - TOML snippet
6. **Permissions needed** - Discord permissions
7. **Rate limit info** - Calls per minute/hour

### User Guide

Create `BOT_COMMANDS.md` with:
- How to set up bot for command execution
- List of all supported commands
- Examples for each platform
- Troubleshooting common issues
- Security best practices

## Implementation Checklist

### Core Infrastructure
- [ ] Create `botticelli_bot_commands` crate
- [ ] Define `BotCommandExecutor` trait
- [ ] Implement `BotCommandError` types
- [ ] Create `BotCommandRegistry`
- [ ] Implement `CachedBotCommandExecutor` wrapper

### Discord Implementation
- [ ] Create `DiscordBotCommandExecutor`
- [ ] Implement `server.*` commands (5 commands)
- [ ] Implement `channels.*` commands (5 commands)
- [ ] Implement `members.*` commands (3 commands)
- [ ] Implement `roles.*` commands (2 commands)
- [ ] Implement `messages.*` commands (2 commands)

### Integration
- [ ] Add `bot_executors` field to `NarrativeExecutor`
- [ ] Implement `with_bot_executor()` builder
- [ ] Implement `with_discord_bot()` convenience builder
- [ ] Handle `Input::BotCommand` in `process_input()`
- [ ] Add `BotCommandError` to `NarrativeErrorKind`

### Testing
- [ ] Unit tests for trait and registry
- [ ] Unit tests for caching
- [ ] Mock executor for testing
- [ ] Integration tests with Discord API (ignored by default)
- [ ] End-to-end narrative execution tests

### Documentation
- [ ] API documentation for all public types
- [ ] Command reference documentation
- [ ] User guide for bot setup
- [ ] Example narratives using bot commands
- [ ] Security guidelines

### Security Audit
- [ ] Review all commands are read-only
- [ ] Validate input sanitization
- [ ] Test permission checks
- [ ] Verify rate limit handling
- [ ] Audit error messages (no sensitive data leakage)

## Timeline Estimate

- **Week 1**: Core infrastructure + trait design (20 hours)
- **Week 2**: Discord commands + caching (25 hours)
- **Week 3**: Integration + testing + docs (20 hours)

**Total**: ~65 hours over 3 weeks

## Dependencies

- `async-trait` - Async trait support
- `serde_json` - JSON handling
- `tokio` - Async runtime
- `tracing` - Logging
- `serenity` or `twilight` - Discord API (choose one)
- `reqwest` - HTTP client

## Example Usage (After Implementation)

```toml
[narrative]
name = "discord_analysis"
description = "Analyze Discord server activity"

[toc]
order = ["fetch_stats", "analyze"]

# Define bot command once
[bots.server_stats]
platform = "discord"
command = "server.get_stats"
guild_id = "1234567890"

[bots.channel_list]
platform = "discord"
command = "channels.list"
guild_id = "1234567890"

[acts]
# Execute bot commands
fetch_stats = ["bots.server_stats", "bots.channel_list"]

# Analyze results
analyze = """
Based on the server statistics and channel list from {{fetch_stats}},
provide insights on:
1. Community engagement levels
2. Recommended channel organization
3. Growth opportunities
"""
```

```rust
// Rust usage
let discord_client = DiscordClient::new(&token).await?;
let cache_ttl = std::time::Duration::from_secs(300); // 5 minutes

let executor = NarrativeExecutor::new(backend)
    .with_discord_bot(Arc::new(discord_client), cache_ttl)
    .build();

let result = executor.execute(&narrative).await?;
```

## Future Enhancements

- Slack command executor
- GitHub command executor (repos, issues, PRs)
- Twitter/X command executor (tweets, metrics)
- Analytics aggregation commands
- Webhook-based push notifications
- Command composition (pipe commands)
- Conditional command execution
- Command retry with exponential backoff
