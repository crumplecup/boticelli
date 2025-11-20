# Phase 2 Bot Commands - Follow-up Planning

## Current Status

✅ **Phase 2 Core Complete** (2024-11-20)

All integration tests passing against live Discord API. Bot command execution is production-ready.

**What's Working**:
- `BotCommandExecutor` trait with multi-platform support
- `BotCommandRegistry` for platform routing
- Discord integration with 3 working commands
- Narrative executor integration (bot commands → JSON → text → LLM)
- Comprehensive error handling with location tracking
- Full tracing instrumentation
- Real Discord API integration tests (5/5 passing)

**Commands Implemented**:
1. `server.get_stats` - Guild statistics (members, channels, boosts, etc.)
2. `channels.list` - List all channels in a guild
3. `roles.list` - List all roles with permissions

---

## Missing Discord Commands (High Priority)

### Category: Server Management

**`server.get_settings`** - Guild configuration details
- Verification level, explicit content filter
- Default notifications, afk timeout
- System channel settings
- Use case: Analyze server configuration for recommendations

**`server.get_features`** - Available guild features
- List enabled features (COMMUNITY, VERIFIED, etc.)
- Feature limits and tier information
- Use case: Generate content about server capabilities

### Category: Members

**`members.list`** - List guild members (paginated)
- Required args: `guild_id`, optional: `limit` (default 100)
- Returns: user info, join date, nickname, roles
- Use case: Analyze community composition, member activity

**`members.search`** - Search members by name
- Required args: `guild_id`, `query`
- Returns: matching members with fuzzy search
- Use case: Find specific users or analyze name patterns

**`members.get`** - Get specific member details
- Required args: `guild_id`, `user_id`
- Returns: full member profile, roles, join date, boost status
- Use case: Generate personalized content about specific members

**`members.count_by_role`** - Count members per role
- Required args: `guild_id`
- Returns: role name → member count mapping
- Use case: Role distribution analysis

### Category: Channels

**`channels.get`** - Get specific channel details
- Required args: `guild_id`, `channel_id`
- Returns: full channel metadata, permissions, topic
- Use case: Deep dive into channel configuration

**`channels.get_stats`** - Channel activity statistics
- Required args: `guild_id`, `channel_id`
- Returns: message count (last 24h), active users, last message time
- Use case: Identify most/least active channels
- **Note**: May require message history access

**`channels.list_threads`** - List active threads
- Required args: `guild_id`, `channel_id`
- Returns: thread metadata, activity, participant count
- Use case: Track discussion topics, thread engagement

### Category: Messages (Read-Only)

**`messages.recent`** - Fetch recent messages
- Required args: `guild_id`, `channel_id`, optional: `limit` (default 50)
- Returns: message content, author, timestamp, reactions
- Use case: Analyze recent discussions, sentiment analysis
- **Note**: Requires MESSAGE_CONTENT privileged intent

**`messages.count`** - Count messages in time window
- Required args: `guild_id`, `channel_id`, optional: `since` (ISO timestamp)
- Returns: message count, unique authors, time range
- Use case: Activity metrics without reading content

### Category: Roles & Permissions

**`roles.get`** - Get specific role details
- Required args: `guild_id`, `role_id`
- Returns: full role metadata, permission breakdown
- Use case: Permission auditing, role documentation

**`roles.hierarchy`** - Get role hierarchy visualization
- Required args: `guild_id`
- Returns: roles ordered by position with indentation
- Use case: Generate visual role structure

### Category: Invites & Growth

**`invites.list`** - List active invites
- Required args: `guild_id`
- Returns: invite codes, uses, inviter, expiration
- Use case: Track invite effectiveness, growth sources
- **Note**: Requires MANAGE_GUILD permission

**`invites.stats`** - Aggregate invite statistics
- Required args: `guild_id`
- Returns: total invites, top inviters, average uses
- Use case: Growth analysis, community building metrics

### Category: Emojis & Stickers

