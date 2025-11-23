# Discord Content Actor Implementation Plan

**Goal**: Design and implement an actor that periodically posts new content to a Discord channel using the botticelli_actor system.

**Date**: 2025-11-23  
**Status**: Implementation In Progress

**Progress**:
- ✅ Phase 1: Database schema (content, post_history, actor_preferences)
- ✅ Phase 2: Core skills implemented (5/5)
- 🚧 Phase 3: Actor integration and testing
- ⏳ Phase 4: Deployment configuration

---

## Overview

This actor will:
1. Query a database table for approved/scheduled content
2. Apply scheduling rules (time windows, frequency limits)
3. Select appropriate content based on criteria
4. Post to Discord channel
5. Track posting history to avoid duplicates

---

## Architecture

### Components

```
┌─────────────────────────────────────────────────────────┐
│                    Actor Configuration                   │
│                  (discord_poster.toml)                   │
└────────────────────┬────────────────────────────────────┘
                     │
        ┌────────────┴────────────┐
        │                         │
        ▼                         ▼
┌───────────────┐         ┌──────────────────┐
│   Knowledge   │         │      Skills      │
│    Tables     │         │                  │
├───────────────┤         ├──────────────────┤
│ • content     │────────▶│ • ContentSelector│
│ • post_history│         │ • Scheduler      │
│ • preferences │         │ • RateLimiter    │
└───────────────┘         │ • DuplicateCheck │
                          └────────┬─────────┘
                                   │
                                   ▼
                          ┌────────────────┐
                          │ Discord Platform│
                          │  (botticelli)  │
                          └────────────────┘
```

### Flow Diagram

```
Start
  │
  ├─▶ Load Configuration (discord_poster.toml)
  │
  ├─▶ Query Knowledge Tables
  │     ├─ content: Get approved posts
  │     ├─ post_history: Get recent posts
  │     └─ preferences: Get posting rules
  │
  ├─▶ Execute Skills
  │     ├─ ContentSelector: Filter & rank content
  │     ├─ Scheduler: Check time windows
  │     ├─ RateLimiter: Check frequency limits
  │     └─ DuplicateCheck: Prevent reposts
  │
  ├─▶ Select Best Content
  │     └─ Returns Content or None
  │
  ├─▶ Post to Discord
  │     └─ If content selected
  │
  └─▶ Update post_history
        └─ Record successful post

End
```

---

## Database Schema

### Table: `content`

Stores approved content ready for posting.

```sql
CREATE TABLE content (
    id SERIAL PRIMARY KEY,
    content_type VARCHAR(50) NOT NULL,  -- 'text', 'image', 'video', etc.
    text_content TEXT,
    media_urls TEXT[],                   -- Array of media URLs
    media_types VARCHAR(20)[],           -- Array of MIME types
    source VARCHAR(255),                 -- Where content came from
    priority INTEGER DEFAULT 0,          -- Higher = more important
    tags TEXT[],                         -- Content categorization
    approved_at TIMESTAMP,
    approved_by VARCHAR(100),
    scheduled_for TIMESTAMP,             -- Optional specific time
    expires_at TIMESTAMP,                -- Optional expiration
    post_count INTEGER DEFAULT 0,        -- Times already posted
    last_posted_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW(),
    metadata JSONB                       -- Flexible extra data
);

CREATE INDEX idx_content_approved ON content(approved_at);
CREATE INDEX idx_content_scheduled ON content(scheduled_for);
CREATE INDEX idx_content_priority ON content(priority DESC);
CREATE INDEX idx_content_tags ON content USING GIN(tags);
```

### Table: `post_history`

Tracks all posts made by actors.

```sql
CREATE TABLE post_history (
    id SERIAL PRIMARY KEY,
    content_id INTEGER REFERENCES content(id),
    actor_name VARCHAR(100) NOT NULL,
    platform VARCHAR(50) NOT NULL,       -- 'discord', 'twitter', etc.
    channel_id VARCHAR(100),              -- Platform-specific ID
    post_id VARCHAR(255),                 -- Platform post ID
    posted_at TIMESTAMP DEFAULT NOW(),
    engagement_count INTEGER DEFAULT 0,   -- Likes, reactions, etc.
    metadata JSONB
);

CREATE INDEX idx_post_history_posted ON post_history(posted_at DESC);
CREATE INDEX idx_post_history_content ON post_history(content_id);
CREATE INDEX idx_post_history_actor ON post_history(actor_name, posted_at);
```

