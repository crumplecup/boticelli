# Security Framework Guide

## Overview

The Botticelli security framework provides multi-layer protection for AI-controlled bot operations. It prevents AI hallucinations, prompt injection, privilege escalation, and abuse while maintaining comprehensive audit trails.

## Architecture

### 5-Layer Security Pipeline

Every bot command passes through 5 sequential security layers:

```
Bot Command Request
  ↓
1. Permission Layer → Command & resource authorization
  ↓
2. Validation Layer → Input format & type validation
  ↓
3. Content Filter Layer → Pattern detection & filtering
  ↓
4. Rate Limit Layer → Token bucket rate limiting
  ↓
5. Approval Layer → Human-in-the-loop for dangerous ops
  ↓
Execute Command
```

**Defense-in-depth:** Each layer provides independent protection. If one layer fails open, others still provide security.

## Layer Details

### Layer 1: Permission Checking

**Purpose:** Authorize commands and resources before execution.

**Features:**
- Per-narrative command allowlists
- Resource-based access control (channels, users, roles)
- Protected user/role lists (prevent targeting admins)
- Unknown resource defaults to deny

**Configuration:**
```toml
[security.permissions]
allowed_commands = [
    "discord.messages.send",
    "discord.channels.list",
    "discord.server.get_stats"
]

[security.permissions.resources.channel]
allowed_ids = ["123456789012345678", "987654321098765432"]

[security.permissions.protected_users]
user_ids = ["111111111111111111"]  # Bot owner
```

**Example Violations:**
- ❌ Command not in allowlist
- ❌ Channel not in resource allowlist
- ❌ Attempting to target protected user

### Layer 2: Input Validation

**Purpose:** Validate input format, types, and ranges before use.

**Platform Validators:**
- **Discord**: Snowflake IDs, channel names, content length
- **Slack**: (future) Team IDs, channel names
- **Telegram**: (future) Chat IDs, user IDs

**Discord Validation Rules:**
- Snowflake IDs: 17-19 digit integers
- Channel names: 1-100 chars, lowercase alphanumeric + `-_`
- Message content: 1-2000 characters
- Role names: 1-100 characters

**Example Violations:**
- ❌ Invalid snowflake format ("abc" instead of "123456789012345678")
- ❌ Message too long (>2000 chars)
- ❌ Invalid channel name (uppercase, spaces, special chars)

### Layer 3: Content Filtering

**Purpose:** Detect and block malicious or abusive content patterns.

**Detection Rules:**
- Mass mentions (`@everyone`, `@here`)
- Prohibited patterns (regex-based)
- URL counting and domain restrictions
- Custom filter rules per platform

**Configuration:**
```toml
[security.content_filter]
# Block mass mentions
block_everyone = true
block_here = true

# Prohibited patterns (regex)
prohibited_patterns = [
    "password:\\s*\\S+",
    "api[_-]?key:\\s*\\S+",
    "(?i)credit[\\s-]?card"
]

# URL restrictions
max_urls = 3
allowed_domains = ["github.com", "docs.rs"]
```

**Example Violations:**
- ❌ `@everyone spam` in message content
- ❌ `password: hunter2` in text
- ❌ 5 URLs when `max_urls = 3`
- ❌ `evil.com` when not in `allowed_domains`

### Layer 4: Rate Limiting

**Purpose:** Prevent abuse through excessive API usage.

**Algorithm:** Token bucket with configurable capacity and refill rate.

**Configuration:**
```toml
[security.rate_limits]
# Format: command = { limit = tokens, window_secs = seconds }
"discord.messages.send" = { limit = 10, window_secs = 60 }
"discord.channels.create" = { limit = 2, window_secs = 3600 }
```

**Token Bucket Parameters:**
- `limit` - Token capacity (max burst)
- `window_secs` - Refill interval
- Refill rate: `limit / window_secs` tokens per second