**`emojis.list`** - List custom emojis
- Required args: `guild_id`
- Returns: emoji name, ID, creator, usage restrictions
- Use case: Emoji catalog generation, usage patterns

**`stickers.list`** - List custom stickers
- Required args: `guild_id`
- Returns: sticker name, ID, description, format
- Use case: Sticker documentation

### Category: Events & Activities

**`events.list`** - List scheduled events
- Required args: `guild_id`
- Returns: event name, time, location, RSVP count
- Use case: Event calendar generation, attendance predictions

**`events.get`** - Get specific event details
- Required args: `guild_id`, `event_id`
- Returns: full event metadata, interested users count
- Use case: Event promotion content generation

---

## Performance & Caching Strategy

### Current State: No Caching

Every bot command execution hits the Discord API directly. This works for testing but has limitations:

**Problems**:
1. **Rate Limits**: Discord API has strict rate limits (50 requests/sec per bot)
2. **Latency**: Each command adds 100-500ms to narrative execution
3. **Cost**: Unnecessary API calls for data that changes infrequently (roles, channels)
4. **Quotas**: Narratives with many bot commands could hit daily limits

### Proposed: Tiered Caching Strategy

#### Tier 1: Static Data (1-24 hours TTL)

Data that rarely changes:
- **Channels**: Structure changes infrequently
- **Roles**: Permissions and hierarchy are stable
- **Emojis/Stickers**: Custom content rarely added/removed
- **Server settings**: Configuration changes are administrative

**Cache Key**: `{platform}:{guild_id}:{command}`  
**Invalidation**: TTL-based (1-6 hours), or manual via `cache_duration: None` in TOML

#### Tier 2: Semi-Dynamic Data (5-60 minutes TTL)

Data that changes moderately:
- **Member lists**: New joins are gradual
- **Invite stats**: Growth happens over hours/days
- **Channel stats**: Activity trends emerge slowly
- **Event lists**: Scheduled events don't change frequently

**Cache Key**: `{platform}:{guild_id}:{command}:{timestamp_bucket}`  
**Invalidation**: TTL-based (5-30 minutes)

#### Tier 3: Real-Time Data (No Cache)

Data that must be fresh:
- **Recent messages**: Discussions happen in real-time
- **Online member count**: Changes constantly
- **Active threads**: Thread activity is dynamic
- **Current events**: Need live RSVP counts

**Cache Key**: None - always fetch fresh  
**Invalidation**: N/A

### Implementation: CachedBotCommandExecutor

```rust
/// Wrapper that adds caching to any BotCommandExecutor
pub struct CachedBotCommandExecutor<E: BotCommandExecutor> {
    inner: E,
    cache: Arc<Mutex<LruCache<String, (JsonValue, Instant)>>>,
    default_ttl: Duration,
}

impl<E: BotCommandExecutor> CachedBotCommandExecutor<E> {
    pub fn new(executor: E, capacity: usize, default_ttl: Duration) -> Self {
        Self {
            inner: executor,
            cache: Arc::new(Mutex::new(LruCache::new(capacity))),
            default_ttl,
        }
    }
    
    fn cache_key(&self, command: &str, args: &HashMap<String, JsonValue>) -> String {
        // Create deterministic cache key from command + sorted args
        let mut key = format!("{}:{}", self.inner.platform(), command);
        let mut sorted_args: Vec<_> = args.iter().collect();
        sorted_args.sort_by_key(|(k, _)| *k);
        for (k, v) in sorted_args {
            key.push_str(&format!(":{}", v));
        }
        key
    }
}

#[async_trait]
impl<E: BotCommandExecutor> BotCommandExecutor for CachedBotCommandExecutor<E> {
    fn platform(&self) -> &str {
        self.inner.platform()
    }
    
    async fn execute(&self, command: &str, args: &HashMap<String, JsonValue>) 
        -> BotCommandResult<JsonValue> 
    {
        let cache_key = self.cache_key(command, args);
        
        // Check cache
        {
            let cache = self.cache.lock().await;
            if let Some((value, cached_at)) = cache.get(&cache_key) {
                if cached_at.elapsed() < self.default_ttl {
                    debug!(cache_key, age_secs = cached_at.elapsed().as_secs(), "Cache hit");
                    return Ok(value.clone());
                }
            }
        }
        
        // Cache miss - execute and store
        debug!(cache_key, "Cache miss, executing command");
        let result = self.inner.execute(command, args).await?;
        
        {
            let mut cache = self.cache.lock().await;
            cache.put(cache_key.clone(), (result.clone(), Instant::now()));
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

### Usage

```rust
// Create Discord executor
let discord = DiscordCommandExecutor::new(token);