### Table: `actor_preferences`

Actor-specific configuration overrides.

```sql
CREATE TABLE actor_preferences (
    id SERIAL PRIMARY KEY,
    actor_name VARCHAR(100) UNIQUE NOT NULL,
    min_post_interval_minutes INTEGER DEFAULT 60,
    max_posts_per_day INTEGER DEFAULT 10,
    preferred_tags TEXT[],               -- Content preferences
    excluded_tags TEXT[],                -- Content to avoid
    time_window_start TIME,              -- e.g., '09:00:00'
    time_window_end TIME,                -- e.g., '17:00:00'
    timezone VARCHAR(50) DEFAULT 'UTC',
    randomize_schedule BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);
```

---

## Actor Configuration

### File: `discord_poster.toml`

```toml
name = "Discord Content Poster"
version = "1.0.0"

[execution]
max_retries = 3
retry_delay_seconds = 60
stop_on_error = false

[platform]
type = "discord"
token_env = "DISCORD_BOT_TOKEN"
channel_id_env = "DISCORD_CHANNEL_ID"

[cache]
strategy = "memory"
ttl_seconds = 300

# Knowledge tables the actor will query
[[knowledge]]
name = "content"
required = true

[[knowledge]]
name = "post_history"
required = true

[[knowledge]]
name = "actor_preferences"
required = false

# Skills the actor will use
[[skills]]
name = "content_selector"
enabled = true
[skills.config]
max_candidates = 10
prefer_scheduled = true
priority_weight = 0.7
freshness_weight = 0.3

[[skills]]
name = "time_window_check"
enabled = true
[skills.config]
default_start = "09:00"
default_end = "21:00"
timezone = "America/Los_Angeles"

[[skills]]
name = "rate_limiter"
enabled = true
[skills.config]
max_posts_per_day = 10
min_interval_minutes = 60

[[skills]]
name = "duplicate_checker"
enabled = true
[skills.config]
lookback_days = 30
similarity_threshold = 0.9

[[skills]]
name = "content_formatter"
enabled = true
[skills.config]
max_text_length = 2000
include_source = true
add_hashtags = true
```

---

## Skill Implementations

### 1. ContentSelectorSkill

**Purpose**: Query and rank content from the database.

**Logic**:
```rust
pub struct ContentSelectorSkill {
    max_candidates: usize,
    priority_weight: f64,
    freshness_weight: f64,
}

impl ContentSelectorSkill {
    async fn execute(&self, context: &SkillContext) -> SkillResult<SkillOutput> {
        // 1. Query content table
        let content = knowledge.query("content")
            .where_clause("approved_at IS NOT NULL")
            .where_clause("(expires_at IS NULL OR expires_at > NOW())")
            .where_clause("(scheduled_for IS NULL OR scheduled_for <= NOW())")
            .order_by("priority DESC, created_at DESC")
            .limit(self.max_candidates)
            .execute()?;
        
        // 2. Score each content item
        let scored = content.map(|item| {
            let priority_score = item.priority as f64 / 10.0;
            let freshness_score = calculate_freshness(item.created_at);
            let score = (priority_score * self.priority_weight) 
                      + (freshness_score * self.freshness_weight);
            (item, score)
        });
        
        // 3. Sort by score and return top candidates
        let candidates = scored.sort_by_score().collect();
        
        Ok(SkillOutput {
            data: json!({ "candidates": candidates }),
            metadata: HashMap::new(),
        })
    }
}
```

**Output**: List of candidate content items with scores.

### 2. TimeWindowCheckSkill

**Purpose**: Verify current time is within allowed posting window.

