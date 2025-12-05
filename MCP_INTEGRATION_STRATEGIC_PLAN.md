# MCP Server Integration Strategic Plan

## Executive Summary

**Recommendation: ✅ HIGHLY FEASIBLE AND RECOMMENDED**

Integrating Model Context Protocol (MCP) server capabilities into Botticelli would be a strategic enhancement that aligns perfectly with the project's multi-LLM architecture and positions it as a comprehensive AI orchestration platform.

## What is MCP?

Model Context Protocol (MCP) is an open protocol developed by Anthropic that enables LLMs to securely access external tools, data sources, and services. It provides a standardized way for AI applications to interact with:

- **Tools**: Functions LLMs can call (APIs, calculations, file operations)
- **Resources**: Data sources LLMs can read (files, databases, APIs)
- **Prompts**: Reusable prompt templates
- **Context**: Structured information about the environment

Think of MCP as "function calling" but standardized across all LLM providers and client applications.

## Why This Makes Sense for Botticelli

### Perfect Alignment

**Botticelli's Current Architecture:**
```
User → Botticelli → Multiple LLM Providers (Gemini, Claude, Groq, etc.)
```

**With MCP:**
```
User → Botticelli → Multiple LLM Providers
                 ↓
            MCP Server (tools, resources, context)
                 ↓
         External Systems (DB, APIs, Files)
```

### Strategic Benefits

1. **Unified Tool Interface**: Single tool definition works across all providers
2. **Enhanced Narratives**: Multi-act workflows can use tools dynamically
3. **Database Integration**: Expose Botticelli's PostgreSQL as MCP resources
4. **Social Media Tools**: Discord commands become MCP tools
5. **Content Pipeline**: Generation → Storage → Curation via MCP
6. **Multi-Agent Coordination**: Different LLMs can share tool access

## Feasibility Analysis

### ✅ Technical Feasibility: HIGH

**Available Rust SDKs:**

1. **Official SDK** (`mcp-server` v0.1.0)
   - From modelcontextprotocol/rust-sdk
   - Official Anthropic implementation
   - MIT licensed
   - Basic but stable

2. **rust-mcp-sdk** (v0.7.4)
   - More feature-complete
   - 104KB, actively maintained
   - Async/await support
   - Multiple transport layers (stdio, SSE, HTTP)
   - OAuth 2.1 support

3. **turbomcp** (v2.3.0-rc)
   - High-performance implementation
   - Ergonomic macros
   - Context management

**Recommendation**: Start with **rust-mcp-sdk** for its maturity and features.

### ✅ Architectural Fit: EXCELLENT

Botticelli already has the infrastructure:

- ✅ **Async runtime** (tokio) - MCP is async
- ✅ **Multi-provider support** - Can expose tools to any LLM
- ✅ **Database layer** - Can expose as MCP resources
- ✅ **Trait-based design** - Easy to add MCP as another interface
- ✅ **Error handling** - Existing patterns work for MCP
- ✅ **Observability** - Tracing can track tool calls

### ✅ Use Case Fit: COMPELLING

**Immediate Use Cases:**

1. **Database Query Tools**
   ```rust
   // LLM can query content via MCP
   Tool: "search_content"
   → Botticelli DB query
   → Structured results to LLM
   ```

2. **Social Media Integration**
   ```rust
   // LLM can post via MCP
   Tool: "post_to_discord"
   → Botticelli social layer
   → Discord API call
   ```

3. **Narrative Composition**
   ```rust
   // LLM discovers available narratives
   Resource: "list_narratives"
   → Returns .toml files
   → LLM can chain narratives
   ```

4. **File Storage**
   ```rust
   // LLM can store/retrieve media
   Tool: "store_image"
   → Content-addressable storage
   → Returns hash for retrieval
   ```

## Implementation Strategy

### Phase 1: Core MCP Server (Week 1)

**Goal**: Basic MCP server with 1-2 tools

**Tasks:**
1. Add `rust-mcp-sdk` dependency
2. Create `botticelli_mcp` crate
3. Implement basic server with stdio transport
4. Define 2 simple tools:
   - `echo` - Test tool
   - `get_server_info` - Return Botticelli version/status

**Deliverables:**
- Working MCP server
- Can connect via MCP client
- Tools callable from Claude Desktop

### Phase 2: Database Tools (Week 2)

**Goal**: Expose database as MCP resources