// Wrap with caching (100 entries, 5 minute TTL)
let cached = CachedBotCommandExecutor::new(
    discord,
    100, // LRU cache capacity
    Duration::from_secs(300), // 5 minute default TTL
);

// Register cached executor
let mut registry = BotCommandRegistry::new();
registry.register(cached);
```

### TOML Cache Control

Allow per-command cache override:

```toml
[bots.server_stats]
platform = "discord"
command = "server.get_stats"
guild_id = "123456"
cache_duration = 300  # 5 minutes (seconds)

[bots.recent_messages]
platform = "discord"
command = "messages.recent"
guild_id = "123456"
cache_duration = 0  # Never cache (or null/None)
```

---

## Rate Limiting Awareness

### Discord Rate Limits

**Per-Route Limits**:
- Most endpoints: 5 requests per 5 seconds (1 req/sec)
- Global limit: 50 requests per second (shared across all endpoints)
- 429 responses include `Retry-After` header (seconds to wait)

**Current Implementation**: None - fires requests immediately

### Proposed: Rate-Aware Executor

```rust
pub struct RateLimitedBotCommandExecutor<E: BotCommandExecutor> {
    inner: E,
    limiter: Arc<RateLimiter>,
}

impl<E: BotCommandExecutor> RateLimitedBotCommandExecutor<E> {
    pub fn new(executor: E, requests_per_second: u32) -> Self {
        Self {
            inner: executor,
            limiter: Arc::new(RateLimiter::new(requests_per_second)),
        }
    }
}

#[async_trait]
impl<E: BotCommandExecutor> BotCommandExecutor for RateLimitedBotCommandExecutor<E> {
    // ... platform(), supports_command(), etc. delegate to inner
    
