# Phase 2: Bot Command Execution - Detailed Implementation Plan

## Overview

Phase 2 adds the ability to execute bot commands (Discord, Slack, etc.) from narrative acts. This enables narratives to fetch real-time data from platforms, analyze it, and incorporate the results into content generation workflows.

## Goals

1. **Execute bot commands from narratives** - Turn `Input::BotCommand` into actual API calls
2. **Platform abstraction** - Support multiple platforms (Discord primary, others later)
3. **Caching & performance** - Cache results, respect rate limits
4. **Error handling** - Graceful failures with helpful context
5. **Security** - Read-only operations, proper permissions
6. **Observability** - Comprehensive tracing for debugging and monitoring

## Architecture

### Tracing & Observability Strategy

**MANDATORY:** All bot command execution must have comprehensive tracing instrumentation.

#### Span Naming Convention

Use hierarchical span names following the pattern: `component.operation`

- `bot_commands.execute` - Top-level command execution
- `bot_commands.cache_lookup` - Cache check operations
- `discord.api_call` - Discord API HTTP requests
- `discord.parse_response` - Response parsing
- `bot_commands.validate_args` - Argument validation

#### Required Instrumentation Points

1. **All public functions** - Use `#[instrument]` macro
2. **Command execution** - Span with command name, platform, args count
3. **Cache operations** - Log hits, misses, expirations
4. **API calls** - Log endpoint, method, response status
5. **Error paths** - Log error context before returning
6. **Performance** - Log operation duration for slow operations (>100ms)

#### Structured Fields

Capture relevant context in span fields:
```rust
#[instrument(
    skip(self, args),
    fields(
        platform = %self.platform(),
        command,
        arg_count = args.len(),
        result_size,
        cache_hit,
        duration_ms
    )
)]
```

Common fields to track:
- `platform` - Discord, Slack, etc.
- `command` - Command name being executed
- `guild_id` / `channel_id` - Discord IDs
- `arg_count` - Number of arguments
- `result_size` - Size of result in bytes
- `cache_hit` - Whether result came from cache
- `duration_ms` - Operation duration
- `error_kind` - Type of error if failed

### Trait Design