**Examples:**
- `{ limit = 10, window_secs = 60 }` → 10 messages/min, refills at 1 token every 6s
- `{ limit = 2, window_secs = 3600 }` → 2 channels/hour, refills at 1 token every 30min

**Violation:**
- ❌ 11th message in 60 seconds when `limit = 10`

### Layer 5: Approval Workflow

**Purpose:** Human-in-the-loop for dangerous operations.

**Features:**
- Configurable per-command approval requirements
- Pending action tracking with 24-hour expiration
- Approval/denial with reason tracking
- Unique action IDs for audit trails

**Configuration:**
```toml
[security.approval]
# Commands requiring human approval
requires_approval = [
    "discord.channels.delete",
    "discord.members.ban",
    "discord.roles.delete"
]
```

**Workflow:**

1. **AI requests dangerous operation**
   ```rust
   let result = executor.execute_secure(
       "narrative1", 
       "discord", 
       "members.ban",
       &args
   ).await?;
   ```

2. **System creates pending action**
   ```rust
   match result {
       ExecutionResult::ApprovalRequired(action_id) => {
           // Notify admin: "Action {action_id} requires approval"
       }
       ExecutionResult::Success(_) => { /* ... */ }
   }
   ```

3. **Human reviews and approves**
   ```rust
   approval_workflow.approve_action(
       &action_id,
       "admin_user_id",
       Some("Reviewed, looks legitimate")
   )?;
   ```

4. **AI retries with approval**
   ```rust
   args.insert("_approval_action_id", action_id);
   let result = executor.execute_secure("narrative1", "discord", "members.ban", &args).await?;
   // ✅ Executes because approval granted
   ```

**Action Expiration:**
- Pending actions expire after 24 hours by default
- Expired actions cannot be approved
- Cleanup via `approval_workflow.cleanup_expired()`

## Usage

### Basic Setup

```rust
use botticelli_security::{
    PermissionConfig, PermissionChecker, DiscordValidator,
    ContentFilter, ContentFilterConfig, RateLimiter, RateLimit,
    ApprovalWorkflow, SecureExecutor,
};
use botticelli_social::{BotCommandRegistryImpl, SecureBotCommandExecutor};

// 1. Configure permissions
let perm_config = PermissionConfig::new()
    .with_allowed_commands(vec![
        "discord.messages.send".to_string(),
        "discord.channels.list".to_string(),
    ])
    .with_resources({
        let mut resources = HashMap::new();
        resources.insert(
            "channel".to_string(),
            ResourcePermission::new()
                .with_allowed_ids(vec!["123456789012345678".to_string()])
        );
        resources
    });

let permission_checker = PermissionChecker::new(perm_config);

// 2. Create validator
let validator = DiscordValidator::new();

// 3. Configure content filter
let content_filter = ContentFilter::new(
    ContentFilterConfig::default()
        .with_block_everyone(true)
        .with_block_here(true)
)?;

// 4. Configure rate limits
let mut rate_limiter = RateLimiter::new();
rate_limiter.add_limit("discord.messages.send", RateLimit::strict(10, 60));

// 5. Create approval workflow
let approval_workflow = ApprovalWorkflow::new();

// 6. Create bot command registry
let mut registry = BotCommandRegistryImpl::new();
registry.register(discord_executor);

// 7. Wrap with secure executor
let secure_executor = SecureBotCommandExecutor::new(
    registry,
    permission_checker,
    validator,
    content_filter,
    rate_limiter,
    approval_workflow,
);
```

### Executing Commands

```rust
use botticelli_social::ExecutionResult;

let mut args = HashMap::new();
args.insert("channel_id", serde_json::json!("123456789012345678"));
args.insert("content", serde_json::json!("Hello, world!"));

match secure_executor.execute_secure("narrative1", "discord", "messages.send", &args).await? {
    ExecutionResult::Success(result) => {
        println!("Message sent: {:?}", result);
    }
    ExecutionResult::ApprovalRequired(action_id) => {
        println!("Approval required: {}", action_id);
        // Notify admin, wait for approval, retry
    }
}
```

