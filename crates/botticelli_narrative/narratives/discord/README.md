# Botticelli Discord Community Narratives

This directory contains narrative TOML files for setting up and managing the Botticelli Discord community server.

## Setup

These narratives demonstrate Botticelli's capabilities while building a functional community space.

### Environment Variables

Set these in your `.env` file:

```bash
GUILD_ID=your_discord_guild_id
DISCORD_TOKEN=your_bot_token
```

### Running the Narratives

From the workspace root:

```bash
# Initial server setup
cargo run --bin botticelli -- execute crates/botticelli_narratives/narratives/discord/server_setup.toml

# Create documentation channels
cargo run --bin botticelli -- execute crates/botticelli_narratives/narratives/discord/docs_channels.toml
```

## Narrative Files

- `server_setup.toml` - Initial server configuration and welcome message
- `docs_channels.toml` - Create documentation and tutorial channels
- `community_channels.toml` - Create discussion and support channels

## Testing

See `crates/botticelli_narratives/tests/discord_narrative_test.rs` for integration tests.
