# NRelay

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.1.0-green.svg)](Cargo.toml)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

A super fast reverse tunnel system written in Rust that allows you to expose local services to the internet through secure tunnels. Similar to ngrok or Cloudflare Tunnel, but self-hosted.

## What is NRelay?

Do you want to show your local web server to a friend? Or open up a local Minecraft server to play with others? Maybe test webhooks without deploying?

NRelay is a reverse proxy tunneling solution that consists of a relay server (running on a public server) and clients (running locally). It allows you to expose local development servers, APIs, or any network service to the internet without port forwarding or firewall configuration.

### Key Features

- **Multiple Protocol Support**: HTTP, HTTPS (TLS/SNI), TCP, UDP, Minecraft, and SSH - all in one tool
- **Flexible Routing**: Hostname-based routing for HTTP/HTTPS, port-based for TCP/UDP
- **Self-Hosted**: Run it on your own infrastructure with complete control
- **High Performance**: Built with Rust and Tokio for blazing fast async I/O
- **Token-Based Security**: Each tunnel gets its own unique authentication token
- **Origin Management**: Easily organize and manage multiple relay server configurations
- **Admin API**: RESTful API for programmatic tunnel management
- **Protocol Sniffing**: Automatically routes traffic based on intelligent protocol detection

### Supported Tunnel Types

| Protocol | Exposure Mode | Default Port | Routing Method |
|----------|---------------|--------------|----------------|
| HTTP | Hostname | 80 | Host header |
| HTTPS/TLS | Hostname | 443 | SNI (Server Name Indication) |
| TCP Raw | Port | 20000-30000 | Direct port mapping |
| UDP Raw | Port | 30000-40000 | Direct port mapping |
| Minecraft | Port | 25565 | Handshake parsing |
| SSH | Port | 20000-30000 | Direct port mapping |

## Requirements

### Server Requirements
- **Rust**: 1.70 or higher
- **Operating System**: Linux, macOS, or Windows
- **Public IP Address**: Required for the relay server
- **Open Ports**:
  - Control port (default: 7000)
  - Admin API port (default: 7001)
  - Protocol-specific ports (80 for HTTP, 443 for HTTPS, etc.)

### Client Requirements
- **Rust**: 1.70 or higher
- **Network Access**: Ability to connect to the relay server's control port

## Installation

### Prerequisites