### Handling Approvals

```rust
// List pending actions
let pending = secure_executor
    .approval_workflow()
    .list_pending_actions("narrative1");

for action in pending {
    println!("Pending: {} - {} ({})", 
        action.id(), 
        action.command(), 
        action.reason().as_deref().unwrap_or("No reason")
    );
}

// Approve an action
secure_executor
    .approval_workflow()
    .approve_action(&action_id, "admin_user", Some("Looks good"))?;

// Deny an action
secure_executor
    .approval_workflow()
    .deny_action(&action_id, "admin_user", Some("Too risky"))?;
```

### Configuration from TOML

```rust
use std::fs;
use toml;

#[derive(Deserialize)]
struct SecurityConfig {
    permissions: PermissionConfig,
    content_filter: ContentFilterConfig,
    rate_limits: HashMap<String, RateLimit>,
    approval: ApprovalConfig,
}

let config_str = fs::read_to_string("security.toml")?;
let config: SecurityConfig = toml::from_str(&config_str)?;

// Use config to build security components
```

## Error Handling

### Error Types

```rust
pub enum SecurityErrorKind {
    PermissionDenied { command, reason },
    ResourceAccessDenied { resource, reason },
    ValidationFailed { field, reason },
    ContentViolation { reason },
    RateLimitExceeded { operation, reason, limit, window_secs },
    ApprovalRequired { operation, reason, action_id },
    ApprovalDenied { action_id, reason },
    Configuration(String),
}
```

### Error Conversion

`SecureBotCommandExecutor` converts `SecurityError` to `BotCommandError`:

```rust
SecurityError::PermissionDenied → BotCommandError::PermissionDenied
SecurityError::ValidationFailed → BotCommandError::InvalidArgument
SecurityError::ContentViolation → BotCommandError::InvalidArgument
SecurityError::RateLimitExceeded → BotCommandError::RateLimitExceeded
```

### Handling Errors

```rust
match secure_executor.execute_secure(narrative_id, platform, command, &args).await {
    Ok(ExecutionResult::Success(result)) => {
        // Command executed successfully
    }
    Ok(ExecutionResult::ApprovalRequired(action_id)) => {
        // Human approval needed
    }
    Err(BotCommandError { kind: BotCommandErrorKind::PermissionDenied { command, reason }, .. }) => {
        // Permission check failed
    }
    Err(BotCommandError { kind: BotCommandErrorKind::RateLimitExceeded { retry_after, .. }, .. }) => {
        // Rate limit exceeded, retry after `retry_after` seconds
    }
    Err(e) => {
        // Other errors
    }
}
```

## Observability

### Tracing

All security layers emit structured tracing events:

```rust
use tracing_subscriber;

// Enable debug logging
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

**Span hierarchy:**
```
botticelli_social.execute_secure
  ├─ security_pipeline
  │   ├─ layer_1_permission_check
  │   ├─ layer_2_validation
  │   ├─ layer_3_content_filter
  │   ├─ layer_4_rate_limit
  │   └─ layer_5_approval
  └─ bot_command.execute
```

**Key events:**
- `info!("Starting security pipeline")`
- `debug!("Layer 1: Checking permissions")`
- `warn!(action_id, "Approval required for command")`
- `error!("Validation failed: {}", reason)`

### Audit Trail

Enable audit logging for compliance:

```rust
use tracing_subscriber::{fmt, EnvFilter};
use tracing_appender;

let file_appender = tracing_appender::rolling::daily("./logs", "security-audit.log");
let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

tracing_subscriber::fmt()
    .with_writer(non_blocking)
    .with_ansi(false)
    .with_env_filter(EnvFilter::from_default_env())
    .init();
