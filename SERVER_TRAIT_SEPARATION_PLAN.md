# Server Trait Separation Plan

## Overview

Currently, `botticelli_server` contains MistralRS-specific implementation code that depends on the `mistralrs` crate from GitHub. To publish Botticelli on crates.io, we need to:

1. Extract MistralRS-specific code into a separate external crate (`botticelli_mistral`)
2. Create a generic server trait interface in `botticelli_server`
3. Make the external crate implement the trait interface

## Current State

### botticelli_server Structure

- **Location**: `/home/erik/repos/botticelli/crates/botticelli_server`
- **Dependencies**: `mistralrs` (git), `hf-hub`, standard Botticelli crates
- **Key Components**:
  - `server.rs` - `ServerHandle` for process lifecycle (mistralrs-specific)
  - `client.rs` - `ServerClient` for OpenAI-compatible API calls (generic)
  - `models.rs` - Model catalog and downloader (mistralrs-specific GGUF models)
  - `config.rs` - Server configuration (generic)
  - `request.rs`, `response.rs` - OpenAI API types (generic)
  - `convert.rs` - Conversion between Botticelli and OpenAI types (generic)
  - `main.rs` - CLI binary for server management

### Dependencies

**botticelli_server → mistralrs (git)**: Cannot publish to crates.io
**botticelli (CLI) → botticelli_server**: Uses server management commands

## Target State

### 1. botticelli_server (Trait Interface)

**Purpose**: Generic server interface that can be published on crates.io

**Contents**:
- Generic server lifecycle traits
- OpenAI-compatible client (no implementation-specific code)
- Request/response types
- Configuration abstractions
- No dependency on `mistralrs` or any specific inference engine

**Exports**:
```rust
// Trait interfaces
pub trait InferenceServer {
    fn port(&self) -> u16;
    fn base_url(&self) -> String;
    async fn health_check(&self) -> Result<(), ServerError>;
    async fn wait_until_ready(&self, timeout: Duration) -> Result<(), ServerError>;
    fn stop(self) -> Result<(), ServerError>;
}

pub trait ServerLauncher {
    type Server: InferenceServer;
    type Config;
    
    fn start(config: Self::Config) -> Result<Self::Server, ServerError>;
}

pub trait ModelManager {
    type ModelSpec;
    
    fn is_downloaded(&self, spec: Self::ModelSpec) -> bool;
    async fn download(&self, spec: Self::ModelSpec) -> Result<PathBuf, ServerError>;
    fn model_path(&self, spec: Self::ModelSpec) -> PathBuf;
}

// Generic client (already mostly generic)
pub struct ServerClient { ... }

// OpenAI API types
pub struct ChatCompletionRequest { ... }
pub struct ChatCompletionResponse { ... }
```

### 2. botticelli_mistral (External Implementation)

**Location**: `/home/erik/repos/botticelli_mistral` (outside workspace)

**Purpose**: MistralRS-specific implementation of server traits

**Contents**:
- `MistralServer` - implements `InferenceServer` trait
- `MistralLauncher` - implements `ServerLauncher` trait
- `MistralModelManager` - implements `ModelManager` trait
- `MistralModelSpec` - GGUF model catalog
- Process spawning code (current `ServerHandle`)
- HuggingFace model downloading
- CLI binary for mistral-specific server management

**Dependencies**:
- `botticelli_server` (from crates.io eventually)
- `mistralrs` (git) - OK here since it's external
- `hf-hub`

**Example Implementation**:
```rust
pub struct MistralServer {
    process: Child,
    port: u16,
    base_url: String,
}

impl InferenceServer for MistralServer {
    fn port(&self) -> u16 { self.port }
    fn base_url(&self) -> String { self.base_url.clone() }
    async fn health_check(&self) -> Result<(), ServerError> { ... }
    async fn wait_until_ready(&self, timeout: Duration) -> Result<(), ServerError> { ... }
    fn stop(self) -> Result<(), ServerError> { ... }
}

pub struct MistralLauncher;

impl ServerLauncher for MistralLauncher {
    type Server = MistralServer;
    type Config = MistralConfig;
    
    fn start(config: MistralConfig) -> Result<MistralServer, ServerError> {
        // Current ServerHandle::start_internal logic
    }
}
```

## Implementation Steps

### Phase 1: Design Trait Interface in botticelli_server

**Goal**: Define clean trait abstractions without breaking existing code

**Tasks**:
1. Create `traits.rs` module in `botticelli_server/src/`
2. Define `InferenceServer`, `ServerLauncher`, and `ModelManager` traits
3. Add trait exports to `lib.rs`
4. Run `cargo check` to ensure compilation

