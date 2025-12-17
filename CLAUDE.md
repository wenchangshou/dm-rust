# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is an industrial device control system written in Rust, ported from a Node.js codebase. The system provides unified management and control of various hardware devices including projectors, LED displays, computers, and lighting systems via HTTP REST API.

**Language:** Rust (edition 2021)
**Key frameworks:** Tokio (async runtime), Axum (web framework), tokio-modbus, tokio-serial

## Build and Development Commands

```bash
# Build project
cargo build

# Build release version (optimized)
cargo build --release

# Run in development mode
cargo run

# Run with custom config file
cargo run -- --config config.custom.json

# Run with custom log level
cargo run -- --log-level debug

# Run tests
cargo test

# Run specific test
cargo test test_name

# Check code without building (fast)
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy

# Auto-reload during development (requires cargo-watch)
cargo install cargo-watch
cargo watch -x run
```

## Windows Service Commands

```bash
# Install as Windows service
cargo run -- --install

# Uninstall Windows service
cargo run -- --uninstall

# Service control
cargo run -- --service start
cargo run -- --service stop
cargo run -- --service restart
```

## Configuration

The system loads configuration from `config.json` by default. Key configuration examples:
- `config.example.json` - Full configuration with all protocols
- `config.minimal.json` - Minimal configuration for testing
- `config.classroom.json` - Real-world classroom scenario
- `config.mock.json` - Mock protocol for testing

See `doc/CONFIGURATION.md` for detailed configuration documentation.

## Core Architecture

The system follows a **modular, event-driven architecture** with clear separation of concerns:

### 1. DeviceController (src/device/mod.rs)
The main facade coordinating all subsystems. Key responsibilities:
- Coordinates all device operations
- Provides unified API: `write_node()`, `read_node()`, `execute_scene()`
- Manages event broadcasting via `tokio::sync::broadcast`
- Does NOT directly interact with hardware

### 2. ChannelManager (src/device/channel_manager.rs)
Manages physical communication channels. Each channel wraps a protocol instance:
- Uses `DashMap<ChannelId, Channel>` for lock-free concurrent access
- Each Channel contains `Arc<RwLock<Box<dyn Protocol>>>` for thread-safe protocol access
- Isolates protocol implementation details from the rest of the system

### 3. NodeManager (src/device/node_manager.rs)
Maintains node configuration and runtime state:
- Stores node configs: `DashMap<GlobalId, NodeConfig>`
- Stores runtime state: `DashMap<GlobalId, NodeState>`
- NodeState includes: current_value, online status, last_update timestamp
- Automatically triggers events on state changes

### 4. DependencyResolver (src/device/dependency_resolver.rs)
Handles node dependency checking and auto-fulfillment:
- Supports value dependencies: `{id: 1, value: 1}` - node 1 must be 1
- Supports status dependencies: `{id: 2, status: true}` - node 2 must be online
- Strategy "auto": automatically fulfills dependencies before operation
- Strategy "manual": only checks dependencies, fails if not met

### 5. TaskScheduler (src/device/task_scheduler.rs)
Manages queued tasks waiting for dependencies:
- Background task checking queue every 500ms (configurable)
- Default timeout: 5 seconds
- Max retries: 3
- Tasks stored in VecDeque for FIFO processing

### 6. SceneExecutor (src/device/scene_executor.rs)
Orchestrates multi-node operations:
- Executes scenes defined in config
- Supports inter-node delays
- Emits progress events

## Protocol System

### Protocol Trait (src/protocols/mod.rs)

All protocols implement the `Protocol` trait with key methods:
- `from_config()` - Create protocol instance from config parameters
- `execute()` - Execute protocol-specific command
- `write()` / `read()` - Simplified data access
- `call_method()` - Custom method invocation (optional)
- `get_methods()` - List available custom methods (optional)

**Design principle:** Framework defines the standard, protocols parse their own config.

### Supported Protocols

Located in `src/protocols/`:
- `pjlink.rs` - PJLink projector control
- `modbus.rs` - Standard Modbus protocol
- `modbus_slave.rs` - Modbus gateway (manages multiple Modbus channels)
- `xinke_q1.rs` - Xinke Q1 power control module
- `computer_control.rs` - Wake-on-LAN
- `custom.rs` - Custom protocol support
- `screen_njlg_plc.rs` - Screen PLC control
- `hs_power_sequencer.rs` - HS power sequencer
- `novastar.rs` - Novastar LED control
- `mock.rs` - Mock protocol for testing

### Adding a New Protocol

1. Create new file in `src/protocols/` (e.g., `my_protocol.rs`)
2. Define protocol config struct with `#[derive(Deserialize)]`
3. Implement `Protocol` trait with `#[async_trait]`
4. Add to `src/protocols/mod.rs` exports
5. Add to `create_protocol()` match in `src/device/channel_manager.rs`
6. Add enum variant to `StatuteType` in `src/config/mod.rs`

Example structure:
```rust
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct MyProtocolConfig {
    addr: String,
    port: u16,
    // protocol-specific fields
}

pub struct MyProtocol {
    channel_id: u32,
    // fields
}

#[async_trait]
impl Protocol for MyProtocol {
    fn from_config(channel_id: u32, params: &HashMap<String, Value>) -> Result<Box<dyn Protocol>> {
        let config: MyProtocolConfig = serde_json::from_value(
            serde_json::to_value(params)?
        )?;
        // create and return instance
    }

    async fn execute(&mut self, command: &str, params: Value) -> Result<Value> {
        // implement protocol logic
    }

    // implement other required methods
}
```

