# TUI Server Management Integration

## Overview

Add local inference server management capabilities to the TUI, allowing users to download models, start/stop servers, and monitor server status - all from within the existing TUI interface.

## Design Principles

1. **Non-blocking**: Server operations should not freeze the TUI
2. **Visual feedback**: Clear status indicators and progress bars
3. **Integrated workflow**: Server management fits naturally into existing TUI navigation
4. **Graceful degradation**: TUI works fine if server features aren't used

## Implementation Plan

### Phase 1: Add Server Management Mode

Add a new `AppMode::ServerManagement` that displays:
- Current server status (stopped/starting/running/error)
- Available models (downloaded/not downloaded)
- Model download progress
- Server configuration (port, model in use)
- Action buttons (start/stop server, download model, configure)

### Phase 2: Integrate ServerManager

1. Add `ServerManager` field to `App` struct (wrapped in `Arc<Mutex<>>` for thread safety)
2. Add async runtime (`tokio::runtime::Runtime`) to `App` for non-blocking operations
3. Update `App::new()` to initialize server manager with config from environment/file

### Phase 3: Add Server Actions

Add keyboard commands:
- `s` - Toggle to Server Management view
- `d` - Download model (shows model selection menu)
- `Space` - Start/Stop server
- `c` - Configure server (edit port, model selection)
- `r` - Refresh server status

### Phase 4: Background Status Monitoring

1. Spawn background task that periodically checks server health
2. Update `App` state with current status (CPU/memory usage, requests/sec)
3. Display status in status bar when server is running

### Phase 5: UI Components

Create new rendering functions in `ui.rs`:
- `render_server_management()` - Main server management view
- `render_server_status_bar()` - Compact status in other views
- `render_model_download_progress()` - Progress bar during downloads
- `render_model_selection_menu()` - Choose which model to download/use

## Data Flow

```
User Input (keyboard) 
  ↓
Event Handler 
  ↓
App::handle_event() 
  ↓
ServerManager (async operation via tokio)
  ↓
Update App state
  ↓
UI render reflects new state
```

## State Management

```rust
pub struct App {
    // Existing fields...
    
    /// Server manager (thread-safe)
    pub server_manager: Arc<Mutex<ServerManager>>,
    
    /// Tokio runtime for async operations
    pub runtime: tokio::runtime::Runtime,
    
    /// Current server status
    pub server_status: ServerStatus,
    
    /// Model download progress (0-100)
    pub download_progress: Option<f32>,
    
    /// Server metrics (requests/sec, etc)
    pub server_metrics: Option<ServerMetrics>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

#[derive(Debug, Clone)]
pub struct ServerMetrics {
    pub requests_per_second: f64,
    pub avg_response_time_ms: f64,
    pub active_requests: usize,
}
```

## UI Layout Example

```
┌─────────────────────────────────────────────────────────┐
│ Botticelli - Server Management             [Running ✓] │
├─────────────────────────────────────────────────────────┤
│                                                          │
│ Server Status:                                           │
│   ● Running on http://localhost:8080                     │
│   ● Model: mistral-7b-instruct-v0.2-q4                  │
│   ● Uptime: 2h 34m                                       │
│   ● Requests: 247 (1.2/sec avg)                         │
│   ● Avg Response: 1.2s                                   │
│                                                          │
│ Available Models:                                        │
│ ● mistral-7b-instruct-v0.2-q4 [Downloaded] [Active]    │
│ ○ llama-3-8b-instruct-q4      [Not Downloaded]          │
│ ○ phi-3-mini-q4               [Not Downloaded]          │
│                                                          │
│ Actions:                                                 │
│   [Space] Stop Server                                    │
│   [d] Download Model                                     │
│   [c] Configure                                          │
│   [←] Back to Content                                    │
│                                                          │
├─────────────────────────────────────────────────────────┤
│ Status: Server running | [s] Server | [q] Quit          │
└─────────────────────────────────────────────────────────┘
```

## Integration with Existing Code

### Minimal Changes to Core TUI

- Add `ServerManagement` variant to `AppMode` enum
- Add server-related fields to `App` struct
- Add keyboard handlers for server commands
- Add `render_server_management()` branch to main render loop

### Thread Safety

Use `Arc<Mutex<ServerManager>>` to share server manager between:
- Main TUI thread (reading status, initiating commands)
- Background monitoring thread (updating status)
- Async runtime (executing server operations)

## Benefits

1. **User-friendly**: No need to manually run server commands
2. **Integrated**: See server status while working with content
3. **Safe**: Server lifecycle managed by application
4. **Visible**: Progress bars and status make operations transparent

## Next Steps

1. Implement Phase 1 (new app mode and basic UI)
2. Wire up ServerManager integration
3. Add keyboard command handlers
4. Implement background status monitoring
5. Test with real model download and server lifecycle