**Files Modified**:
- `crates/botticelli_server/src/traits.rs` (new)
- `crates/botticelli_server/src/lib.rs` (add exports)

**Validation**:
- `cargo check --package botticelli_server`
- Ensure no breaking changes to existing code

**Commit**: `feat(server): add trait interface for inference servers`

### Phase 2: Extract Generic Components

**Goal**: Separate generic code (client, types) from implementation-specific code

**Tasks**:
1. Review `client.rs` - already mostly generic, may need minor tweaks
2. Keep `config.rs`, `request.rs`, `response.rs`, `convert.rs` in `botticelli_server`
3. Mark `server.rs` and `models.rs` as temporary (will move in Phase 3)
4. Add deprecation notices to implementation-specific items

**Files Modified**:
- `crates/botticelli_server/src/client.rs` (review/minor edits)
- `crates/botticelli_server/src/server.rs` (add deprecation notice)
- `crates/botticelli_server/src/models.rs` (add deprecation notice)

**Validation**:
- `cargo check --package botticelli_server`
- `cargo test --package botticelli_server`

**Commit**: `refactor(server): prepare generic components for trait migration`

### Phase 3: Create botticelli_mistral External Crate

**Goal**: Set up new external crate structure

**Tasks**:
1. Create directory: `/home/erik/repos/botticelli_mistral`
2. Initialize with `cargo init --lib`
3. Set up `Cargo.toml` with dependencies:
   - `botticelli_server` (path = "../botticelli/crates/botticelli_server")
   - `botticelli_error` (path = "../botticelli/crates/botticelli_error")
   - `mistralrs` (git)
   - `hf-hub`
   - `tokio`, `async-trait`, `tracing`, etc.
4. Create module structure:
   - `src/lib.rs`
   - `src/server.rs` (MistralServer)
   - `src/launcher.rs` (MistralLauncher)
   - `src/models.rs` (MistralModelManager + ModelSpec)
   - `src/config.rs` (MistralConfig)
   - `src/main.rs` (CLI binary)

**New Files**:
- `/home/erik/repos/botticelli_mistral/Cargo.toml`
- `/home/erik/repos/botticelli_mistral/src/lib.rs`
- `/home/erik/repos/botticelli_mistral/src/server.rs`
- `/home/erik/repos/botticelli_mistral/src/launcher.rs`
- `/home/erik/repos/botticelli_mistral/src/models.rs`
- `/home/erik/repos/botticelli_mistral/src/config.rs`
- `/home/erik/repos/botticelli_mistral/src/main.rs`
- `/home/erik/repos/botticelli_mistral/README.md`

**Validation**:
- `cd /home/erik/repos/botticelli_mistral && cargo check`

**Commit** (in botticelli_mistral repo): `feat: initial MistralRS server implementation`

### Phase 4: Move MistralRS Code to botticelli_mistral

**Goal**: Copy and adapt current MistralRS implementation to new crate

**Tasks**:
1. **Copy `ServerHandle` logic** → `MistralServer` implementing `InferenceServer`
   - Process spawning
   - Health checks
   - Graceful shutdown
   - Drop implementation
2. **Copy model management** → `MistralModelManager` implementing `ModelManager`
   - ModelSpec enum and methods
   - HuggingFace downloading
   - Path management
3. **Create launcher** → `MistralLauncher` implementing `ServerLauncher`
   - Wrap server creation logic
4. **Create config** → `MistralConfig`
   - Model path, port, tokenizer ID
5. **Create CLI binary** → `main.rs`
   - Model download command
   - Server start/stop/status
   - Model listing

**Code Changes**:
```rust
// In botticelli_mistral/src/server.rs
pub struct MistralServer {
    process: Child,
    port: u16,
    model_path: String,
}

impl InferenceServer for MistralServer {
    fn port(&self) -> u16 { self.port }
    
    fn base_url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }
    
    async fn health_check(&self) -> Result<(), ServerError> {
        let client = reqwest::Client::new();
        let url = format!("{}/health", self.base_url());
        // ... current health check logic
    }
    
    async fn wait_until_ready(&self, timeout: Duration) -> Result<(), ServerError> {
        // ... current wait_until_ready logic
    }
    
    fn stop(mut self) -> Result<(), ServerError> {
        // ... current stop logic
    }
}

// In botticelli_mistral/src/launcher.rs
pub struct MistralLauncher;

impl ServerLauncher for MistralLauncher {
    type Server = MistralServer;
    type Config = MistralConfig;
    
    fn start(config: MistralConfig) -> Result<MistralServer, ServerError> {
        // ... current ServerHandle::start_internal logic
    }
}
```