```rust
/// Executes bot commands for a specific platform.
///
/// Implementations handle the platform-specific API calls and return
/// structured JSON results that can be converted to text for LLM consumption.
///
/// # Tracing
/// All implementations MUST instrument the `execute` method with:
/// - `#[instrument]` macro
/// - Span fields: platform, command, arg_count
/// - Debug events for key operations
/// - Error events with context
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
    ///
    /// # Tracing
    /// Must emit:
    /// - info! at start with command name
    /// - debug! for validation steps
    /// - error! if execution fails with full context
    /// - Record result_size in span
    #[instrument(
        skip(self, args),
        fields(
            platform = %self.platform(),
            command,
            arg_count = args.len(),
            result_size
        )
    )]
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
        tracing::debug!("Creating new BotCommandRegistry");
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
        tracing::info!(
            platform = %platform,
            commands = executor.supported_commands().len(),
            "Registering bot command executor"
        );
        self.executors.insert(platform, Arc::new(executor));
        self
    }
    
    /// Get executor for a platform.
    pub fn get(&self, platform: &str) -> Option<&Arc<dyn BotCommandExecutor>> {
        self.executors.get(platform)
    }
    
    /// Execute a command on a platform.
    #[instrument(
        skip(self, args),
        fields(
            platform,
            command,
            arg_count = args.len()
        )
    )]
    pub async fn execute(
        &self,
        platform: &str,
        command: &str,
        args: &HashMap<String, serde_json::Value>,
    ) -> BotCommandResult<serde_json::Value> {
        tracing::info!("Executing bot command via registry");
        
        let executor = self
            .get(platform)
            .ok_or_else(|| {
                tracing::error!(
                    platform,
                    available_platforms = ?self.platforms(),
                    "Platform not found in registry"
                );
                BotCommandError::new(
                    BotCommandErrorKind::PlatformNotFound(platform.to_string())
                )
            })?;
        
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
        tracing::info!(
            platform = %executor.platform(),
            ttl_secs = default_ttl.as_secs(),
            "Creating cached bot command executor"
        );
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
    
    #[instrument(
        skip(self, args),
        fields(
            platform = %self.platform(),
            command,
            arg_count = args.len(),
            cache_hit,
            cache_age_secs
        )
    )]
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
                    let age_secs = cached.cached_at.elapsed().as_secs();
                    tracing::Span::current().record("cache_hit", true);
                    tracing::Span::current().record("cache_age_secs", age_secs);
                    tracing::info!(age_secs, "Cache hit for bot command");
                    return Ok(cached.result.clone());
                } else {
                    tracing::debug!("Cached result expired, re-executing");
                }
            }
        }
        
        // Cache miss - execute command
        tracing::Span::current().record("cache_hit", false);
        tracing::info!("Cache miss, executing bot command");
        let start = std::time::Instant::now();
        let result = self.inner.execute(command, args).await?;
        let duration_ms = start.elapsed().as_millis();
        
        tracing::debug!(duration_ms, "Bot command executed");
        
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
            tracing::debug!(cache_size = cache.len(), "Result cached");
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
    #[instrument(skip(client), fields(commands_registered))]
    pub fn new(client: Arc<DiscordClient>) -> Self {
        tracing::info!("Creating Discord bot command executor");
        
        let mut executor = Self {
            client,
            commands: HashMap::new(),
        };
        
        executor.register_server_commands();
        executor.register_channel_commands();
        executor.register_member_commands();
        executor.register_role_commands();
        executor.register_message_commands();
        
        tracing::Span::current().record("commands_registered", executor.commands.len());
        tracing::info!(
            command_count = executor.commands.len(),
            "Discord executor initialized"
        );
        
        executor
    }
    
    fn register_server_commands(&mut self) {
        tracing::debug!("Registering server commands");
        
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
                        let span = tracing::info_span!(
                            "discord.server_get_stats",
                            guild_id,
                            member_count,
                            channel_count
                        );
                        let _enter = span.enter();
                        
                        tracing::debug!("Parsing guild_id argument");
                        let guild_id = args
                            .get("guild_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                tracing::error!("Missing guild_id argument");
                                BotCommandError::new(
                                    BotCommandErrorKind::MissingArgument {
                                        command: "server.get_stats".to_string(),
                                        arg_name: "guild_id".to_string(),
                                    }
                                )
                            })?;
                        
                        let guild_id: u64 = guild_id
                            .parse()
                            .map_err(|_| {
                                tracing::error!(guild_id, "Invalid guild_id format");
                                BotCommandError::new(
                                    BotCommandErrorKind::InvalidArgument {
                                        command: "server.get_stats".to_string(),
                                        arg_name: "guild_id".to_string(),
                                        reason: "Invalid guild ID format".to_string(),
                                    }
                                )
                            })?;
                        
                        tracing::Span::current().record("guild_id", guild_id);
                        tracing::info!("Fetching guild stats from Discord API");
                        
                        let stats = client.get_guild_stats(guild_id).await.map_err(|e| {
                            tracing::error!(error = %e, "Failed to fetch guild stats");
                            e
                        })?;
                        
                        tracing::Span::current().record("member_count", stats.member_count);
                        tracing::Span::current().record("channel_count", stats.channel_count);
                        tracing::info!(
                            member_count = stats.member_count,
                            channel_count = stats.channel_count,
                            "Successfully retrieved guild stats"
                        );
                        
                        Ok(serde_json::to_value(stats).map_err(|e| {
                            tracing::error!(error = %e, "Failed to serialize stats");
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
        tracing::debug!(count = 1, "Server commands registered");
    }
}

#[async_trait]
impl BotCommandExecutor for DiscordBotCommandExecutor {
    fn platform(&self) -> &str {
        "discord"
    }
    
    #[instrument(
        skip(self, args),
        fields(
            platform = "discord",
            command,
            arg_count = args.len(),
            result_size,
            duration_ms
        )
    )]
    async fn execute(
        &self,
        command: &str,
        args: &HashMap<String, serde_json::Value>,
    ) -> BotCommandResult<serde_json::Value> {
        tracing::info!("Executing Discord bot command");
        
        let cmd_def = self
            .commands
            .get(command)
            .ok_or_else(|| {
                tracing::error!(
                    command,
                    available_commands = ?self.supported_commands(),
                    "Command not found"
                );
                BotCommandError::new(
                    BotCommandErrorKind::CommandNotFound(command.to_string())
                )
            })?;
        
        tracing::debug!(
            required_args = ?cmd_def.required_args,
            optional_args = ?cmd_def.optional_args,
            "Validating command arguments"
        );
        
        // Validate required arguments
        for required_arg in &cmd_def.required_args {
            if !args.contains_key(required_arg) {
                tracing::error!(
                    command,
                    missing_arg = required_arg,
                    "Missing required argument"
                );
                return Err(BotCommandError::new(
                    BotCommandErrorKind::MissingArgument {
                        command: command.to_string(),
                        arg_name: required_arg.clone(),
                    }
                ));
            }
        }
        
        tracing::debug!("Argument validation passed, executing handler");
        let start = std::time::Instant::now();
        
        // Execute command
        let result = (cmd_def.handler)(&self.client, args).await?;
        
        let duration_ms = start.elapsed().as_millis();
        let result_size = serde_json::to_string(&result)
            .map(|s| s.len())
            .unwrap_or(0);
        
        tracing::Span::current().record("duration_ms", duration_ms);
        tracing::Span::current().record("result_size", result_size);
        tracing::info!(
            duration_ms,
            result_size,
            "Discord command executed successfully"
        );
        
        Ok(result)
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
        tracing::info!(
            platform = %platform,
            commands = executor.supported_commands().len(),
            "Registering bot executor with narrative executor"
        );
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
        tracing::info!(
            cache_ttl_secs = cache_ttl.as_secs(),
            "Adding Discord bot executor with caching"
        );
        let executor = DiscordBotCommandExecutor::new(discord_client);
        let cached_executor = CachedBotCommandExecutor::new(executor, cache_ttl);
        self.with_bot_executor(cached_executor)
    }
}
```

### Input Processing

```rust
impl NarrativeExecutor {
    #[instrument(
        skip(self, input),
        fields(
            input_type,
            platform,
            command,
            required,
            result_length,
            success
        )
    )]
    async fn process_input(&self, input: &Input) -> Result<String, NarrativeError> {
        match input {
            Input::Text(content) => {
                tracing::Span::current().record("input_type", "text");
                Ok(content.clone())
            }
            
            Input::BotCommand {
                platform,
                command,
                args,
                required,
                cache_duration,
            } => {
                tracing::Span::current().record("input_type", "bot_command");
                tracing::Span::current().record("platform", platform.as_str());
                tracing::Span::current().record("command", command.as_str());
                tracing::Span::current().record("required", *required);
                
                tracing::info!(
                    platform,
                    command,
                    arg_count = args.len(),
                    required,
                    "Processing bot command input"
                );
                
                let executor = self
                    .bot_executors
                    .get(platform)
                    .ok_or_else(|| {
                        tracing::error!(
                            platform,
                            available_platforms = ?self.bot_executors.keys().collect::<Vec<_>>(),
                            "No executor registered for platform"
                        );
                        NarrativeError::new(
                            NarrativeErrorKind::BotCommandError(format!(
                                "No executor registered for platform: {}",
                                platform
                            ))
                        )
                    })?;
                
                tracing::debug!("Executor found, executing command");
                
                match executor.execute(command, args).await {
                    Ok(result) => {
                        tracing::debug!("Command executed, serializing result");
                        
                        // Convert JSON result to pretty-printed string for LLM
                        let text = serde_json::to_string_pretty(&result)
                            .map_err(|e| {
                                tracing::error!(error = %e, "Failed to serialize result");
                                NarrativeError::new(
                                    NarrativeErrorKind::BotCommandError(
                                        format!("Failed to serialize result: {}", e)
                                    )
                                )
                            })?;
                        
                        tracing::Span::current().record("result_length", text.len());
                        tracing::Span::current().record("success", true);
                        tracing::info!(
                            result_length = text.len(),
                            "Bot command executed successfully"
                        );
                        
                        Ok(text)
                    }
                    Err(e) => {
                        if *required {
                            tracing::Span::current().record("success", false);
                            tracing::error!(
                                command,
                                error = %e,
                                "Required bot command failed, halting execution"
                            );
                            // Halt execution if command is required
                            Err(NarrativeError::new(
                                NarrativeErrorKind::BotCommandError(
                                    format!("Required command failed: {}", e)
                                )
                            ))
                        } else {
                            tracing::Span::current().record("success", false);
                            // Continue with error message as context
                            tracing::warn!(
                                command,
                                error = %e,
                                "Optional bot command failed, continuing with error message"
                            );
                            let error_msg = format!("[Bot command '{}' failed: {}]", command, e);
                            tracing::Span::current().record("result_length", error_msg.len());
                            Ok(error_msg)
                        }
                    }
                }
            }
            
            // ... other input types
        }
    }
}
```

## Tracing Output Examples

### Successful Command Execution

```
INFO bot_commands.execute{platform="discord" command="server.get_stats" arg_count=1}: Executing bot command via registry
INFO discord.execute{platform="discord" command="server.get_stats" arg_count=1}: Executing Discord bot command
DEBUG discord.execute: Validating command arguments required_args=["guild_id"] optional_args=[]
DEBUG discord.execute: Argument validation passed, executing handler
INFO discord.server_get_stats{guild_id=1234567890}: Fetching guild stats from Discord API
INFO discord.server_get_stats{guild_id=1234567890 member_count=1250 channel_count=45}: Successfully retrieved guild stats member_count=1250 channel_count=45
INFO discord.execute{duration_ms=234 result_size=543}: Discord command executed successfully duration_ms=234 result_size=543
INFO bot_commands.execute{cache_hit=false}: Cache miss, executing bot command
DEBUG bot_commands.execute: Bot command executed duration_ms=235
DEBUG bot_commands.execute: Result cached cache_size=1
INFO narrative.process_input{input_type="bot_command" platform="discord" command="server.get_stats" required=true result_length=543 success=true}: Bot command executed successfully result_length=543
```

### Cache Hit

```
INFO bot_commands.execute{platform="discord" command="server.get_stats" arg_count=1 cache_hit=true cache_age_secs=45}: Cache hit for bot command age_secs=45
INFO narrative.process_input{result_length=543 success=true}: Bot command executed successfully result_length=543
```

### Failed Required Command

```
INFO bot_commands.execute{platform="discord" command="server.get_stats" arg_count=0}: Executing bot command via registry
ERROR discord.execute: Missing required argument command="server.get_stats" missing_arg="guild_id"
ERROR narrative.process_input{success=false}: Required bot command failed, halting execution command="server.get_stats" error="Bot Command Error: Missing required argument 'guild_id' for command 'server.get_stats' at line 234 in bot_commands.rs"
```

### Failed Optional Command

```
INFO bot_commands.execute{platform="discord" command="channels.list" arg_count=1}: Executing bot command via registry
ERROR discord.api_call: API call failed status=403 reason="Missing permissions"
WARN narrative.process_input{success=false result_length=78}: Optional bot command failed, continuing with error message command="channels.list" error="Permission denied"
```

### Platform Not Found

```
INFO bot_commands.execute{platform="slack" command="channels.list" arg_count=1}: Executing bot command via registry
ERROR bot_commands.execute: Platform not found in registry platform="slack" available_platforms=["discord"]
ERROR narrative.process_input: No executor registered for platform platform="slack" available_platforms=["discord"]
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
- [ ] Define `BotCommandExecutor` trait with `#[instrument]`
- [ ] Implement `BotCommandError` types
- [ ] Create `BotCommandRegistry` with tracing
- [ ] Implement `CachedBotCommandExecutor` wrapper with cache metrics