**Logic**:
```rust
pub struct TimeWindowCheckSkill {
    default_start: NaiveTime,
    default_end: NaiveTime,
    timezone: Tz,
}

impl TimeWindowCheckSkill {
    async fn execute(&self, context: &SkillContext) -> SkillResult<SkillOutput> {
        // 1. Get preferences from database if available
        let prefs = knowledge.query_one("actor_preferences")
            .where_clause("actor_name = $1", &context.actor_name)
            .execute_optional()?;
        
        // 2. Use preferences or defaults
        let (start, end) = match prefs {
            Some(p) => (p.time_window_start, p.time_window_end),
            None => (self.default_start, self.default_end),
        };
        
        // 3. Check current time in timezone
        let now = Utc::now().with_timezone(&self.timezone);
        let current_time = now.time();
        
        let in_window = current_time >= start && current_time <= end;
        
        Ok(SkillOutput {
            data: json!({
                "in_window": in_window,
                "current_time": current_time,
                "window_start": start,
                "window_end": end,
            }),
            metadata: HashMap::new(),
        })
    }
}
```

**Output**: Boolean indicating if current time is in allowed window.

### 3. RateLimiterSkill

**Purpose**: Enforce posting frequency limits.

**Logic**:
```rust
pub struct RateLimiterSkill {
    max_posts_per_day: usize,
    min_interval_minutes: i64,
}

impl RateLimiterSkill {
    async fn execute(&self, context: &SkillContext) -> SkillResult<SkillOutput> {
        // 1. Query recent posts
        let since_today = Utc::now().date().and_hms(0, 0, 0);
        let posts_today = knowledge.query("post_history")
            .where_clause("actor_name = $1", &context.actor_name)
            .where_clause("posted_at >= $1", &since_today)
            .count()?;
        
        // 2. Check daily limit
        if posts_today >= self.max_posts_per_day {
            return Ok(SkillOutput {
                data: json!({
                    "allowed": false,
                    "reason": "daily_limit_reached",
                    "posts_today": posts_today,
                    "max_posts": self.max_posts_per_day,
                }),
                metadata: HashMap::new(),
            });
        }
        
        // 3. Check interval since last post
        let last_post = knowledge.query_one("post_history")
            .where_clause("actor_name = $1", &context.actor_name)
            .order_by("posted_at DESC")
            .limit(1)
            .execute_optional()?;
        
        if let Some(post) = last_post {
            let elapsed = Utc::now() - post.posted_at;
            let min_interval = Duration::minutes(self.min_interval_minutes);
            
            if elapsed < min_interval {
                return Ok(SkillOutput {
                    data: json!({
                        "allowed": false,
                        "reason": "interval_too_short",
                        "elapsed_minutes": elapsed.num_minutes(),
                        "required_minutes": self.min_interval_minutes,
                    }),
                    metadata: HashMap::new(),
                });
            }
        }
        
        // 4. Rate limit checks passed
        Ok(SkillOutput {
            data: json!({
                "allowed": true,
                "posts_today": posts_today,
                "remaining_today": self.max_posts_per_day - posts_today,
            }),
            metadata: HashMap::new(),
        })
    }
}
```

**Output**: Boolean + metadata about rate limit status.

### 4. DuplicateCheckerSkill

**Purpose**: Prevent posting duplicate or very similar content.

**Logic**:
```rust
pub struct DuplicateCheckerSkill {
    lookback_days: i64,
    similarity_threshold: f64,
}

impl DuplicateCheckerSkill {
    async fn execute(&self, context: &SkillContext) -> SkillResult<SkillOutput> {
        // 1. Get candidate content
        let candidates = context.get_data::<Vec<ContentItem>>("candidates")?;
        
        // 2. Query recent posts
        let since = Utc::now() - Duration::days(self.lookback_days);
        let recent_posts = knowledge.query("post_history")
            .where_clause("actor_name = $1", &context.actor_name)
            .where_clause("posted_at >= $1", &since)
            .join("content", "content_id")
            .execute()?;
        
        // 3. Check each candidate for duplicates
        let mut filtered = Vec::new();
        for candidate in candidates {
            let is_duplicate = recent_posts.iter().any(|posted| {
                self.is_similar(&candidate, &posted.content)
            });
            
            if !is_duplicate {
                filtered.push(candidate);
            }
        }
        
        Ok(SkillOutput {
            data: json!({
                "filtered_candidates": filtered,
                "removed_count": candidates.len() - filtered.len(),
            }),
            metadata: HashMap::new(),
        })
    }
    
    fn is_similar(&self, a: &ContentItem, b: &ContentItem) -> bool {
        // Simple approach: exact text match
        if a.text_content == b.text_content {
            return true;
        }
        
        // Advanced: compute similarity score (Levenshtein, cosine, etc.)
        let similarity = compute_text_similarity(&a.text_content, &b.text_content);
        similarity > self.similarity_threshold
    }
}
```