**Validation**:
- `cd /home/erik/repos/botticelli_mistral && cargo check`
- `cd /home/erik/repos/botticelli_mistral && cargo test`

**Commit** (in botticelli_mistral repo): `feat: implement MistralRS server traits`

### Phase 5: Remove MistralRS from botticelli_server

**Goal**: Clean up workspace crate, remove git dependency

**Tasks**:
1. Remove `server.rs` from `botticelli_server`
2. Remove `models.rs` from `botticelli_server`
3. Remove `main.rs` binary (CLI moves to botticelli_mistral)
4. Update `Cargo.toml`:
   - Remove `mistralrs` dependency
   - Remove `hf-hub` dependency
   - Remove binary target
5. Update `lib.rs` exports:
   - Remove `ServerHandle`, `ModelManager`, `ModelSpec`
   - Keep traits, client, types
6. Update documentation to reference external implementations

**Files Deleted**:
- `crates/botticelli_server/src/server.rs`
- `crates/botticelli_server/src/models.rs`
- `crates/botticelli_server/src/main.rs`

**Files Modified**:
- `crates/botticelli_server/Cargo.toml` (remove deps, remove [[bin]])
- `crates/botticelli_server/src/lib.rs` (remove exports, add trait exports)
- `crates/botticelli_server/README.md` (update docs)

**Validation**:
- `cargo check --package botticelli_server`
- Ensure no references to removed code

**Commit**: `refactor(server)!: extract MistralRS to external crate`

### Phase 6: Update botticelli CLI

**Goal**: Remove server commands from main CLI, document external tool

**Tasks**:
1. Remove `ServerCommands` from `crates/botticelli/src/cli/mod.rs`
2. Remove `server.rs` from `crates/botticelli/src/cli/`
3. Update CLI help text to mention `botticelli_mistral` CLI
4. Update README to document external server crate

**Files Deleted**:
- `crates/botticelli/src/cli/server.rs`

**Files Modified**:
- `crates/botticelli/src/cli/mod.rs` (remove ServerCommands)
- `crates/botticelli/README.md` (update docs)
- Root `README.md` (mention server implementations)

**Validation**:
- `cargo check --package botticelli`
- `cargo test --package botticelli`
- Verify CLI still works without server commands

**Commit**: `refactor(cli)!: remove server commands, delegate to external crates`

### Phase 7: Update Workspace Configuration

**Goal**: Clean up workspace, document new structure

**Tasks**:
1. Update workspace `Cargo.toml` metadata
2. Update workspace README
3. Add documentation about external server implementations
4. Create migration guide for users

**Files Modified**:
- `Cargo.toml` (workspace metadata if needed)
- `README.md` (document architecture)
- `SERVER_TRAIT_SEPARATION_PLAN.md` → archive as implementation guide

**New Files**:
- `EXTERNAL_SERVERS.md` (guide for server implementations)

**Validation**:
- `cargo check --workspace`
- `cargo test --workspace --lib --tests` (local tests)
- `cargo clippy --all-targets`
- Review all documentation

**Commit**: `docs: update workspace for external server architecture`

### Phase 8: Publish Preparation

**Goal**: Ensure crates.io readiness

**Tasks**:
1. Audit all workspace crates for:
   - No git dependencies
   - All path dependencies publishable
   - Correct metadata (license, description, keywords)
2. Run `cargo publish --dry-run` for each crate
3. Test publishing to crates.io (optional test run)
4. Document publishing order

**Publishing Order**:
1. `botticelli_error`
2. `botticelli_core`
3. `botticelli_interface`
4. `botticelli_rate_limit`
5. `botticelli_storage`
6. `botticelli_models`
7. `botticelli_database`
8. `botticelli_narrative`
9. `botticelli_social`
10. `botticelli_server` (trait interface only, no mistralrs)
11. `botticelli_tui`
12. `botticelli` (main facade)

**External (not published as part of workspace)**:
- `botticelli_mistral` (separate repo, own versioning)

**Validation**:
- `cargo publish --dry-run` for each crate in order
- Check for dependency resolution issues

**Commit**: `chore: prepare workspace for crates.io publication`

## Benefits

### For Botticelli

1. **Publishable**: No git dependencies, can publish to crates.io
2. **Extensible**: Trait interface allows multiple server implementations
3. **Separation of Concerns**: Generic API client vs. specific server implementation
4. **Maintainability**: Changes to MistralRS don't affect core library

