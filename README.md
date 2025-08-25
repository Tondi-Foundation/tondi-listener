
# Tondi Scan

Tondi Scan is a high-performance blockchain scanning service that supports multiple protocols (HTTP/2, gRPC, WebSocket) and database backends.

## Features

* 🚀 **High Performance**: Built with Rust and the Tokio async runtime
* 🔒 **Security**: Built-in rate limiting, request validation, and secure CORS configuration
* 📊 **Multi-protocol**: Supports HTTP/2, gRPC, WebSocket
* 🗄️ **Database**: Integrated with PostgreSQL and Diesel ORM
* 🌐 **WebAssembly**: Supports frontend integration
* 📝 **Logging**: Structured logs and tracing
* ⚙️ **Configuration**: Flexible, environment variable-driven configuration

## Quick Start

### Requirements

* Rust 1.75+ (nightly)
* PostgreSQL 12+
* WebAssembly-enabled browser (optional)

### Installation

1. Clone the repository

```bash
git clone <repository-url>
cd tondi-listener
```

2. Install dependencies

```bash
cargo build
```

3. Configure environment variables

```bash
cp env.example .env
# Edit the .env file and set your configuration
```

4. Start the server

```bash
# Start gRPC server
cargo run -p tondi-listener-server --bin server

# Start HTTP router
cargo run -p tondi-listener-server --bin router
```

## Configuration

### Configuration Priority

The configuration system follows this priority order (highest to lowest):

1. **Environment Variables** - Highest priority, override all other settings
2. **TOML Configuration File** - Second priority, used if environment variables are not set
3. **Default Values** - Lowest priority, used as fallback

### Environment Variables

| Variable             | Description                  | Default                                           |
| -------------------- | ---------------------------- | ------------------------------------------------- |
| `TONDI_SCAN_HOST_URL`     | Server listening address     | `127.0.0.1:3000`                                  |
| `TONDI_SCAN_GRPC_URL`     | gRPC service address         | `grpc://8.210.45.192:16610`                       |
| `TONDI_SCAN_DATABASE_URL` | PostgreSQL connection string | `postgres://postgres:postgres@127.0.0.1/postgres` |
| `TONDI_SCAN_ENVIRONMENT`  | Runtime environment          | `development`                                     |
| `TONDI_SCAN_LOG_LEVEL`    | Log level                    | `info`                                            |

### Event Configuration

| Variable                    | Description                           | Default                                    |
| --------------------------- | ------------------------------------- | ------------------------------------------ |
| `TONDI_SCAN_ENABLED_EVENTS` | Comma-separated blockchain events     | `block-added,utxos-changed,virtual-chain-changed` |
| `TONDI_SCAN_EVENT_STRATEGY` | Event processing strategy              | `real-time`                               |
| `TONDI_SCAN_BATCH_SIZE`     | Batch size for batch processing       | `100`                                     |
| `TONDI_SCAN_BATCH_TIMEOUT_MS` | Batch timeout in milliseconds         | `100`                                     |
| `TONDI_SCAN_BUFFER_SIZE`    | Event buffer size                      | `1000`                                    |
| `TONDI_SCAN_ENABLE_DEDUPLICATION` | Enable event deduplication           | `true`                                    |
| `TONDI_SCAN_HIGH_PRIORITY_EVENTS` | High priority events (comma-separated) | `block-added,utxos-changed`               |
| `TONDI_SCAN_MEDIUM_PRIORITY_EVENTS` | Medium priority events (comma-separated) | `virtual-chain-changed`                   |
| `TONDI_SCAN_LOW_PRIORITY_EVENTS` | Low priority events (comma-separated) | `new-block-template`                      |

### CORS Configuration

| Variable                    | Description                           | Default                                    |
| --------------------------- | ------------------------------------- | ------------------------------------------ |
| `TONDI_SCAN_CORS_ALLOWED_ORIGINS` | Allowed origins (use `*` for all)    | `*` (allow all)                            |
| `TONDI_SCAN_CORS_ALLOWED_METHODS` | Allowed HTTP methods (use `*` for all) | `*` (allow all)                            |
| `TONDI_SCAN_CORS_ALLOWED_HEADERS` | Allowed headers (use `*` for all)     | `*` (allow all)                            |
| `TONDI_SCAN_CORS_MAX_AGE`  | Preflight cache time in seconds       | `3600`                                    |

### Security Configuration

| Variable                    | Description                           | Default                                    |
| --------------------------- | ------------------------------------- | ------------------------------------------ |
| `TONDI_SCAN_RATE_LIMIT`    | Rate limit (requests per minute)      | `100`                                     |
| `TONDI_SCAN_MAX_BODY_SIZE` | Maximum request body size in bytes    | `10485760` (10MB)                          |

### Configuration File

You can also use a TOML configuration file. See `config.example.toml` for a complete example.

**Note**: Environment variables always take precedence over TOML file settings.

## Project Structure

```
tondi-listener/
├── crates/
│   ├── db/              # Database layer (Diesel ORM)
│   ├── http2-client/    # HTTP/2 client
│   ├── http2-server/    # HTTP/2 server
│   ├── http3-client/    # HTTP/3 client
│   ├── library/         # Shared library
│   ├── server/          # Main server
│   ├── wasm2-client/    # WebAssembly client
│   └── wasm3-client/    # WebAssembly components
├── protowire/           # Protocol Buffers definitions
└── src/                 # Root library
```

## Development

### Code Quality

This project enforces strict code quality tools:

