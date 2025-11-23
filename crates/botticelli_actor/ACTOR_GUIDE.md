# Actor System User Guide

The actor system provides a platform-agnostic framework for automating social media content posting with configurable skills and knowledge integration.

## Quick Start

### 1. Create Actor Configuration

Create an `actor.toml` file:

```toml
[actor]
name = "My Content Bot"
description = "Automatically posts content from knowledge tables"
knowledge = ["approved_posts", "scheduled_content"]
skills = ["content_scheduling", "rate_limiting"]

[actor.config]
max_posts_per_day = 10
min_interval_minutes = 60
timezone = "America/New_York"

[actor.cache]
strategy = "memory"
ttl_seconds = 300

[actor.execution]
stop_on_unrecoverable = true
max_retries = 3
continue_on_error = true

[skills.content_scheduling]
enabled = true
schedule_window_start = "09:00"
schedule_window_end = "17:00"
randomize_within_window = true

[skills.rate_limiting]
enabled = true
max_posts_per_day = 10
min_interval_minutes = 60
```

### 2. Write Your Actor

```rust
use botticelli_actor::{
    Actor, ActorConfigBuilder, ContentSchedulingSkill, DiscordPlatform,
    ExecutionConfigBuilder, RateLimitingSkill, Skill, SkillRegistry,
};
use botticelli_actor::skills::{
    ContentFormatterSkill, ContentSelectionSkill, DuplicateCheckSkill,
};
use botticelli_database::establish_connection;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration
    let execution_config = ExecutionConfigBuilder::default()
        .continue_on_error(true)
        .stop_on_unrecoverable(true)
        .max_retries(3)
        .build()?;

    let config = ActorConfigBuilder::default()
        .name("my_actor".to_string())
        .description("My content poster".to_string())
        .knowledge(vec!["content".to_string(), "post_history".to_string()])
        .skills(vec!["content_selection".to_string()])
        .execution(execution_config)
        .build()?;

    // Register skills
    let mut registry = SkillRegistry::new();
    registry.register(Arc::new(ContentSelectionSkill::default()) as Arc<dyn Skill>);
    registry.register(Arc::new(ContentSchedulingSkill::default()) as Arc<dyn Skill>);
    registry.register(Arc::new(RateLimitingSkill::default()) as Arc<dyn Skill>);
    registry.register(Arc::new(DuplicateCheckSkill::default()) as Arc<dyn Skill>);
    registry.register(Arc::new(ContentFormatterSkill::default()) as Arc<dyn Skill>);

    // Create platform
    let token = std::env::var("DISCORD_TOKEN")?;
    let channel_id: u64 = std::env::var("DISCORD_CHANNEL_ID")?.parse()?;
    let platform = Arc::new(DiscordPlatform::new(token, channel_id.to_string())?);

    // Build actor
    let actor = Actor::builder()
        .config(config)
        .skills(registry)
        .platform(platform)
        .build()?;

    // Execute with database connection
    let mut conn = establish_connection()?;
    let result = actor.execute(&mut conn).await?;
    
    println!("Execution complete!");
    println!("  Succeeded: {}", result.succeeded.len());
    println!("  Failed: {}", result.failed.len());
    println!("  Skipped: {}", result.skipped.len());

    Ok(())
}
```

### 3. Run Your Actor

```bash
export DISCORD_TOKEN="your_bot_token"
export DISCORD_CHANNEL_ID="1234567890"
export DATABASE_URL="postgresql://user:pass@localhost/db"

# Run the example
just run-example botticelli_actor discord_poster

# Or build and run manually
cargo run --example discord_poster --features discord
```

See `crates/botticelli_actor/examples/discord_poster.rs` for a complete working example.

## Core Concepts

### Actors

An **Actor** orchestrates the execution of skills using knowledge from database tables. Actors:
- Load knowledge from configured tables
- Execute skills in order
- Handle errors according to configuration
- Return execution results

### Skills

**Skills** are reusable capabilities that process knowledge and interact with platforms. Built-in skills:

- **ContentSchedulingSkill** - Calculates optimal posting times within configured windows
- **RateLimitingSkill** - Enforces posting frequency limits

Custom skills implement the `Skill` trait:

```rust
use botticelli_actor::{Skill, SkillContext, SkillOutput, SkillResult};
use async_trait::async_trait;

pub struct MyCustomSkill;

#[async_trait]
impl Skill for MyCustomSkill {
    fn name(&self) -> &str {
        "my_custom_skill"
    }

    fn description(&self) -> &str {
        "Does something custom"
    }

    async fn execute(&self, context: &SkillContext) -> SkillResult<SkillOutput> {
        // Access knowledge: context.knowledge
        // Access config: context.config
        // Access platform: context.platform

        Ok(SkillOutput {
            skill_name: self.name().to_string(),
            data: serde_json::json!({"status": "success"}),
        })
    }
}
```