### Discord Implementation
- [ ] Create `DiscordBotCommandExecutor`
- [ ] Implement `server.*` commands (5 commands) with tracing
- [ ] Implement `channels.*` commands (5 commands) with tracing
- [ ] Implement `members.*` commands (3 commands) with tracing
- [ ] Implement `roles.*` commands (2 commands) with tracing
- [ ] Implement `messages.*` commands (2 commands) with tracing
- [ ] Add tracing to all command handlers

### Integration
- [ ] Add `bot_executors` field to `NarrativeExecutor`
- [ ] Implement `with_bot_executor()` builder with tracing
- [ ] Implement `with_discord_bot()` convenience builder with tracing
- [ ] Handle `Input::BotCommand` in `process_input()` with full instrumentation
- [ ] Add `BotCommandError` to `NarrativeErrorKind`

### Tracing & Observability
- [ ] Add `#[instrument]` to all public functions
- [ ] Use span fields for command, platform, args
- [ ] Record result_size in spans
- [ ] Record duration_ms for operations
- [ ] Log cache hits/misses with metrics
- [ ] Log error context before returning
- [ ] Emit info! events for major operations
- [ ] Emit debug! events for detailed flow
- [ ] Emit error! events with full context
- [ ] Test tracing output in unit tests

### Testing
- [ ] Unit tests for trait and registry
- [ ] Unit tests for caching with trace verification
- [ ] Mock executor for testing
- [ ] Integration tests with Discord API (ignored by default)
- [ ] End-to-end narrative execution tests
- [ ] Verify tracing spans in tests

### Documentation
- [ ] API documentation for all public types
- [ ] Command reference documentation
- [ ] User guide for bot setup
- [ ] Example narratives using bot commands
- [ ] Security guidelines
- [ ] Tracing best practices documentation

### Security Audit
- [ ] Review all commands are read-only
- [ ] Validate input sanitization
- [ ] Test permission checks
- [ ] Verify rate limit handling
- [ ] Audit error messages (no sensitive data leakage)
- [ ] Verify no sensitive data in trace logs

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