## Web API (src/web/)

HTTP REST API on port 8080 (configurable). Key endpoints:

**Device Control:**
- `POST /lspcapi/device/write` - Write to node (control device)
- `POST /lspcapi/device/read` - Read node state
- `POST /lspcapi/device/writeMany` - Write multiple nodes
- `POST /lspcapi/device/readMany` - Read multiple nodes
- `POST /lspcapi/device/getAllStatus` - Get all channel status
- `POST /lspcapi/device/getAllNodeStates` - Get all node states
- `POST /lspcapi/device/scene` - Execute scene
- `POST /lspcapi/device/executeCommand` - Execute channel command
- `POST /lspcapi/device/callMethod` - Call custom protocol method
- `POST /lspcapi/device/getMethods` - List protocol methods
- `POST /lspcapi/device/batchRead` - Batch read from multiple channels/protocols

**File Management** (if enabled in config):
- `GET /lspcapi/file/list` - List files
- `POST /lspcapi/file/upload` - Upload file
- `GET /lspcapi/file/download` - Download file
- `DELETE /lspcapi/file/delete` - Delete file

**Database API** (if enabled):
- Screen management: `/lspcapi/db/screens/*`
- Material management: `/lspcapi/db/materials/*`

**Documentation:**
- `GET /swagger-ui/` - Swagger API documentation (uses utoipa)

See `doc/HTTP_API.md` for complete API documentation.

## Event System

The system uses `tokio::sync::broadcast` for event-driven notifications:

```rust
pub enum DeviceEvent {
    NodeStateChanged { global_id, old_value, new_value },
    ChannelConnected { channel_id },
    ChannelDisconnected { channel_id, reason },
    TaskCompleted { task_id, success },
    SceneStarted { scene_name },
    SceneCompleted { scene_name, success },
}
```

Subscribe to events via `controller.subscribe_events()` to receive real-time notifications.

## Concurrency Model

- **Runtime:** Tokio async runtime with work-stealing scheduler
- **Thread safety:** All state uses `Arc` + `DashMap` or `RwLock`
- **Lock-free reads:** DashMap for concurrent HashMap access
- **Broadcast events:** Multi-producer multi-consumer via `broadcast::channel`

**Important:** All protocol instances are wrapped in `Arc<RwLock<>>` - acquire write lock only when needed.

## Error Handling

Custom error types in `src/utils/error.rs`:
- `DeviceError` - Main error enum
- Uses `thiserror` for error derivation
- All async operations return `Result<T, DeviceError>`

**Convention:** Always use `?` operator for error propagation, never silently ignore errors.

## Database Integration (Optional)

If database config is enabled, uses `sqlx` with MySQL:
- Connection pooling via sqlx::MySqlPool
- Repositories in `src/db/`:
  - `screen_repo.rs` - Screen management
  - `material_repo.rs` - Material management
- Models in `src/db/models.rs`

Database URL format: `mysql://username:password@host:port/database`

## Logging

Uses `tracing` for structured logging:
- Log levels: trace, debug, info, warn, error
- Configure via config file `log` section or `--log-level` flag
- Supports console output, file output, or both
- Filter by module: `RUST_LOG=dm_rust::protocols=debug cargo run`

## Key Differences from Node.js Version

1. **Type Safety:** Compile-time guarantees vs runtime checks
2. **Concurrency:** True multi-threading with safety guarantees
3. **Error Handling:** Explicit Result types, no silent failures
4. **Memory Safety:** No garbage collector, ownership system prevents leaks
5. **Performance:** ~3-5x faster startup, ~50% less memory usage

## Testing Guidelines

- Unit tests in same file as implementation using `#[cfg(test)]`
- Integration tests in `tests/` directory
- Use `mockall` for mocking (see dev-dependencies)
- Mock protocol available for testing (`src/protocols/mock.rs`)

## Common Development Patterns

**Reading config:**
```rust
let cfg = config::load_config_from_file("config.json")?;
```

**Creating controller:**
```rust
let controller = DeviceController::new(cfg).await?;
```

**Writing to device:**
```rust
controller.write_node(global_id, value).await?;
```

**Subscribing to events:**
```rust
let mut events = controller.subscribe_events();
tokio::spawn(async move {
    while let Ok(event) = events.recv().await {
        // handle event
    }
});
```

## Documentation

Extensive documentation in `doc/` directory:
- `ARCHITECTURE.md` - Detailed architecture design (Chinese)
- `QUICKSTART.md` - Quick start guide
- `HTTP_API.md` - Complete HTTP API reference
- `CONFIGURATION.md` - Configuration guide
- `CUSTOM_METHODS_GUIDE.md` - Custom protocol methods
- `BATCH_READ_API.md` - Batch read functionality
- Protocol-specific guides: MODBUS, PJLink, NovaStar, etc.

## Important Notes

- **Default config path:** `config.json` in working directory
- **Default HTTP port:** 8080
- **WebSocket legacy port:** 9000 (if configured)
- **Config is loaded once at startup** - changes require restart
- **All node IDs (global_id) must be unique** across the system
- **Channel IDs must be unique** and referenced correctly in node configs
- **Scenes execute sequentially** with optional delays between operations