Make sure you have Rust installed. If not, install it from [rustup.rs](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Building from Source

Clone the repository and build all components:

```bash
git clone https://github.com/sammwyy/NRelay.git
cd NRelay

# Build all workspace members
cargo build --release

# Binaries will be available in target/release/
# - nrelay (CLI tool)
# - nrelay_server (Relay server)
# - nrelay_client (Tunnel client)
```

### Installing Binaries

```bash
# Install all binaries to ~/.cargo/bin/
cargo install --path nrelay
cargo install --path nrelay_server
cargo install --path nrelay_client
```

> **💡 Pro Tip**: Make sure `~/.cargo/bin` is in your PATH to run the binaries from anywhere!

## Usage

### 1. Running the Relay Server

On your public server, start the relay server:

```bash
# Set admin API token
export ADMIN_TOKEN="your-secret-admin-token"

# Run the server
nrelay_server \
  --control-port 7000 \
  --admin-port 7001 \
  --domain yourdomain.com
```

**Server Options:**
- `--control-port`: Port for client control connections (default: 7000)
- `--admin-port`: Port for admin API (default: 7001)
- `--admin-iface`: Interface to bind admin API (default: 0.0.0.0)
- `--domain`: Base domain for hostname-based tunnels
- `--admin-token`: Bearer token for admin API authentication (can use env var)

### 2. Configuring an Origin (Client)

Add your relay server as an "origin":

```bash
nrelay origin add myserver \
  --server relay.yourdomain.com:7000 \
  --admin-url http://relay.yourdomain.com:7001 \
  --token your-secret-admin-token \
  --kind server
```

**Origin Modes:**
- `--kind server` (self-hosted): CLI acts as ADMIN, directly connects to relay server. Use this for personal/self-hosted deployments.
- `--kind service` (SaaS mode, default): CLI acts as USER, requests permissions from a backend service. Used for team/business deployments with dashboard authentication. *(Note: SaaS backend not yet implemented)*

> **💡 Pro Tip**: For self-hosted deployments, always use `--kind server` to avoid permission errors!

List configured origins:

```bash
nrelay origin list
```

Set default origin:

```bash
nrelay origin set-default myserver
```

> **💡 Pro Tip**: Once you set a default origin, you won't need to specify `--origin` in your tunnel commands!

### 3. Creating Tunnels

#### HTTP Tunnel

Expose a local HTTP server:

```bash
# Expose localhost:8080 via HTTP
nrelay http localhost:8080

# Output: Your tunnel is available at http://{tunnel-id}.yourdomain.com
```

#### HTTPS Tunnel

Expose a local HTTPS server:

```bash
nrelay https localhost:8443
```

#### TCP Tunnel

Expose a local TCP service:

```bash
# Expose local SSH server
nrelay tcp localhost:22

# Output: Your tunnel is available at relay.yourdomain.com:25432
```

#### Minecraft Server

Expose a Minecraft server:

```bash
nrelay minecraft localhost:25565
```

> **💡 Pro Tip**: The Minecraft tunnel runs on port 25565 by default, so your friends can connect directly without specifying a port!

#### Custom Options

```bash
# Specify origin
nrelay http localhost:3000 --origin myserver

# With custom configuration
nrelay tcp localhost:5432 --origin production-relay
```

### 4. Managing Origins

```bash
# List all origins
nrelay origin list

# Remove an origin
nrelay origin remove myserver

# Show origin details
nrelay origin show myserver
```

## Architecture

```
┌─────────────────────────────────────────┐
│     Public Internet Traffic             │
│  (HTTP/HTTPS/TCP/UDP/Minecraft/SSH)     │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌────────────────────────────────────────┐
│         NRelay Server (Public)         │
│                                        │
│  ┌─────────────────────────────────┐   │
│  │  Protocol Listeners             │   │
│  │  - HTTP:80                      │   │
│  │  - HTTPS:443                    │   │
│  │  - TCP: 20000-30000             │   │
│  │  - UDP: 30000-40000             │   │
│  │  - Minecraft: 25565             │   │
│  └─────────────────────────────────┘   │
│                                        │
│  ┌─────────────────────────────────┐   │
│  │  Tunnel Registry                │   │
│  │  (In-memory state)              │   │
│  └─────────────────────────────────┘   │
│                                        │
│  ┌─────────────────────────────────┐   │
│  │  Admin API (:7001)              │   │
│  │  - POST /tunnels                │   │
│  │  - Bearer token auth            │   │
│  └─────────────────────────────────┘   │
│                                        │
│  ┌─────────────────────────────────┐   │
│  │  Control Port (:7000)           │   │
│  │  - Client connections           │   │
│  │  - Protobuf protocol            │   │
│  └─────────────────────────────────┘   │
└─────────────────┬──────────────────────┘
                  │ Control Protocol
                  │ (Protobuf over TCP)
                  ▼
┌────────────────────────────────────────┐
│      NRelay Client (Local)             │
│                                        │
│  ┌─────────────────────────────────┐   │
│  │  Control Connection             │   │
│  │  - Receives tunnel requests     │   │
│  │  - Spawns tunnel handlers       │   │
│  └─────────────────────────────────┘   │
│                                        │
│  ┌─────────────────────────────────┐   │
│  │  Tunnel Handlers                │   │
│  │  - Bidirectional proxy          │   │
│  │  - Per-connection spawning      │   │
│  └─────────────────────────────────┘   │
└─────────────────┬──────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│       Local Service                     │
│   (localhost:8080, :3000, etc.)         │
└─────────────────────────────────────────┘
```

### How It Works

1. **Tunnel Creation**: Client calls admin API to create tunnel, receives unique token
2. **Control Connection**: Client establishes persistent connection to relay server
3. **Incoming Traffic**: External request arrives at relay server
4. **Protocol Sniffing**: Server extracts routing info (Host header, SNI, etc.)
5. **Tunnel Matching**: Server identifies which tunnel should handle the request
6. **Connection Request**: Server sends `OpenTunnelRequest` to client via control connection
7. **Tunnel Connection**: Client spawns handler, connects back to server with tunnel token
8. **Bidirectional Proxy**: Data flows: External ↔ Server ↔ Client ↔ Local Service

## Project Structure

```
NRelay/
├── nrelay/              # CLI tool for tunnel management
├── nrelay_server/       # Relay server (gateway)
├── nrelay_client/       # Tunnel client
├── nrelay_core/         # Shared types and protocol definitions
├── nrelay_proto_http/   # HTTP protocol sniffer
├── nrelay_proto_sni/    # TLS/SNI sniffer
├── nrelay_proto_tcp/    # TCP proxy handler
├── nrelay_proto_udp/    # UDP proxy handler
└── nrelay_proto_mc/     # Minecraft protocol handler
```

## Configuration

### Origin Configuration

Origins are stored in `~/.nrelay/origins.toml`:

```toml
[[origins]]
name = "myserver"
server = "relay.example.com:7000"
admin_url = "http://relay.example.com:7001"
admin_token = "your-admin-token"
kind = "server"  # "server" for self-hosted, "service" for SaaS (not yet implemented)
default = true
```

### Environment Variables

**Server:**
- `ADMIN_TOKEN`: Admin API bearer token

**Client:**
- `RUST_LOG`: Logging level (e.g., `info`, `debug`, `trace`)

## Security

- **Admin API**: Protected by bearer token authentication
- **Tunnel Access**: Each tunnel has a unique access token
- **Dual Auth Modes**: Separate authentication for control and data connections
- **Message Size Limits**: 64KB maximum for control messages to prevent DoS

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

**Sammwy**

## Acknowledgments

Built with:
- [Tokio](https://tokio.rs/) - Async runtime
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [Prost](https://github.com/tokio-rs/prost) - Protocol Buffers
- [Clap](https://github.com/clap-rs/clap) - CLI parsing

---

(｡♥‿♥｡) Happy Relaying 💖
