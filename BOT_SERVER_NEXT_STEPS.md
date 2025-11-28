# Bot Server Next Steps

**Status**: Implementation Roadmap  
**Created**: 2025-11-28  
**Updated**: 2025-11-28  

---

## Current Situation

The `botticelli_bot` crate exists with driver-generic implementations but is **disabled in the workspace** due to missing functionality:

```toml
# Cargo.toml line 17:
# "crates/botticelli_bot",  # TODO: Re-enable after implementing execute_narrative_by_name
```

### What Exists

- ✅ `botticelli_interface::bot_server` - Driver-generic traits (`BotActor<D>`, `BotServer<D>`)
- ✅ `botticelli_bot/src/` - Concrete bot implementations (generation, curation, posting)
- ✅ All bots are generic over `D: BotticelliDriver`
- ✅ Narratives tested: generation_carousel, curate_and_approve, discord_poster

### What's Blocking

The bot implementations call `executor.execute_narrative_by_name()` which **doesn't exist**:

```rust
// From botticelli_bot/src/generation.rs:
let result = self
    .executor
    .execute_narrative_by_name(&self.config.narrative_name)  // ❌ Method missing
    .await?;
```

---

## Implementation Phases

### Phase 1: Add `execute_narrative_by_name` to NarrativeExecutor ⭐ **START HERE**

**Goal**: Allow executing a named narrative from a multi-narrative TOML file

**Why**: Bots need to load a TOML once, then execute specific narratives by name repeatedly

**Changes in `botticelli_narrative/src/executor.rs`**:

1. Store loaded narratives:
```rust
pub struct NarrativeExecutor<D: BotticelliDriver> {
    driver: Arc<D>,
    database: Arc<DatabaseRepository>,
    narratives: HashMap<String, Narrative>,  // New
}
```

2. Add loading method:
```rust
/// Load all narratives from a multi-narrative TOML file
pub async fn load_narratives(&mut self, path: impl AsRef<Path>) -> NarrativeResult<Vec<String>> {
    let content = fs::read_to_string(path)?;
    let parsed = toml::from_str::<TomlNarratives>(&content)?;
    
    for (name, toml_narrative) in parsed.narratives {
        let narrative = convert_toml_narrative(toml_narrative, &parsed.acts)?;
        self.narratives.insert(name, narrative);
    }
    
    Ok(self.narratives.keys().cloned().collect())
}
```

3. Add execution method:
```rust
/// Execute a previously loaded narrative by name
pub async fn execute_narrative_by_name(&self, name: &str) -> NarrativeResult<NarrativeOutput> {
    let narrative = self.narratives
        .get(name)
        .ok_or_else(|| NarrativeError::not_found(name))?;
    
    self.execute_narrative_internal(narrative).await
}
```

**Testing**:
```bash
# Manual test - should work identically to current behavior
just narrate generation_carousel.batch_generate
```

### Phase 2: Re-enable botticelli_bot in Workspace

**Changes**:

1. Uncomment in `Cargo.toml`:
```toml
members = [
    # ...
    "crates/botticelli_bot",
]
```

2. Verify compilation:
```bash
just check -p botticelli_bot
```

### Phase 3: Implement Bot Server Orchestration

**Create `botticelli_server/src/discord_bot_server.rs`**:

```rust
use botticelli_bot::{GenerationBot, CurationBot, PostingBot};
use botticelli_interface::{BotServer, BotticelliDriver};
use tokio::sync::mpsc;

pub struct DiscordBotServer<D: BotticelliDriver> {
    config: BotServerConfig,
    generation_tx: mpsc::Sender<GenerationMessage>,
    curation_tx: mpsc::Sender<CurationMessage>,
    posting_tx: mpsc::Sender<PostingMessage>,
}

impl<D: BotticelliDriver + Clone + 'static> BotServer<D> for DiscordBotServer<D> {
    async fn start(&mut self, driver: D) -> BotResult<()> {
        // Load narratives once
        let executor = NarrativeExecutor::new(driver, database);
        executor.load_narratives("generation_carousel.toml").await?;
        
        // Spawn bots with shared executor
        let (gen_tx, gen_rx) = mpsc::channel(10);
        let gen_bot = GenerationBot::new(config, Arc::new(executor), gen_rx);
        tokio::spawn(gen_bot.run());
        
        // Similar for curation and posting...
        
        Ok(())
    }
}
```

### Phase 4: Configuration

**Create `actor_server.toml`**:

```toml
[generation]
narrative_file = "crates/botticelli_narrative/narratives/discord/generation_carousel.toml"
narrative_name = "batch_generate"
schedule_hours = 24

[curation]
narrative_file = "crates/botticelli_narrative/narratives/discord/curate_and_approve.toml"
narrative_name = "select_and_approve"
schedule_hours = 12
drain_queue = true

[posting]
narrative_file = "crates/botticelli_narrative/narratives/discord/discord_poster.toml"
narrative_name = "post_next"
base_interval_seconds = 7200  # 2 hours
jitter_seconds = 1800  # ±30 min
```

### Phase 5: CLI Integration

**Update justfile**:

```just
# Start bot server with all actors
bot-server config="actor_server.toml":
    cargo run --release --features local -- bot-server --config {{config}}
```

---

## Success Criteria

- [ ] `execute_narrative_by_name` implemented
- [ ] `botticelli_bot` compiles
- [ ] `just bot-server` starts all bots
- [ ] Bots run on schedule
- [ ] Structured logging works
- [ ] 48-hour stability test passes

---

## Timeline

- Phase 1: 2-3 hours
- Phase 2: 30 minutes  
- Phase 3: 2-3 hours
- Phase 4: 1 hour
- Phase 5: 1 hour
- Testing: 3-4 hours

**Total**: ~10-12 hours