**Tools to implement:**
1. `list_tables` - Show available DB tables
2. `query_content` - Search content table
3. `get_content_by_id` - Retrieve specific content
4. `list_narratives` - Show available narratives

**Resources to expose:**
1. `content://{table}/{id}` - Individual content items
2. `narrative://{name}` - Narrative TOML files

### Phase 3: Execution Tools (Week 3)

**Goal**: LLMs can trigger Botticelli actions

**Tools:**
1. `execute_narrative` - Run a narrative with parameters
2. `generate_with_model` - Direct LLM call
3. `store_output` - Save generation to database

### Phase 4: Social Media Tools (Week 4)

**Goal**: LLMs can interact with social platforms

**Tools:**
1. `post_to_discord` - Send Discord message
2. `get_discord_channels` - List available channels
3. `approve_content` - Mark content for posting

### Phase 5: Advanced Features (Week 5-6)

**Advanced capabilities:**
1. **Streaming** - Stream tool outputs for long-running operations
2. **Sampling** - Let tools request LLM help
3. **Roots** - Expose filesystem roots for file access
4. **Context Management** - Track conversation state across tools

## Architecture Design

### Crate Structure

```
crates/
├── botticelli_mcp/           # NEW: MCP server implementation
│   ├── src/
│   │   ├── lib.rs            # Server setup
│   │   ├── tools/            # Tool implementations
│   │   │   ├── mod.rs
│   │   │   ├── database.rs   # DB query tools
│   │   │   ├── narrative.rs  # Narrative tools
│   │   │   ├── social.rs     # Social media tools
│   │   │   └── system.rs     # System info tools
│   │   ├── resources/        # Resource handlers
│   │   │   ├── mod.rs
│   │   │   ├── content.rs
│   │   │   └── narratives.rs
│   │   ├── prompts/          # Prompt templates
│   │   │   └── mod.rs
│   │   └── server.rs         # MCP server core
│   └── Cargo.toml
└── botticelli/               # MODIFIED: Add mcp feature
    ├── src/
    │   └── lib.rs            # Re-export MCP server
    └── Cargo.toml            # Add mcp feature
```

### Trait-Based Design

```rust
// botticelli_mcp/src/tools/mod.rs
#[async_trait]
pub trait McpTool: Send + Sync {
    /// Tool name
    fn name(&self) -> &'static str;
    
    /// Tool description for LLM
    fn description(&self) -> &'static str;
    
    /// Input schema (JSON Schema)
    fn input_schema(&self) -> serde_json::Value;
    
    /// Execute tool with inputs
    async fn execute(&self, input: serde_json::Value) 
        -> Result<serde_json::Value, McpError>;
}

// Example implementation
pub struct QueryContentTool {
    db_pool: DatabasePool,
}

#[async_trait]
impl McpTool for QueryContentTool {
    fn name(&self) -> &'static str { "query_content" }
    
    fn description(&self) -> &'static str {
        "Search the content database by text or filters"
    }
    
    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "query": {"type": "string"},
                "limit": {"type": "integer", "default": 10}
            }
        })
    }
    
    async fn execute(&self, input: serde_json::Value) 
        -> Result<serde_json::Value, McpError> {
        // Parse input
        // Query database
        // Return results
    }
}
```

### Integration with Existing Code

```rust
// Botticelli MCP server leverages existing infrastructure

pub struct BotticelliMcpServer {
    // Reuse existing components
    database: Arc<DatabaseRepository>,
    narrative_executor: Arc<NarrativeExecutor>,
    social_client: Arc<DiscordClient>,
    drivers: HashMap<String, Arc<dyn BotticelliDriver>>,
    
    // MCP-specific
    tools: Vec<Arc<dyn McpTool>>,
    resources: Vec<Arc<dyn McpResource>>,
}

impl BotticelliMcpServer {
    pub fn new(
        database: Arc<DatabaseRepository>,
        narrative_executor: Arc<NarrativeExecutor>,
        social_client: Arc<DiscordClient>,
    ) -> Self {
        let mut server = Self {
            database: database.clone(),
            narrative_executor,
            social_client,
            drivers: HashMap::new(),
            tools: vec![],
            resources: vec![],
        };
        
        // Register tools
        server.register_tool(Arc::new(
            QueryContentTool::new(database.clone())
        ));
        
        server.register_tool(Arc::new(
            ExecuteNarrativeTool::new(narrative_executor.clone())
        ));
        
        server
    }
}
```

## Deployment Options