**Output**: Filtered list of non-duplicate content.

### 5. ContentFormatterSkill

**Purpose**: Format content for Discord's requirements.

**Logic**:
```rust
pub struct ContentFormatterSkill {
    max_text_length: usize,
    include_source: bool,
    add_hashtags: bool,
}

impl ContentFormatterSkill {
    async fn execute(&self, context: &SkillContext) -> SkillResult<SkillOutput> {
        // 1. Get selected content
        let content = context.get_data::<ContentItem>("selected_content")?;
        
        // 2. Format text
        let mut text = content.text_content.clone();
        
        // Truncate if too long
        if text.len() > self.max_text_length {
            text.truncate(self.max_text_length - 3);
            text.push_str("...");
        }
        
        // Add source attribution
        if self.include_source && content.source.is_some() {
            text.push_str(&format!("\n\n📎 Source: {}", content.source));
        }
        
        // Add hashtags
        if self.add_hashtags && !content.tags.is_empty() {
            let tags = content.tags.iter()
                .map(|t| format!("#{}", t))
                .collect::<Vec<_>>()
                .join(" ");
            text.push_str(&format!("\n\n{}", tags));
        }
        
        // 3. Build Content object
        let formatted = ContentBuilder::default()
            .text(text)
            .media(content.media_urls.iter().zip(&content.media_types)
                .map(|(url, mime)| MediaAttachment {
                    url: url.clone(),
                    media_type: MediaType::from_mime(mime),
                })
                .collect())
            .build()?;
        
        Ok(SkillOutput {
            data: json!({ "formatted_content": formatted }),
            metadata: HashMap::new(),
        })
    }
}
```

**Output**: Formatted Content ready for posting.

---

## Actor Execution Flow

### Main Loop (Cron/Systemd Timer)

```bash
# Run every 30 minutes
*/30 * * * * /usr/local/bin/botticelli-actor discord_poster.toml
```

### Execution Steps

```rust
async fn main() -> Result<()> {
    // 1. Load configuration
    let config = ActorConfig::from_file("discord_poster.toml")?;
    
    // 2. Create platform
    let token = std::env::var("DISCORD_BOT_TOKEN")?;
    let channel_id = std::env::var("DISCORD_CHANNEL_ID")?;
    let platform = DiscordPlatform::new(token, channel_id)?;
    
    // 3. Register skills
    let mut registry = SkillRegistry::new();
    registry.register(Arc::new(ContentSelectorSkill::new()));
    registry.register(Arc::new(TimeWindowCheckSkill::new()));
    registry.register(Arc::new(RateLimiterSkill::new()));
    registry.register(Arc::new(DuplicateCheckerSkill::new()));
    registry.register(Arc::new(ContentFormatterSkill::new()));
    
    // 4. Build actor
    let actor = Actor::builder()
        .config(config)
        .skills(registry)
        .platform(Arc::new(platform))
        .build()?;
    
    // 5. Connect to database
    let mut conn = establish_connection()?;
    
    // 6. Execute actor workflow
    let result = actor.execute(&mut conn).await?;
    
    // 7. Handle result
    match result.success {
        true => {
            info!("Actor execution succeeded");
            info!("Skills executed: {}", result.executed.len());
            
            // Update post_history if content was posted
            if let Some(post_id) = result.metadata.get("post_id") {
                record_post(&mut conn, post_id)?;
            }
        }
        false => {
            error!("Actor execution failed: {:?}", result.errors);
        }
    }
    
    Ok(())
}
```

---

## Deployment Options

### Option 1: Systemd Timer

**File**: `/etc/systemd/system/discord-poster.timer`

```ini
[Unit]
Description=Discord Content Poster Timer
Requires=discord-poster.service

[Timer]
OnBootSec=5min
OnUnitActiveSec=30min
AccuracySec=1min

[Install]
WantedBy=timers.target
```