### For Users

1. **Choice**: Can implement traits for other inference engines (llama.cpp, candle, etc.)
2. **Flexibility**: Can use generic client with any OpenAI-compatible server
3. **Optional**: Server functionality is opt-in via external crates
4. **Clear Boundaries**: Interface vs. implementation is explicit

### For Ecosystem

1. **Standard Interface**: Other projects can implement the same traits
2. **Composability**: Mix and match server implementations
3. **Innovation**: Community can create alternative implementations

## Future Server Implementations

With the trait interface, users could create:

- `botticelli_llamacpp` - llama.cpp server wrapper
- `botticelli_candle` - Candle ML server integration
- `botticelli_ollama` - Ollama server wrapper
- `botticelli_vllm` - vLLM server integration

Each would implement `InferenceServer`, `ServerLauncher`, and `ModelManager` traits.

## Migration Guide (for Users)

### Before (current)

```rust
use botticelli_server::{ServerHandle, ServerConfig, ModelManager, ModelSpec};

// Download and start server
let manager = ModelManager::new("./models");
let model_path = manager.ensure_model(ModelSpec::Mistral7BInstructV03Q4).await?;

let config = ServerConfig::new("http://localhost:8080", "mistral-7b");
let server = ServerHandle::start(config, model_path, 8080)?;
server.wait_until_ready(Duration::from_secs(60)).await?;
```

### After (trait-based)

```rust
use botticelli_server::{ServerClient, ServerConfig};
use botticelli_mistral::{MistralLauncher, MistralConfig, MistralModelManager, MistralModelSpec};
use botticelli_server::{ServerLauncher, ModelManager}; // traits

// Download model
let manager = MistralModelManager::new("./models");
let model_path = manager.ensure_model(MistralModelSpec::Mistral7BInstructV03Q4).await?;

// Start server
let mistral_config = MistralConfig::new(model_path, 8080);
let server = MistralLauncher::start(mistral_config)?;
server.wait_until_ready(Duration::from_secs(60)).await?;

// Use generic client (same as before)
let config = ServerConfig::new("http://localhost:8080", "mistral-7b");
let client = ServerClient::new(config);
```

### CLI Commands

**Before**:
```bash
botticelli server download mistral-7b
botticelli server start mistral-7b
```

**After**:
```bash
# Use separate binary from botticelli_mistral crate
cargo install botticelli_mistral
botticelli-mistral download mistral-7b
botticelli-mistral start mistral-7b
```

## Risks and Mitigations

### Risk 1: Trait Design Complexity

**Risk**: Traits may be too generic or too specific

**Mitigation**:
- Start with minimal trait surface area
- Iterate based on second implementation attempt
- Keep traits focused on lifecycle, not inference details

### Risk 2: Breaking Changes

**Risk**: Users of current `botticelli_server` will face breaking changes

**Mitigation**:
- Version bump to 0.3.0 (breaking change signaled)
- Comprehensive migration guide
- Deprecation notices before removal
- Clear commit messages marking breaking changes

### Risk 3: External Crate Maintenance

**Risk**: `botticelli_mistral` becomes orphaned or out of sync

**Mitigation**:
- Document clearly in both repos
- Keep trait interface stable
- External crate is optional, not required for core functionality

### Risk 4: Path Dependencies During Development

**Risk**: External crate needs path deps during development but versions after publish

**Mitigation**:
- Use path dependencies initially
- Document how to switch to crates.io versions
- Test with published versions before major releases

## Success Criteria

- [ ] All workspace crates compile without git dependencies
- [ ] `cargo publish --dry-run` succeeds for all workspace crates
- [ ] `botticelli_mistral` successfully implements all traits
- [ ] Generic `ServerClient` works with MistralServer
- [ ] All tests pass in both workspace and external crate
- [ ] Documentation updated to reflect new architecture
- [ ] Migration guide written and tested
- [ ] CLI commands work from external crate binary

## Timeline Estimate

- **Phase 1-2**: 2-3 hours (trait design + generic extraction)
- **Phase 3-4**: 3-4 hours (external crate setup + code migration)
- **Phase 5-6**: 2-3 hours (cleanup workspace + CLI)
- **Phase 7-8**: 2-3 hours (documentation + publish prep)

**Total**: 9-13 hours of focused implementation work

## Notes

- This is a breaking change (0.2.0 → 0.3.0)
- External crate can have independent versioning
- Trait interface should remain stable even as implementations evolve
- Consider async-trait for trait definitions with async methods
- Keep trait interface in botticelli_server minimal to ease future implementations