### Option 1: Embedded Server (Recommended for Phase 1)

Run MCP server in same process as Botticelli:

```rust
// botticelli binary with --mcp flag
async fn main() {
    if args.mcp {
        // Start MCP server on stdio
        let server = BotticelliMcpServer::new(...);
        server.run_stdio().await?;
    } else {
        // Normal Botticelli operation
        run_normal_mode().await?;
    }
}
```

**Pros:**
- Simple deployment
- Direct access to Botticelli internals
- No IPC overhead

**Cons:**
- Single process
- Can't hot-reload tools

### Option 2: Separate MCP Binary

Dedicated MCP server binary:

```bash
# Terminal 1: Botticelli server
cargo run --release --features database,discord

# Terminal 2: MCP server
cargo run --release --bin botticelli-mcp-server
```

**Pros:**
- Independent scaling
- Can restart MCP without affecting main server
- Multiple MCP servers for different use cases

**Cons:**
- IPC overhead
- More complex deployment

### Option 3: HTTP/SSE Transport

Expose MCP over HTTP:

```rust
// Run MCP server on HTTP
server.run_http("0.0.0.0:3001").await?;
```

**Pros:**
- Network-accessible
- Can serve multiple clients
- Easy to load balance

**Cons:**
- Authentication required
- Network latency

**Recommendation**: Start with Option 1 (embedded stdio), add Option 3 (HTTP) in Phase 5.

## Integration with Claude Desktop

Users can add Botticelli MCP server to Claude Desktop config:

```json
{
  "mcpServers": {
    "botticelli": {
      "command": "botticelli",
      "args": ["--mcp"],
      "env": {
        "DATABASE_URL": "postgresql://...",
        "DISCORD_TOKEN": "..."
      }
    }
  }
}
```

Now Claude Desktop can:
- Query Botticelli's database
- Execute narratives
- Post to Discord
- All via natural language!

## Security Considerations

### Critical: Tool Authorization

```rust
pub struct McpSecurityContext {
    allowed_tools: HashSet<String>,
    allowed_resources: HashSet<String>,
    max_query_results: usize,
    rate_limits: RateLimitConfig,
}

impl BotticelliMcpServer {
    pub fn with_security(mut self, ctx: McpSecurityContext) -> Self {
        self.security = Some(ctx);
        self
    }
    
    async fn check_tool_allowed(&self, tool: &str) -> Result<(), McpError> {
        if let Some(sec) = &self.security {
            if !sec.allowed_tools.contains(tool) {
                return Err(McpError::ToolNotAllowed(tool.to_string()));
            }
        }
        Ok(())
    }
}
```

### Security Best Practices

1. **Allowlist Tools**: Only expose safe tools by default
2. **Rate Limiting**: Prevent abuse of expensive operations
3. **Input Validation**: Sanitize all tool inputs
4. **Audit Logging**: Log all tool calls with full context
5. **Database Permissions**: Use read-only connections for query tools
6. **Resource Limits**: Cap query results, execution time
7. **Authentication**: Require auth for HTTP transport

## Benefits Summary

### For Users

1. **Natural Language Interface**: "Query content about X" → Claude uses MCP tool
2. **Unified Experience**: Same tools work in Claude Desktop, API, or custom clients
3. **Composability**: Chain Botticelli tools with other MCP servers
4. **Visibility**: See tool calls in Claude Desktop UI

### For Developers

1. **Standardized**: MCP is becoming industry standard (Anthropic, Zed, Cody)
2. **Reusable**: Tools defined once, work everywhere
3. **Extensible**: Easy to add new tools as Botticelli grows
4. **Observable**: MCP has built-in logging/debugging
5. **Type-Safe**: Rust + JSON Schema = compile-time safety

### For Botticelli Project

1. **Differentiation**: Few LLM orchestrators have MCP support
2. **Ecosystem Integration**: Works with Claude Desktop, cursor, etc.
3. **Future-Proof**: MCP is actively developed by Anthropic
4. **Community**: MCP has growing ecosystem and tooling
5. **Marketing**: "First Rust LLM orchestrator with built-in MCP server"

## Risks and Mitigations

### Risk 1: SDK Immaturity

**Risk**: Rust MCP SDKs are relatively new  
**Severity**: Medium  
**Mitigation**: 
- Start with stable `rust-mcp-sdk` v0.7.4
- Implement minimal tools first
- Can fall back to manual JSON-RPC if needed