    async fn execute(&self, command: &str, args: &HashMap<String, JsonValue>) 
        -> BotCommandResult<JsonValue> 
    {
        // Wait for rate limit slot
        self.limiter.wait().await;
        
        // Execute with retry on 429
        let mut retries = 0;
        loop {
            match self.inner.execute(command, args).await {
                Ok(result) => return Ok(result),
                Err(e) if matches!(e.kind, BotCommandErrorKind::RateLimitExceeded { .. }) => {
                    if retries >= 3 {
                        return Err(e);
                    }
                    
                    if let BotCommandErrorKind::RateLimitExceeded { retry_after, .. } = e.kind {
                        warn!(retry_after, retries, "Rate limited, waiting");
                        tokio::time::sleep(Duration::from_secs(retry_after)).await;
                        retries += 1;
                    } else {
                        return Err(e);
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }
}
```

### Combining Cache + Rate Limit

```rust
// Create executor with both caching and rate limiting
let discord = DiscordCommandExecutor::new(token);
let cached = CachedBotCommandExecutor::new(discord, 100, Duration::from_secs(300));
let rate_limited = RateLimitedBotCommandExecutor::new(cached, 5); // 5 req/sec

registry.register(rate_limited);
```

---

## Additional Platforms

### Slack Integration (High Priority)

**Use Cases**:
- Workspace analytics (channels, members, messages)
- Thread sentiment analysis
- Knowledge base extraction from conversations
- Automated community insights

**Commands to Implement**:
1. `workspace.info` - Workspace name, plan, limits
2. `channels.list` - All channels (public + private bot is in)
3. `channels.history` - Recent messages from channel
4. `users.list` - Workspace members
5. `threads.list` - Active threads in channel
6. `reactions.get` - Reactions on recent messages

**Implementation**:
```rust
pub struct SlackCommandExecutor {
    http: Arc<reqwest::Client>,
    token: String,
}

#[async_trait]
impl BotCommandExecutor for SlackCommandExecutor {
    fn platform(&self) -> &str {
        "slack"
    }
    
    async fn execute(&self, command: &str, args: &HashMap<String, JsonValue>) 
        -> BotCommandResult<JsonValue> 
    {
        match command {
            "workspace.info" => self.workspace_info().await,
            "channels.list" => self.channels_list().await,
            // ...
        }
    }
}
```

### Telegram Integration (Medium Priority)

**Use Cases**:
- Channel/group analytics
- Message history analysis
- Bot interaction metrics
- Content performance tracking

**Commands to Implement**:
1. `chat.info` - Chat details (members, type, permissions)
2. `chat.members` - List chat members
3. `messages.recent` - Recent messages from chat
4. `messages.stats` - Message count, activity patterns

### Matrix Integration (Low Priority)

**Use Cases**:
- Federated community analytics
- Cross-server insights
- Open protocol advantages

**Commands to Implement**:
1. `room.info` - Room metadata
2. `room.members` - List room members
3. `messages.recent` - Recent room messages

---

## Security & Permissions

### Current State: Read-Only Operations

All implemented commands are **read-only** - they only fetch data, never modify anything.

**Implemented Safeguards**:
- No write operations (no channel creation, role assignment, message posting)
- No permission elevation
- No user data modification

### Future Write Operations (Phase 2.5?)

**Use Cases**:
- Automated community management (role assignment)
- Scheduled announcements (message posting)
- Dynamic channel creation (event-based)
- Reaction-based workflows

**Required Safeguards**:
1. **Explicit opt-in**: Write commands must be explicitly enabled
2. **Permission validation**: Check bot has required permissions before execution
3. **Audit logging**: All write operations logged with timestamp, user, action
4. **Rate limiting**: Stricter limits on write operations
5. **Dry-run mode**: Test narratives without side effects

**Example Write Commands** (NOT YET IMPLEMENTED):
- `channels.create` - Create new channel
- `roles.assign` - Assign role to member
- `messages.post` - Post message to channel
- `threads.create` - Start new thread
- `reactions.add` - Add reaction to message

**Security Review Required Before Implementation**

---

## Monitoring & Observability Enhancements

### Current State: Tracing Instrumentation

All bot command execution has structured tracing:
- Span fields: platform, command, arg_count, duration_ms, result_size
- Events: debug/info/warn/error at key points
- Location tracking in errors

### Proposed: Metrics & Dashboards

**Key Metrics to Track**:

1. **Execution Metrics**:
   - `bot_commands_total{platform, command, status}` - Total executions
   - `bot_commands_duration_seconds{platform, command}` - Latency histogram
   - `bot_commands_errors_total{platform, command, error_kind}` - Error counts

2. **Cache Metrics**:
   - `bot_commands_cache_hits_total{platform}` - Cache hit count
   - `bot_commands_cache_misses_total{platform}` - Cache miss count
   - `bot_commands_cache_evictions_total{platform}` - Eviction count
   - `bot_commands_cache_size{platform}` - Current cache entry count

3. **Rate Limit Metrics**:
   - `bot_commands_rate_limited_total{platform}` - 429 response count
   - `bot_commands_retry_total{platform}` - Retry attempt count
   - `bot_commands_waiting_seconds{platform}` - Time spent waiting for rate limit

4. **Usage Metrics**:
   - `bot_commands_unique_guilds{platform}` - Unique guild count
   - `bot_commands_narratives_using` - Narratives using bot commands
   - `bot_commands_most_used{command}` - Command popularity ranking

**Dashboard Views**:
- Real-time command execution rate (per platform)
- Error rate over time (grouped by error kind)
- Cache hit ratio (target: >80%)
- Rate limit impact (time spent waiting)
- Top commands by volume
- Slowest commands by latency (p50, p95, p99)

### Proposed: Health Checks

```rust
pub trait BotCommandExecutor {
    // Existing methods...
    
    /// Health check - verify bot can reach API
    async fn health_check(&self) -> BotCommandResult<HealthStatus>;
}

pub struct HealthStatus {
    pub healthy: bool,
    pub latency_ms: u64,
    pub authenticated: bool,
    pub rate_limit_remaining: Option<u32>,
    pub error: Option<String>,
}
```

**Use Cases**:
- Pre-flight checks before narrative execution
- Continuous monitoring of bot connectivity
- Alert on authentication failures
- Dashboard status indicators

---

## Testing Improvements

### Current State: Integration Tests

5 integration tests covering core functionality, all passing against live Discord API.

**Coverage**:
- ✅ Individual command execution (3 commands)
- ✅ Registry routing
- ✅ End-to-end narrative integration

**Gaps**:
- No tests for error conditions (missing permissions, invalid IDs)
- No tests for cache behavior
- No tests for rate limiting
- No tests for optional vs required commands
- No performance benchmarks

### Proposed: Expanded Test Suite

#### Error Condition Tests

```rust
#[tokio::test]
async fn test_invalid_guild_id() {
    let executor = DiscordCommandExecutor::new(token);
    let mut args = HashMap::new();
    args.insert("guild_id".to_string(), json!("invalid_id"));
    
    let result = executor.execute("server.get_stats", &args).await;
    assert!(matches!(result.unwrap_err().kind, 
        BotCommandErrorKind::InvalidArgument { .. }));
}

#[tokio::test]
async fn test_missing_permissions() {
    // Bot not in guild or lacks permissions
    let executor = DiscordCommandExecutor::new(token);
    let mut args = HashMap::new();
    args.insert("guild_id".to_string(), json!("999999999"));
    
    let result = executor.execute("channels.list", &args).await;
    assert!(matches!(result.unwrap_err().kind, 
        BotCommandErrorKind::PermissionDenied { .. } |
        BotCommandErrorKind::ResourceNotFound { .. }));
}

#[tokio::test]
async fn test_missing_required_argument() {
    let executor = DiscordCommandExecutor::new(token);
    let args = HashMap::new(); // Missing guild_id
    
    let result = executor.execute("server.get_stats", &args).await;
    assert!(matches!(result.unwrap_err().kind, 
        BotCommandErrorKind::MissingArgument { .. }));
}
```

#### Cache Behavior Tests

```rust
#[tokio::test]
async fn test_cache_hit_reduces_latency() {
    let executor = DiscordCommandExecutor::new(token);
    let cached = CachedBotCommandExecutor::new(
        executor, 10, Duration::from_secs(60)
    );
    
    let mut args = HashMap::new();
    args.insert("guild_id".to_string(), json!(guild_id));
    
    // First call - cache miss
    let start = Instant::now();
    cached.execute("server.get_stats", &args).await.unwrap();
    let first_duration = start.elapsed();
    
    // Second call - cache hit
    let start = Instant::now();
    cached.execute("server.get_stats", &args).await.unwrap();
    let second_duration = start.elapsed();
    
    // Cached call should be at least 10x faster
    assert!(second_duration < first_duration / 10);
}

#[tokio::test]
async fn test_cache_expiration() {
    let executor = DiscordCommandExecutor::new(token);
    let cached = CachedBotCommandExecutor::new(
        executor, 10, Duration::from_millis(100) // 100ms TTL
    );
    
    let mut args = HashMap::new();
    args.insert("guild_id".to_string(), json!(guild_id));
    
    // First call
    cached.execute("server.get_stats", &args).await.unwrap();
    
    // Wait for cache expiration
    tokio::time::sleep(Duration::from_millis(150)).await;
    
    // Second call should hit API again (cache expired)
    // Verify by checking tracing spans for "Cache miss" event
}
```

#### Optional vs Required Command Tests

```rust
#[tokio::test]
async fn test_required_command_failure_halts_narrative() {
    let executor = DiscordCommandExecutor::new(token);
    let mut registry = BotCommandRegistry::new();
    registry.register(executor);
    
    let narrative_executor = NarrativeExecutor::new(MockDriver)
        .with_bot_registry(Box::new(registry));
    
    // Narrative with required bot command that will fail
    let narrative = create_narrative_with_required_bot_command(
        "invalid_guild_id"
    );
    
    let result = narrative_executor.execute(&narrative).await;
    
    // Should return error and NOT execute subsequent acts
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), 
        BotticelliError::Narrative(NarrativeError { 
            kind: NarrativeErrorKind::BotCommandFailed(_), .. 
        })));
}

#[tokio::test]
async fn test_optional_command_failure_continues_narrative() {
    // Similar test but with required = false
    // Should continue execution with error message as text input
}
```

#### Performance Benchmarks

```rust
#[tokio::test]
#[ignore] // Only run for performance testing
async fn bench_command_execution_latency() {
    let executor = DiscordCommandExecutor::new(token);
    let mut args = HashMap::new();
    args.insert("guild_id".to_string(), json!(guild_id));
    
    let mut latencies = Vec::new();
    
    for _ in 0..10 {
        let start = Instant::now();
        executor.execute("server.get_stats", &args).await.unwrap();
        latencies.push(start.elapsed());
        
        // Respect rate limits
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    
    let avg = latencies.iter().sum::<Duration>() / latencies.len() as u32;
    let p95 = latencies[latencies.len() * 95 / 100];
    
    println!("Average latency: {:?}", avg);
    println!("P95 latency: {:?}", p95);
    
    // Assert reasonable latency (adjust based on Discord API performance)
    assert!(avg < Duration::from_millis(500), "Average latency too high");
    assert!(p95 < Duration::from_secs(1), "P95 latency too high");
}
```

---

## Documentation Updates

### User Guide: Bot Commands in Narratives

Create `docs/bot_commands_guide.md` covering:

1. **Introduction**:
   - What are bot commands?
   - Use cases and examples
   - Supported platforms (Discord, future: Slack, Telegram)

2. **Getting Started**:
   - Prerequisites (bot token, permissions)
   - First bot command narrative
   - Running with bot registry

3. **TOML Syntax Reference**:
   - `[bots.name]` section format
   - Required vs optional commands
   - Arguments and data types
   - Cache duration configuration

4. **Command Reference**:
   - Discord commands (with examples)
   - Argument descriptions
   - Response formats
   - Permission requirements

5. **Best Practices**:
   - When to use bot commands
   - Caching strategies
   - Rate limit awareness
   - Error handling in narratives

6. **Troubleshooting**:
   - Common errors and solutions
   - Permission issues
   - Rate limiting
   - Invalid guild/channel IDs

### API Documentation

Update `NARRATIVE_TOML_SPEC.md`:

**Add section**: Bot Command References

```markdown
## Bot Command References

Bot commands enable narratives to query social media platforms for real-time data.

### Defining Bot Commands

```toml
[bots.server_stats]
platform = "discord"           # Platform: discord, slack, telegram
command = "server.get_stats"   # Command to execute
guild_id = "1234567890"        # Platform-specific arguments
required = true                # Halt on error (default: true)
cache_duration = 300           # Cache result for 5 minutes (optional)
```

### Using Bot Commands

Reference bot commands in act inputs:

```toml
[acts]
fetch_data = ["bots.server_stats", "bots.channel_list"]
```

Bot commands are executed before the LLM call. JSON results are formatted as
text and passed to the LLM as context.

### Supported Platforms

#### Discord

Requires: DISCORD_TOKEN environment variable

Commands:
- `server.get_stats` - Guild statistics
- `channels.list` - List channels
- `roles.list` - List roles

See `docs/bot_commands_guide.md` for full command reference.
```

### Code Documentation

Improve rustdoc coverage:

**Missing docs** (from warnings):
- `BotCommandErrorKind` field documentation
- `BotCommandError` field documentation
- All Discord model struct fields (244 warnings!)

**Priority**: Document Discord models to eliminate warnings

---

## Implementation Roadmap

### Short Term (1-2 weeks)

**Priority 1: Essential Commands**
- [ ] `members.list` - High value for community analysis
- [ ] `members.count_by_role` - Quick role distribution
- [ ] `channels.get` - Deep dive into specific channel

**Priority 2: Caching**
- [ ] Implement `CachedBotCommandExecutor`
- [ ] Add cache hit/miss tracing
- [ ] Test cache behavior with integration tests
- [ ] Document cache configuration in TOML

**Priority 3: Documentation**
- [ ] Fix missing rustdoc warnings (244 warnings!)
- [ ] Create bot commands user guide
- [ ] Update NARRATIVE_TOML_SPEC.md
- [ ] Add examples for each command

### Medium Term (3-4 weeks)

**Priority 1: More Commands**
- [ ] `messages.recent` - High value but needs MESSAGE_CONTENT intent
- [ ] `messages.count` - Activity metrics without content access
- [ ] `emojis.list` - Custom emoji documentation
- [ ] `events.list` - Scheduled events

**Priority 2: Rate Limiting**
- [ ] Implement `RateLimitedBotCommandExecutor`
- [ ] Handle 429 responses with retry
- [ ] Add rate limit tracing
- [ ] Test rate limiting behavior

**Priority 3: Slack Integration**
- [ ] Create `SlackCommandExecutor`
- [ ] Implement core commands (workspace.info, channels.list)
- [ ] Integration tests with Slack API
- [ ] Documentation

### Long Term (1-2 months)

**Priority 1: Monitoring**
- [ ] Metrics collection (Prometheus format)
- [ ] Grafana dashboard templates
- [ ] Health check endpoints
- [ ] Alerting on failures

**Priority 2: Advanced Features**
- [ ] Telegram integration
- [ ] Matrix integration
- [ ] Command result formatting options (markdown, CSV)
- [ ] Batch command execution

**Priority 3: Write Operations** (Security Review Required)
- [ ] Design permission model
- [ ] Audit logging infrastructure
- [ ] Dry-run mode
- [ ] Implement safe write commands

---

## Success Metrics

### Phase 2 Success Criteria

✅ **Achieved**:
- [x] Bot command infrastructure implemented
- [x] Discord integration with 3 commands
- [x] Narrative integration working
- [x] Integration tests passing (5/5)
- [x] Error handling with location tracking
- [x] Comprehensive tracing

### Phase 2.1 Goals

**Technical**:
- [ ] 10+ Discord commands implemented
- [ ] Cache hit ratio > 80%
- [ ] P95 latency < 500ms (with cache)
- [ ] Zero rustdoc warnings
- [ ] 20+ integration tests

**User Experience**:
- [ ] User guide published
- [ ] 5+ example narratives using bot commands
- [ ] Error messages clear and actionable
- [ ] Cache behavior transparent and configurable

### Phase 2.5 Goals (Optional)

**Expansion**:
- [ ] Slack integration (5+ commands)
- [ ] Telegram integration (3+ commands)
- [ ] Multi-platform narratives (Discord + Slack)
- [ ] 100+ integration tests across platforms

**Production Readiness**:
- [ ] Metrics & dashboards deployed
- [ ] Health checks integrated
- [ ] Rate limiting tested under load
- [ ] Write operations available (if security review passes)

---

## Conclusion

Phase 2 bot command execution is **production-ready** for read-only operations. The architecture is solid, tested, and extensible.

**Immediate next steps**:
1. Implement caching to improve performance
2. Add essential member and message commands
3. Fix documentation warnings
4. Create user guide

**Future enhancements**:
1. Slack and Telegram integrations
2. Rate limiting and metrics
3. Write operations (with security review)

The foundation is strong. Now we build on it.

---

**Document Version**: 1.0  
**Last Updated**: 2024-11-20  
**Status**: Planning  
**Related Docs**: `PHASE_2_BOT_COMMANDS.md`, `NARRATIVE_TOML_SPEC.md`