* **Clippy**: pedantic mode enabled
* **Rustfmt**: formatting enforcement
* **Miri**: memory checks

### Testing

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p tondi-listener-server
```

### Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# WebAssembly build
cargo build --target wasm32-unknown-unknown
```

## Deployment

### Production Setup

1. Set environment variables

```bash
export TONDI_SCAN_ENVIRONMENT=production
export TONDI_SCAN_LOG_LEVEL=warn
```

2. Use production middleware

```rust
use tondi_listener_server::middleware::production_middleware;
let middleware = production_middleware(&config);
```

3. Enable security features

* Strict CORS policy
* Rate limiting
* Request validation
* Compression and timeout


## Event Configuration Details

### Supported Blockchain Events

The system supports the following blockchain events that can be selectively enabled:

| Event Type | Description | Default Enabled |
|------------|-------------|-----------------|
| `block-added` | New block added to blockchain | ✅ |
| `utxos-changed` | UTXO set changes | ✅ |
| `virtual-chain-changed` | Virtual chain changes | ✅ |
| `finality-conflict` | Finality conflicts | ❌ |
| `finality-conflict-resolved` | Finality conflict resolution | ❌ |
| `sink-blue-score-changed` | Sink blue score changes | ❌ |
| `virtual-daa-score-changed` | Virtual DAA score changes | ❌ |
| `pruning-point-utxo-set-override` | Pruning point UTXO set override | ❌ |
| `new-block-template` | New block template | ❌ |

### wRPC Configuration

The system now supports both gRPC and wRPC (WebSocket RPC) protocols. wRPC is particularly useful for Web environments and provides better real-time event handling.

#### wRPC Ports by Network and Encoding

| Network | Borsh Encoding | JSON Encoding |
|---------|----------------|----------------|
| Mainnet | 17110 | 18110 |
| Testnet | 17210 | 18210 |
| Devnet  | 17610 | 18610 |
| Simnet  | 17510 | 18510 |

#### wRPC Configuration Options

```toml
[server.wrpc]
# Enable wRPC (if true, wRPC will be used instead of gRPC)
enabled = false

# Protocol: "ws", "wss", "grpc"
protocol = "ws"

# Node host address
host = "8.210.45.192"

# Port (0 = use default for network/encoding)
port = 0

# Network type: "mainnet", "testnet", "devnet", "simnet"
network = "devnet"

# Encoding: "borsh", "json"
encoding = "borsh"
```

#### Environment Variables for wRPC

```bash
# Enable wRPC
export TONDI_SCAN_WRPC_ENABLED=true

# wRPC protocol
export TONDI_SCAN_WRPC_PROTOCOL=ws

# wRPC host
export TONDI_SCAN_WRPC_HOST=8.210.45.192

# wRPC port (0 = use default)
export TONDI_SCAN_WRPC_PORT=0

# wRPC network
export TONDI_SCAN_WRPC_NETWORK=devnet

# wRPC encoding
export TONDI_SCAN_WRPC_ENCODING=borsh
```

#### wRPC vs gRPC

| Feature | gRPC | wRPC |
|---------|------|------|
| Protocol | HTTP/2 | WebSocket |
| Port (Devnet) | 16610 | 17610 (Borsh) / 18610 (JSON) |
| Web Support | Limited | Full (WASM) |
| Real-time Events | Good | Excellent |
| Connection Management | Manual | Automatic |
| Reconnection | Manual | Built-in |

### Event Processing Strategies

#### Real-Time Strategy (Default)
- Process all events immediately
- Suitable for scenarios requiring instant response
- Higher resource consumption

#### Batch Strategy
- Process events in batches to reduce database writes
- Configurable batch size and timeout
- Suitable for high-throughput scenarios

#### Priority Strategy
- Process events by priority (high, medium, low)
- Important events processed first
- Suitable for resource-constrained environments

### Performance Optimization

#### Production Environment
```bash
# Enable only core events
export TONDI_SCAN_ENABLED_EVENTS="block-added,utxos-changed"

# Use batch processing
export TONDI_SCAN_EVENT_STRATEGY="batch"
export TONDI_SCAN_BATCH_SIZE=500
export TONDI_SCAN_BATCH_TIMEOUT_MS=50

# Optimize buffer
export TONDI_SCAN_BUFFER_SIZE=2000
export TONDI_SCAN_ENABLE_DEDUPLICATION=true
```

#### Development Environment
```bash
# Enable more events for testing
export TONDI_SCAN_ENABLED_EVENTS="block-added,utxos-changed,virtual-chain-changed,new-block-template"

# Use real-time processing
export TONDI_SCAN_EVENT_STRATEGY="real-time"

# Smaller buffer for development
export TONDI_SCAN_BUFFER_SIZE=500
```

### CORS Configuration Examples

#### Allow All Origins (No CORS Restrictions)
```bash
# Method 1: Set to wildcard
export TONDI_SCAN_CORS_ALLOWED_ORIGINS="*"

# Method 2: Leave empty
export TONDI_SCAN_CORS_ALLOWED_ORIGINS=""
```

#### Restrict Specific Origins
```bash
export TONDI_SCAN_CORS_ALLOWED_ORIGINS="http://localhost:3000,https://yourdomain.com"
export TONDI_SCAN_CORS_ALLOWED_METHODS="GET,POST"
export TONDI_SCAN_CORS_ALLOWED_HEADERS="Content-Type,Authorization"
```

**Note**: This is a project under development, APIs may change.