### Risk 2: Performance Overhead

**Risk**: MCP adds latency to tool calls  
**Severity**: Low  
**Mitigation**:
- Stdio transport is fast (< 1ms overhead)
- Tools async, don't block server
- Can cache frequent queries

### Risk 3: Complexity

**Risk**: Adds another layer to maintain  
**Severity**: Medium  
**Mitigation**:
- Keep tools simple (thin wrappers)
- Reuse existing Botticelli code
- Feature-gate behind `mcp` feature
- Excellent test coverage

### Risk 4: Security

**Risk**: LLMs could abuse tools  
**Severity**: High  
**Mitigation**:
- Implement strict authorization
- Rate limiting on all tools
- Read-only database access
- Audit logging
- User confirmation for destructive ops

## Why NOT to Do This

**Devil's Advocate Arguments:**

1. **Scope Creep**: Botticelli is already complex
   - **Counter**: MCP is thin layer, reuses existing code

2. **Early Adoption Risk**: MCP protocol could change
   - **Counter**: Version 2025-06-18 is stable, feature-gate allows rollback

3. **Limited Adoption**: What if MCP doesn't become standard?
   - **Counter**: Already adopted by Anthropic, Zed, Cody, Cursor - momentum is strong

4. **Maintenance Burden**: Another system to maintain
   - **Counter**: ~1000 lines of code, minimal surface area

5. **User Confusion**: Tools add complexity for end users
   - **Counter**: Optional feature, power users benefit greatly

**Verdict**: Counterarguments outweigh concerns. Benefits > Risks.

## Timeline and Effort

### Estimated Effort

- **Phase 1** (Core): 1 week, ~40 hours
- **Phase 2** (Database): 1 week, ~40 hours
- **Phase 3** (Execution): 1 week, ~40 hours
- **Phase 4** (Social): 1 week, ~40 hours
- **Phase 5** (Advanced): 2 weeks, ~80 hours

**Total**: 6 weeks, ~240 hours for full implementation

### Minimal Viable Product (MVP)

For quick validation:
- **Phase 1 only**: 1 week
- 2-3 basic tools
- Stdio transport
- Works with Claude Desktop

Can decide whether to proceed after MVP validation.

## Comparison with Alternatives

### Alternative 1: Custom Tool Protocol

**Pros**: Complete control  
**Cons**: No ecosystem, have to maintain spec, no client support  
**Verdict**: Reinventing the wheel - MCP is better

### Alternative 2: Function Calling Only

**Pros**: Simpler, provider-native  
**Cons**: Provider-specific, no standardization, limited to supported LLMs  
**Verdict**: MCP is superset - can support both

### Alternative 3: No Tool Support

**Pros**: Simplest  
**Cons**: Limits Botticelli to "dumb" orchestration  
**Verdict**: Misses major opportunity for differentiation

**Recommendation**: MCP is the right choice

## Conclusion

### Final Recommendation: ✅ PROCEED

**Reasons:**

1. ✅ **Technical Feasibility**: High - mature SDK, fits architecture
2. ✅ **Strategic Fit**: Excellent - aligns with multi-LLM vision
3. ✅ **User Value**: High - enables new use cases
4. ✅ **Risk/Reward**: Favorable - moderate effort, high payoff
5. ✅ **Timing**: Perfect - MCP gaining momentum, early mover advantage

### Proposed Next Steps

1. **Create RFC**: Share this plan with stakeholders
2. **Prototype**: 2-day spike with rust-mcp-sdk
3. **Validate**: Test with Claude Desktop
4. **Decide**: Go/no-go after prototype
5. **Implement**: Phase 1 MVP if go-ahead

### Success Metrics

After Phase 1:
- ✅ MCP server connects to Claude Desktop
- ✅ At least 2 tools working
- ✅ Tool calls traced in observability
- ✅ Zero security incidents
- ✅ < 5ms tool call overhead

### The Vision

Imagine a user saying to Claude:

> "Use Botticelli to generate a blog post about Rust, store it in the content database, have it reviewed, and post the approved version to Discord."

Claude uses MCP to:
1. Call `execute_narrative` with blog post generation narrative
2. Call `store_content` to save result
3. Call `query_content` to retrieve it
4. Call `execute_narrative` with review narrative
5. Call `approve_content` to mark for posting
6. Call `post_to_discord` to publish

All orchestrated by natural language, powered by Botticelli + MCP.

**That's the future we're building.** 🚀