**File**: `/etc/systemd/system/discord-poster.service`

```ini
[Unit]
Description=Discord Content Poster
After=postgresql.service

[Service]
Type=oneshot
User=botticelli
WorkingDirectory=/opt/botticelli
EnvironmentFile=/etc/botticelli/discord-poster.env
ExecStart=/usr/local/bin/botticelli-actor /etc/botticelli/discord_poster.toml
StandardOutput=journal
StandardError=journal
```

### Option 2: Kubernetes CronJob

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: discord-poster
spec:
  schedule: "*/30 * * * *"
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: actor
            image: botticelli/discord-poster:latest
            env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: botticelli-secrets
                  key: database-url
            - name: DISCORD_BOT_TOKEN
              valueFrom:
                secretKeyRef:
                  name: botticelli-secrets
                  key: discord-token
            - name: DISCORD_CHANNEL_ID
              value: "123456789012345678"
          restartPolicy: OnFailure
```

### Option 3: AWS Lambda (Scheduled)

```rust
use lambda_runtime::{handler_fn, Context, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = handler_fn(actor_handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn actor_handler(_event: Value, _ctx: Context) -> Result<Value, Error> {
    // Same execution flow as above
    let result = run_actor().await?;
    Ok(json!({ "success": result.success }))
}
```

---

## Monitoring & Observability

### Metrics to Track

1. **Execution metrics**:
   - Execution duration
   - Success/failure rate
   - Skills executed per run
   - Retry attempts

2. **Content metrics**:
   - Posts per day
   - Content backlog size
   - Average time from approval to post
   - Duplicate detection rate

3. **Rate limit metrics**:
   - Current posts per day
   - Time until next allowed post
   - Rate limit violations

4. **Platform metrics**:
   - Discord API response times
   - API errors/rate limits
   - Engagement (reactions, replies)

### Logging Strategy

```rust
// In actor execution
#[instrument(skip(conn), fields(actor_name = %config.name()))]
async fn execute(&self, conn: &mut PgConnection) -> ActorResult<ExecutionResult> {
    info!("Starting actor execution");
    
    // Log each skill
    for skill in &self.skills {
        debug!(skill = %skill.name(), "Executing skill");
        let output = skill.execute(&context).await?;
        info!(skill = %skill.name(), success = true, "Skill completed");
    }
    
    // Log final outcome
    if let Some(content_id) = result.metadata.get("content_id") {
        info!(content_id = %content_id, "Posted content to Discord");
    } else {
        info!("No content selected for posting");
    }
    
    Ok(result)
}
```

### Alerting

Set up alerts for:
- ❌ Failed executions (3+ in a row)
- ⚠️ No posts in 24+ hours (content drought)
- ⚠️ Rate limit violations
- ❌ Database connection failures
- ⚠️ Low content backlog (< 10 items)

---

## Testing Strategy

### Unit Tests

Test each skill individually:

```rust
#[tokio::test]
async fn test_content_selector_filters_expired() {
    let skill = ContentSelectorSkill::new(10, 0.7, 0.3);
    let context = mock_context_with_expired_content();
    
    let output = skill.execute(&context).await.unwrap();
    let candidates = output.get_data::<Vec<ContentItem>>("candidates").unwrap();
    
    assert!(candidates.iter().all(|c| !c.is_expired()));
}

#[tokio::test]
async fn test_rate_limiter_blocks_when_limit_reached() {
    let skill = RateLimiterSkill::new(10, 60);
    let context = mock_context_with_10_posts_today();
    
    let output = skill.execute(&context).await.unwrap();
    let allowed = output.get_bool("allowed").unwrap();
    
    assert!(!allowed);
}
```

### Integration Tests

Test full actor workflow:

```rust
#[tokio::test]
async fn test_actor_posts_content_successfully() {
    let (config, registry, platform) = setup_test_actor();
    let mut conn = test_database_connection();
    
    // Insert test content
    insert_test_content(&mut conn, "Test post", vec!["test"]);
    
    // Execute actor
    let actor = Actor::builder()
        .config(config)
        .skills(registry)
        .platform(platform)
        .build()
        .unwrap();
    
    let result = actor.execute(&mut conn).await.unwrap();
    
    assert!(result.success);
    assert_eq!(result.executed.len(), 5); // All skills ran
    
    // Verify post was recorded
    let posts = query_post_history(&mut conn);
    assert_eq!(posts.len(), 1);
}
```

### Manual Testing

```bash
# Test with dry-run mode (don't actually post)
BOTTICELLI_DRY_RUN=true cargo run --bin botticelli-actor discord_poster.toml

# Test with specific content ID
BOTTICELLI_FORCE_CONTENT_ID=123 cargo run --bin botticelli-actor discord_poster.toml

# Test time window logic
BOTTICELLI_OVERRIDE_TIME="2025-11-23T15:00:00Z" cargo run --bin botticelli-actor discord_poster.toml
```

---

## Future Enhancements

### Phase 2: Multi-Channel Support

Support posting to multiple channels:

```toml
[[platform]]
type = "discord"
channel_id = "123456789"
tags = ["general", "announcements"]

[[platform]]
type = "discord"
channel_id = "987654321"
tags = ["memes", "fun"]
```

### Phase 3: Content Generation

Add skill to generate content from narratives:

```rust
pub struct ContentGeneratorSkill {
    narrative_name: String,
    prompt_template: String,
}

// Executes narrative to generate new content
// Stores in content table for approval workflow
```

### Phase 4: Engagement Tracking

Query Discord API for engagement metrics:

```rust
pub struct EngagementTrackerSkill;

// Updates post_history with reaction counts
// Adjusts content selection based on engagement
```

### Phase 5: A/B Testing

Test different content strategies:

```rust
pub struct ABTestingSkill {
    experiments: HashMap<String, Experiment>,
}

// Randomly assigns content to test groups
// Tracks performance metrics
// Reports results
```

### Phase 6: Smart Scheduling

ML-based optimal posting times:

```rust
pub struct SmartSchedulerSkill {
    model: EngagementPredictionModel,
}

// Analyzes historical engagement patterns
// Predicts best posting times
// Adjusts schedule dynamically
```

---

## Implementation Checklist

### Database Setup
- [ ] Create `content` table
- [ ] Create `post_history` table
- [ ] Create `actor_preferences` table
- [ ] Add indexes for performance
- [ ] Create test fixtures

### Skills Implementation
- [ ] ContentSelectorSkill
- [ ] TimeWindowCheckSkill
- [ ] RateLimiterSkill
- [ ] DuplicateCheckerSkill
- [ ] ContentFormatterSkill
- [ ] Unit tests for each skill

### Actor Configuration
- [ ] Write `discord_poster.toml`
- [ ] Document configuration options
- [ ] Add validation

### Integration
- [ ] Create binary `botticelli-actor`
- [ ] Integration tests
- [ ] End-to-end test with test Discord server

### Deployment
- [ ] Create systemd service/timer
- [ ] Docker container
- [ ] Environment variable documentation
- [ ] Deployment guide

### Monitoring
- [ ] Add structured logging
- [ ] Export metrics
- [ ] Set up dashboards
- [ ] Configure alerts

### Documentation
- [ ] User guide
- [ ] API reference
- [ ] Troubleshooting guide
- [ ] Example configurations

---

## Timeline Estimate

- **Week 1**: Database schema + skills implementation
- **Week 2**: Actor integration + testing
- **Week 3**: Deployment setup + monitoring
- **Week 4**: Documentation + polish

**Total**: 4 weeks for MVP

---

## Success Criteria

✅ Actor runs reliably on schedule  
✅ Posts content without duplicates  
✅ Respects time windows and rate limits  
✅ Handles errors gracefully with retries  
✅ Full observability (logs, metrics, traces)  
✅ Comprehensive test coverage (>80%)  
✅ Production-ready deployment scripts  
✅ Complete documentation  

---

## References

- [Actor Architecture](./ACTOR_ARCHITECTURE.md)
- [Actor User Guide](./crates/botticelli_actor/ACTOR_GUIDE.md)
- [Discord API Documentation](https://discord.com/developers/docs)
- [Botticelli Narrative System](./NARRATIVE_TOML_SPEC.md)