```

**Audit log entries include:**
- Timestamp
- Narrative ID
- Command name
- Arguments (sanitized)
- Security decision (allow/deny)
- Layer that made decision
- Reason for denial
- Approval actions (create, approve, deny)

## Testing

### Unit Tests

```rust
#[tokio::test]
async fn test_permission_denied() {
    let executor = create_test_executor();
    let result = executor.execute_secure(
        "narrative1",
        "discord",
        "forbidden.command",
        &HashMap::new()
    ).await;
    
    assert!(matches!(
        result.unwrap_err().kind,
        BotCommandErrorKind::PermissionDenied { .. }
    ));
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_full_security_pipeline() {
    let mut executor = create_test_executor();
    
    // 1. Test permission check
    // 2. Test validation
    // 3. Test content filter
    // 4. Test rate limit
    // 5. Test approval workflow
    // 6. Test successful execution
}
```

## Best Practices

### 1. Principle of Least Privilege

**Allowlist commands minimally:**
```toml
# ✅ GOOD: Only necessary commands
allowed_commands = [
    "discord.messages.send",
    "discord.channels.list"
]

# ❌ BAD: Overly permissive
allowed_commands = ["discord.*"]
```

### 2. Resource Restrictions

**Restrict resource access:**
```toml
# ✅ GOOD: Specific channels
[security.permissions.resources.channel]
allowed_ids = ["123456789012345678"]

# ❌ BAD: No restrictions (implicit "allow all")
```

### 3. Protected Users

**Always protect bot owners and admins:**
```toml
[security.permissions.protected_users]
user_ids = [
    "111111111111111111",  # Bot owner
    "222222222222222222"   # Admin
]
```

### 4. Conservative Rate Limits

**Start strict, relax if needed:**
```toml
# ✅ GOOD: Conservative limits
"discord.messages.send" = { limit = 10, window_secs = 60 }

# ❌ BAD: Too permissive
"discord.messages.send" = { limit = 1000, window_secs = 60 }
```

### 5. Approval for Destructive Operations

**Require approval for:**
- Deletions (messages, channels, roles)
- Bans/kicks
- Permission changes
- Bulk operations

```toml
[security.approval]
requires_approval = [
    "discord.channels.delete",
    "discord.members.ban",
    "discord.messages.delete"
]
```

### 6. Comprehensive Logging

**Enable audit logging in production:**
```rust
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .with_target(true)
    .with_thread_ids(true)
    .with_file(true)
    .with_line_number(true)
    .init();
```

### 7. Regular Cleanup

**Clean up expired approvals:**
```rust
// Run periodically (e.g., hourly)
let removed = secure_executor
    .approval_workflow()
    .cleanup_expired();
info!("Cleaned up {} expired actions", removed);
```

## Security Considerations

### Threats Mitigated

1. **AI Hallucination**
   - Symptom: AI invents commands/resources that don't exist
   - Mitigation: Allowlist validation (Layer 1)

2. **Prompt Injection**
   - Symptom: User input tricks AI into executing unintended commands
   - Mitigation: Command allowlist (Layer 1), content filtering (Layer 3)

3. **Privilege Escalation**
   - Symptom: AI attempts to target protected users or high-privilege operations
   - Mitigation: Protected user lists (Layer 1), approval workflow (Layer 5)

4. **Resource Exhaustion**
   - Symptom: AI spams API with excessive requests
   - Mitigation: Rate limiting (Layer 4)

5. **Credential Leakage**
   - Symptom: AI accidentally includes secrets in output
   - Mitigation: Content filtering with prohibited patterns (Layer 3)

6. **Social Engineering**
   - Symptom: AI sends phishing messages or malicious links
   - Mitigation: Content filtering, domain allowlisting (Layer 3)

### Attack Surface

**Entry Points:**
- Narrative TOML files (user-controlled)
- AI-generated content (LLM output)
- Bot command arguments (derived from narrative + AI)

**Trust Boundaries:**
- User → Narrative → AI → Bot Commands → Platform API
- Each boundary needs validation

**Security Perimeter:**
```
Untrusted: [User Input] → [Narrative TOML] → [AI Output]
              ↓
Security Framework (5 layers)
              ↓
Trusted: [Validated Commands] → [Platform API]
```

### Known Limitations

1. **No Persistent Approval Storage**
   - Approvals lost on restart
   - Mitigation: Use database-backed approval workflow (Phase 3)

2. **Static Configuration**
   - Security policies require restart to update
   - Mitigation: Hot-reload configuration (future)

3. **No ML-Based Detection**
   - Content filtering is pattern-based only
   - Mitigation: Integrate toxicity/intent detection (future)

4. **Per-Command Rate Limits**
   - No per-user or per-narrative limits
   - Mitigation: Multi-level rate limiting (future)

## Troubleshooting

### Common Issues

#### 1. All Commands Denied

**Symptom:**
```
PermissionDenied: Command 'discord.messages.send' not in allowlist
```

**Solution:**
- Check `allowed_commands` in security config
- Verify command name format: `platform.command`

#### 2. Resource Access Denied

**Symptom:**
```
ResourceAccessDenied: Resource 'channel:123456...' not in allow list
```

**Solution:**
- Add channel ID to `allowed_ids` for resource type
- Verify snowflake ID format (17-19 digits)

#### 3. Rate Limit Exceeded

**Symptom:**
```
RateLimitExceeded: Rate limit exceeded for 'discord.messages.send'
```

**Solution:**
- Wait for token refill (check `window_secs`)
- Increase `limit` if legitimate use case
- Check for loops in narrative logic

#### 4. Validation Failures

**Symptom:**
```
ValidationFailed: Invalid argument 'channel_id': Invalid Discord channel ID format
```

**Solution:**
- Verify argument format matches validator rules
- Check Discord snowflake format (17-19 digits, integers only)

#### 5. Content Filter Violations

**Symptom:**
```
ContentViolation: Mass mention detected: @everyone
```

**Solution:**
- Remove `@everyone`/`@here` from content
- Adjust content filter config if false positive
- Check prohibited patterns

### Debug Mode

Enable debug logging to trace security decisions:

```bash
RUST_LOG=botticelli_security=debug,botticelli_social=debug cargo run
```

**Output:**
```
DEBUG botticelli_social::secure_executor: Starting secure bot command execution
DEBUG botticelli_security::permission: Checking command permission: discord.messages.send
DEBUG botticelli_security::permission: Command allowed
DEBUG botticelli_security::validation: Validating Discord command parameters
DEBUG botticelli_security::validation: Validation passed
DEBUG botticelli_security::content: Filtering content
DEBUG botticelli_security::content: No violations detected
DEBUG botticelli_security::rate_limit: Checking rate limit for discord.messages.send
DEBUG botticelli_security::rate_limit: Rate limit check passed (9 tokens remaining)
INFO  botticelli_security::executor: All security checks passed
```

## Future Enhancements

### Phase 3 Roadmap

1. **Database-Backed Approvals**
   - PostgreSQL storage for pending actions
   - Query APIs for approval UI
   - Historical approval audit log

2. **Approval Management UI**
   - CLI commands (`approve`, `deny`, `list`)
   - Web dashboard
   - Discord DM notifications

3. **Advanced Content Filtering**
   - ML-based toxicity detection
   - Intent classification
   - Context-aware filtering

4. **Dynamic Rate Limiting**
   - Adaptive limits based on behavior
   - Per-user rate limits
   - Burst detection

5. **Multi-Platform Support**
   - Slack validator
   - Telegram validator
   - Generic validator framework

6. **Policy Editor**
   - Web UI for editing security policies
   - Policy templates
   - Versioning and rollback

## References

- Security Framework Implementation: `crates/botticelli_security/`
- Integration Layer: `crates/botticelli_social/src/secure_executor.rs`
- Architecture Doc: `PHASE_3_SECURITY_FRAMEWORK.md`
- Error Patterns: `CLAUDE.md` (Error Handling section)
- Testing Guide: `TESTING.md`

---

*Last Updated: 2025-11-20*  
*Version: 0.2.0*
