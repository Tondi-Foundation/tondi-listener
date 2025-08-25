
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
cd tondi-scan
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
cargo run -p tondi-scan-server --bin server

# Start HTTP router
cargo run -p tondi-scan-server --bin router
```

## Configuration

### Environment Variables

| Variable             | Description                  | Default                                           |
| -------------------- | ---------------------------- | ------------------------------------------------- |
| `TONDI_SCAN_HOST_URL`     | Server listening address     | `127.0.0.1:3000`                                  |
| `TONDI_SCAN_GRPC_URL`     | gRPC service address         | `grpc://8.210.45.192:16610`                       |
| `TONDI_SCAN_DATABASE_URL` | PostgreSQL connection string | `postgres://postgres:postgres@127.0.0.1/postgres` |
| `TONDI_SCAN_ENVIRONMENT`  | Runtime environment          | `development`                                     |
| `TONDI_SCAN_LOG_LEVEL`    | Log level                    | `info`                                            |

### Security Settings

* **Rate limit**: 100 requests per minute by default
* **Timeout**: 15 seconds by default
* **Request body size**: 10MB by default
* **CORS**: Configurable cross-origin policies

## Project Structure

```
tondi-scan/
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
cargo test -p tondi-scan-server
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
use tondi_scan_server::middleware::production_middleware;
let middleware = production_middleware(&config);
```

3. Enable security features

* Strict CORS policy
* Rate limiting
* Request validation
* Compression and timeout


**Note**: This is a project under development, APIs may change.