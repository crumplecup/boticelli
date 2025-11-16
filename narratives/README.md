# Example Narratives

This directory contains example narratives demonstrating how to use Boticelli's narrative system and processor pipeline.

## Available Narratives

### discord_infrastructure.toml

Creates basic Discord infrastructure in the database:

- **Guild (Server)**: Creates a demo Discord server
- **Bot User**: Creates a bot user account
- **Guild Member**: Adds the bot to the server
- **Channel**: Creates a text channel for content

**Run with:**

```bash
boticelli run --narrative narratives/discord_infrastructure.toml --process-discord
```

**What it demonstrates:**

- Using narrative preambles from DISCORD_NARRATIVE.md
- JSON generation following Discord schema
- Automatic processor pipeline with `--process-discord` flag
- Database insertion via Discord processors

**After running**, you can query the database to see the created entities:

```sql
-- View the created guild
SELECT * FROM discord_guilds WHERE id = 1100000000000000001;

-- View the bot user
SELECT * FROM discord_users WHERE id = 1100000000000000200;

-- View the guild membership
SELECT * FROM discord_guild_members
WHERE guild_id = 1100000000000000001
  AND user_id = 1100000000000000200;

-- View the channel
SELECT * FROM discord_channels WHERE id = 1100000000000000300;
```

### discord_content_examples.toml

Generates creative content examples for Discord bot posts:

- Daily motivational messages
- Technology tips and tutorials
- Creative writing prompts
- Community discussion questions
- Interesting facts

**Run with:**

```bash
boticelli run --narrative narratives/discord_content_examples.toml
```

**What it demonstrates:**

- Plain text generation (not JSON)
- Multiple themed acts in a single narrative
- Content variety for different use cases
- Creative prompt engineering

**Note:** This narrative generates content ideas but does NOT create Discord messages in the database. Discord messages are sent via the Discord API, not stored locally.

### test_minimal.toml

Minimal narrative for API testing and quota conservation.

## Narrative Format

All narratives use TOML format with this structure:

```toml
[narration]
name = "Narrative Name"
description = "What this narrative does"

[toc]
order = ["act1", "act2", "act3"]

[acts]
act1 = "Prompt for first act"
act2 = "Prompt for second act"
act3 = "Prompt for third act"
```

## Using the Processor Pipeline

To enable automatic Discord data processing:

1. **Add the flag**: Use `--process-discord` when running the narrative
2. **Use proper JSON**: Follow the schema from DISCORD_NARRATIVE.md
3. **Check logs**: Use `RUST_LOG=boticelli=info` to see processor activity

**Example:**

```bash
RUST_LOG=boticelli=info boticelli run \
  --narrative narratives/discord_infrastructure.toml \
  --process-discord
```

You'll see logs showing:

- ✓ Registered 6 Discord processors
- Processing act with registered processors
- Processing Discord guilds/users/channels/etc.
- Successfully stored entities

## Creating Your Own Narratives

See [DISCORD_NARRATIVE.md](../DISCORD_NARRATIVE.md) for:

- Complete database schemas
- Required and optional fields
- Example JSON outputs
- Narrative preambles

See [NARRATIVE_PROCESSORS.md](../NARRATIVE_PROCESSORS.md) for:

- How processors work
- Extending with new processors
- Error handling
- Testing strategies

## Tips

1. **Test incrementally**: Start with one act, verify it works, then add more
2. **Check the database**: Query after running to verify data was inserted
3. **Use logging**: `RUST_LOG=boticelli=trace` shows detailed processor activity
4. **Handle errors**: Processor errors are logged but don't fail the narrative
5. **Unique IDs**: Use different Discord snowflake IDs for each entity to avoid conflicts

## Troubleshooting

**Processor doesn't run:**

- Check that you used `--process-discord` flag
- Verify JSON is valid (no markdown code blocks)
- Check `should_process()` logic matches your act name or JSON content

**Database errors:**

- Ensure foreign key relationships are correct (e.g., guild_id references existing guild)
- Check for unique constraint violations (duplicate IDs)
- Verify required fields are present

**JSON parsing errors:**

- Remove markdown code blocks (no ```json)
- Start with `{` and end with `}`
- Use `null` for optional fields, not missing keys
- Check for trailing commas