### Platforms

**Platforms** implement the `SocialMediaPlatform` trait to integrate with specific services:

- **DiscordPlatform** - Posts to Discord channels
- Create your own by implementing `SocialMediaPlatform`

### Knowledge

**Knowledge** is structured data stored in database tables that actors consume. Knowledge tables:
- Are produced by narratives or other systems
- Contain JSON-serializable data
- Are queried using `KnowledgeTable`

## Configuration

### Actor Settings

| Setting | Description | Default |
|---------|-------------|---------|
| `max_posts_per_day` | Maximum posts allowed per day | 10 |
| `min_interval_minutes` | Minimum minutes between posts | 60 |
| `retry_attempts` | Number of retry attempts | 3 |
| `timezone` | IANA timezone name | "UTC" |

### Cache Settings

| Setting | Description | Default |
|---------|-------------|---------|
| `strategy` | Cache strategy: "none", "memory", "disk" | "memory" |
| `ttl_seconds` | Time-to-live for cache entries | 300 |
| `max_entries` | Maximum cache entries | 1000 |
| `disk_path` | Path for disk cache (disk only) | None |

### Execution Settings

| Setting | Description | Default |
|---------|-------------|---------|
| `stop_on_unrecoverable` | Stop execution on unrecoverable errors | true |
| `max_retries` | Maximum retry attempts for recoverable errors | 3 |
| `continue_on_error` | Continue execution after errors | true |

## Error Handling

The actor system distinguishes between **recoverable** and **unrecoverable** errors:

### Recoverable Errors
- Temporary platform issues
- Rate limit exceeded
- Network timeouts

These can be retried according to `max_retries` configuration.

### Unrecoverable Errors
- Authentication failures
- Invalid configuration
- Missing knowledge tables
- Validation failures

These stop execution if `stop_on_unrecoverable` is true.

## Execution Results

`ExecutionResult` contains:
- `succeeded` - Successfully executed skills with outputs
- `failed` - Failed skills with error details
- `skipped` - Disabled or skipped skills

```rust
let result = actor.execute(&mut conn).await?;

println!("Succeeded: {}", result.succeeded.len());
println!("Failed: {}", result.failed.len());
println!("Skipped: {}", result.skipped.len());

for output in result.succeeded {
    println!("Skill: {}", output.skill_name);
    println!("Data: {}", output.data);
}

for (skill, error) in result.failed {
    println!("Skill {} failed: {}", skill, error);
}
```

## Best Practices

### 1. Use Configuration Files

Store actor configuration in TOML files for easy maintenance and version control.

### 2. Validate Configuration

Always validate configuration before execution:

```rust
let config = ActorConfig::from_file("actor.toml")?;
let warnings = config.validate();
for warning in warnings {
    eprintln!("Warning: {}", warning);
}
```

### 3. Handle Execution Results

Check execution results and handle failures appropriately:

```rust
let result = actor.execute(&mut conn).await?;

if !result.failed.is_empty() {
    eprintln!("Some skills failed:");
    for (skill, error) in result.failed {
        eprintln!("  - {}: {}", skill, error);
    }
}
```

### 4. Use Tracing

Initialize tracing for observability:

```rust
tracing_subscriber::fmt()
    .with_target(false)
    .with_level(true)
    .init();
```

### 5. Secure Credentials

Never hardcode tokens or credentials. Use environment variables or secure vaults:

```rust
let token = std::env::var("DISCORD_BOT_TOKEN")?;
let channel = std::env::var("DISCORD_CHANNEL_ID")?;
```

## Troubleshooting

### "Knowledge table not found"

- Ensure tables exist in database
- Check table names in configuration
- Verify database connection

### "Rate limit exceeded"

- Increase `min_interval_minutes`
- Decrease `max_posts_per_day`
- Check platform rate limits

### "Authentication failed"

- Verify bot token is correct
- Check token has required permissions
- Ensure token isn't expired

### "Validation failed"

- Check content length limits
- Verify media attachment counts
- Ensure content has text or media

## Examples

See `examples/post_scheduler.rs` for a complete working example.

## Further Reading

- [Actor Architecture Document](../../ACTOR_ARCHITECTURE.md)
- [Platform Trait Documentation](src/platform.rs)
- [Skill Trait Documentation](src/skill.rs)
- [Configuration Types](src/config.rs)
