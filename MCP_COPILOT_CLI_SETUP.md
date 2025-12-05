# Using Botticelli MCP with GitHub Copilot CLI

**How to use Botticelli MCP server with the GitHub Copilot CLI you're using right now!**

## Overview

GitHub Copilot CLI supports MCP (Model Context Protocol) servers through `.vscode/mcp.json` configuration. This allows you to use natural language commands that interact with your database through Botticelli's MCP tools.

## Quick Setup

### 1. Configuration Already Created! ✅

The configuration file is at `.vscode/mcp.json`:

```json
{
  "mcpServers": {
    "botticelli": {
      "command": "/home/erik/repos/botticelli/target/release/botticelli-mcp",
      "args": [],
      "env": {
        "DATABASE_URL": "postgres://boticelli:renaissance@localhost:5432/boticelli",
        "RUST_LOG": "info"
      }
    }
  }
}
```

### 2. Binary Already Built! ✅

```bash
ls -lh target/release/botticelli-mcp
```

The release binary is optimized and ready.

### 3. Test the Server

```bash
# Quick test (should see "Router initialized tools=3")
timeout 1 ./target/release/botticelli-mcp 2>&1 | grep "Router initialized"
```

## How to Use with Copilot CLI

### Method 1: Direct Questions (Recommended)

Just ask natural language questions in your current Copilot session:

```
Query the content table and show me the latest 5 entries
```

```
What tables are available in the database?
```

```
Show me all discord guilds
```

Copilot will automatically detect and use the MCP tools!

### Method 2: Explicit Tool Calls

You can also explicitly mention tools:

```
Use the query_content tool to get data from the content table
```

### Method 3: Agent Mode (if available)

```bash
gh copilot agent
> query_content table=content limit=5
```

## Available MCP Tools

Once connected, you have access to:

### 1. `query_content`
Query any database table with optional limit.

**Example:**
```
Query the content table with a limit of 10
```

**Behind the scenes:**
```json
{
  "tool": "query_content",
  "params": {
    "table": "content",
    "limit": 10
  }
}
```

### 2. `get_server_info`
Get information about the MCP server and available tools.

**Example:**
```
What tools are available in the Botticelli MCP server?
```

### 3. `echo`
Test connectivity (useful for debugging).

**Example:**
```
Echo "test message" through the MCP server
```

## Testing the Integration

### Step 1: Verify MCP Config

```bash
cat .vscode/mcp.json
```

Should show the Botticelli configuration.

### Step 2: Check Binary

```bash
./target/release/botticelli-mcp --version 2>&1 | head -5
```

### Step 3: Test Database Connection

```bash
# The server uses DATABASE_URL from .env
grep DATABASE_URL .env
```

### Step 4: Try a Query

In this Copilot session, just ask:

```
Can you query the content table using the MCP server and show me what's in there?
```

## Troubleshooting

### "MCP server not found"

**Check the binary path:**
```bash
ls -l target/release/botticelli-mcp
```

**Make it executable:**
```bash
chmod +x target/release/botticelli-mcp
```

### "Database connection failed"

**Verify PostgreSQL is running:**
```bash
pg_isready -h localhost -p 5432
```

**Check DATABASE_URL:**
```bash
echo $DATABASE_URL
# OR
grep DATABASE_URL .env
```

### "No tools available"

**Check server logs:**
```bash
RUST_LOG=debug ./target/release/botticelli-mcp 2>&1 | head -20
```

Should see: `Router initialized tools=3`

### "Permission denied"

```bash
chmod +x target/release/botticelli-mcp
```

## How It Works

```
GitHub Copilot CLI (this session)
        ↓
  Reads .vscode/mcp.json
        ↓
  Launches botticelli-mcp
        ↓
  stdio (JSON-RPC 2.0)
        ↓
  Tool: query_content
        ↓
  PostgreSQL Database
        ↓
  Results returned to Copilot
        ↓
  Natural language response to you!
```

## Configuration Options

### Change Log Level

In `.vscode/mcp.json`:
```json
{
  "env": {
    "RUST_LOG": "debug"  // or "trace" for maximum detail
  }
}
```

### Use Different Database

```json
{
  "env": {
    "DATABASE_URL": "postgres://other:pass@host:5432/db"
  }
}
```

### Add Environment Variables

```json
{
  "env": {
    "DATABASE_URL": "postgres://...",
    "RUST_LOG": "info",
    "CUSTOM_VAR": "value"
  }
}
```

## What You Can Ask

### Database Queries

```
Show me the latest 10 rows from the content table
```

```
How many records are in the discord_guilds table?
```

```
Query the approved_discord_posts table
```

### Server Info

```
What MCP tools are available?
```

```
What version is the Botticelli MCP server?
```

### Testing

```
Echo "hello world" through the MCP server
```

```
Test the MCP connection
```

## Advanced Usage

### Combining Multiple Tools

```
First get server info, then query the content table
```

Copilot will make multiple MCP calls in sequence.

### Filtering Results

```
Query the content table with limit 5 and explain what each field means
```

Copilot will query and interpret results.

### Data Analysis

```
Query the content table and analyze the most common patterns
```

Copilot queries data and performs analysis.

## Limitations (Current Phase 1)

- ✅ **Supported:** Read-only queries
- ❌ **Not yet:** Write operations (INSERT, UPDATE, DELETE)
- ❌ **Not yet:** Narrative execution
- ❌ **Not yet:** Social media posting
- ❌ **Not yet:** Streaming results

See [MCP_INTEGRATION_STRATEGIC_PLAN.md](./MCP_INTEGRATION_STRATEGIC_PLAN.md) for roadmap.

## Benefits for Your Workflow

**Before MCP:**
```bash
$ psql postgres://... -c "SELECT * FROM content LIMIT 5"
# Copy results
# Paste into Copilot
# Ask questions
```

**With MCP:**
```
Show me the latest 5 content entries and summarize them
```

**That's it!** Copilot handles the query automatically.

## Comparison: Desktop vs CLI

| Feature | Claude Desktop | Copilot CLI |
|---------|---------------|-------------|
| Config Location | `~/Library/Application Support/Claude/` | `.vscode/mcp.json` |
| Interface | GUI App | Terminal/CLI |
| This Project | ✅ Configured | ✅ Configured |
| MCP Support | ✅ Native | ✅ Native |
| Database Access | ✅ Yes | ✅ Yes |

**Both work!** Use whichever you prefer.

## Next Steps

1. **Try it now!** Ask a question in this Copilot session:
   ```
   Query the content table and show me what's there
   ```

2. **Check logs** if issues occur:
   ```bash
   RUST_LOG=debug ./target/release/botticelli-mcp
   ```

3. **Experiment** with different queries:
   ```
   What tables exist in the database?
   Show me discord guild data
   Query the community_rules table
   ```

4. **Give feedback** on what works and what doesn't!

## Status

✅ **MCP Server:** Built and ready  
✅ **Configuration:** Created (`.vscode/mcp.json`)  
✅ **Database:** Connected (PostgreSQL)  
✅ **Tools:** 3 available (echo, get_server_info, query_content)  
✅ **Binary:** Optimized release build  
✅ **Copilot CLI:** Should auto-detect on next restart/reload  

**Ready to use RIGHT NOW in this session!** 🚀

---

*For more details, see [MCP.md](./MCP.md) - Complete MCP reference documentation*
