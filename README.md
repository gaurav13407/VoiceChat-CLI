# VoiceChat-CLI

A lightweight, private, end-to-end encrypted chat application built in Rust for low latency and small private groups.

## 🚀 Quick Start

### Build
```bash
cargo build --release
```

### Run Signaling Server
```bash
./target/release/signaling
# Listening on 127.0.0.1:9000
```

### Create a Room
```bash
./target/release/client create
# Output: Room Code: XXXX-YYYY
```

### Join a Room
```bash
./target/release/client join XXXX-YYYY
```

### Chat Commands
- `/msg <text>` - Send a message
- `/exit` - Leave the room

## 🌐 Test Online

Set the server address via environment variable:

```bash
# Start signaling server
./target/release/signaling

# Create room with custom server
SERVER_ADDR="your-server-ip:9000" ./target/release/client create

# Join room with custom server
SERVER_ADDR="your-server-ip:9000" ./target/release/client join XXXX-YYYY
```

## 🎯 Features

🔐 **End-to-End Encrypted** - ChaCha20-Poly1305 AEAD encryption  
💬 **Text Chat** - Secure encrypted messaging  
👥 **Private Rooms** - Invite code based access  
🔄 **Relay Server** - Works through NAT/firewalls  
⚡ **Low Latency** - Direct TCP connections

## 📦 Architecture

- **Signaling Server** - Coordinates peer connections and relays traffic
- **Client** - Command-line chat interface
- **VC Core** - Cryptography and protocol implementation
  - X25519 key exchange
  - Ed25519 identity keys
  - ChaCha20-Poly1305 encryption

## 🛠️ Technical Details

### Handshake Protocol
1. Client sends ephemeral X25519 public key
2. Host responds with ephemeral X25519 public key
3. Diffie-Hellman key exchange
4. Derive shared secret
5. All messages encrypted with ChaCha20-Poly1305

### Message Format
```
[2 bytes: length][encrypted payload]
```

## 🔧 Development

### Run in Debug Mode
```bash
cargo build
./target/debug/signaling &
./target/debug/client create
```

### Project Structure
```
├── signaling/    # Signaling & relay server
├── client/       # CLI client application
├── vc_core/      # Core crypto & protocol
└── audio/        # Audio capture/playback (WIP)
```

## 📝 Errors Fixed

All compilation errors resolved:
- ✅ Added missing `anyhow` dependencies
- ✅ Fixed path separator syntax (`::` instead of `:`)
- ✅ Fixed handshake role coordination (HOST/CLIENT)
- ✅ Implemented message relay in signaling server
- ✅ Fixed non-blocking I/O deadlock issues
- ✅ Added proper error handling throughout

## 🎯 Motivation

Modern chat apps are heavy and centralized. This project explores:
- Peer-to-peer architecture
- Minimal resource usage
- True end-to-end encryption
- No dependence on third-party servers

## 📄 License

MIT

🧠 Minimal CPU & RAM usage

🖥️ CLI-first (no fancy UI)

🚫 Non-Goals (Very Important)

This project is not trying to be a Discord replacement.

It intentionally does NOT include:

User accounts or login

Message history

Public servers or communities

Bots, roles, or moderation

Fancy UI or mobile apps (for now)

Keeping the scope small is by design.

🧠 High-Level Architecture
Client (CLI)
 ├─ CLI interface
 ├─ Audio pipeline (Opus)
 ├─ Encryption (E2EE)
 └─ Networking logic

Signaling Server
 └─ Room code ↔ peer introduction

Relay Server (fallback)
 └─ Encrypted packet forwarding only


Audio and chat data are never decrypted on servers

Servers act only as connectors or relays

Anyone can host a session

🔁 Connection Strategy

Connection attempts follow this order:

Direct P2P (lowest latency)

Host-as-Server

Relay fallback (guaranteed connectivity)

This makes the system robust across:

Wi-Fi ↔ Wi-Fi

Wi-Fi ↔ Hotspot

Different ISPs

NAT / CGNAT environments

📁 Project Structure

### Overview
```
VoiceChat-CLI/
├── client/      # CLI client (thin layer)
├── vc_core/     # Core logic (state machine, protocol, crypto)
├── audio/       # Audio capture, playback, codec
├── signaling/   # Signaling server
├── relay/       # Relay server (fallback)
└── Cargo.toml   # Workspace config
```

The project is library-first to keep logic testable and reusable.

### Detailed Component Breakdown

#### 🖥️ `/client` - Client Application
The CLI interface that users interact with to join voice chats.

**Files:**
- `main.rs` - Entry point, handles room creation/joining commands
- `cli.rs` - Command-line argument parsing
- `app.rs` - Main application loop and logic
- `host.rs` - Host-as-server mode implementation
- `identity.rs` - User identity management (cryptographic keys)
- `config/` - Client configuration files
- `tests/` - Integration and session tests

**Purpose:** This is what users run. It connects to the signaling server, exchanges encryption keys, establishes connections (P2P/host/relay), and manages the audio streaming pipeline.

#### 🔐 `/vc_core` - Core Library
The security and protocol foundation shared across all components.

**Structure:**
```
vc_core/
├── crypto/              # End-to-end encryption
│   ├── crypto.rs       # ChaCha20-Poly1305, X25519 key exchange
│   └── mod.rs
├── protocol/            # Communication protocols
│   ├── handshake.rs    # Secure handshake implementation
│   └── mod.rs
├── net/                 # Networking layer
│   ├── client_handshake.rs  # Client-side handshake logic
│   ├── host_handshek.rs     # Host-side handshake logic
│   ├── secure_stream.rs     # Encrypted data streaming
│   └── mod.rs
├── room/                # Room management
│   ├── code.rs         # Room code generation/validation
│   └── mod.rs
└── state/               # Connection state management
    ├── machine.rs      # State machine for connection lifecycle
    ├── secure_session.rs  # Secure session state
    └── mod.rs
```

**Purpose:** Contains all security-critical code including:
- ECDH key exchange and session key derivation
- Authenticated encryption/decryption
- Handshake protocol implementation
- Secure connection state management
- Room code logic

#### 🎤 `/audio` - Audio Processing
Low-level audio handling for voice communication.

**Files:**
- `capture.rs` - Microphone input capture
- `playback.rs` - Speaker output playback
- `device.rs` - Audio device enumeration and selection
- `codec.rs` - Opus codec encoding/decoding

**Purpose:** Manages the audio pipeline from microphone to network (encoding) and network to speakers (decoding). Handles device selection, buffer management, and real-time audio processing.

#### 📡 `/signaling` - Signaling Server
Lightweight matchmaking server for peer discovery.

**Files:**
- `main.rs` - Server entry point
- `server.rs` - TCP server handling CREATE/JOIN commands
- `room.rs` - Room state management
- `protocol.rs` - Signaling protocol definitions

**Purpose:** Acts as a rendezvous point. When users create or join rooms, this server:
- Generates and validates room codes
- Exchanges peer information (public keys, addresses)
- Facilitates initial peer discovery

**Important:** This server never handles voice or chat data - only connection setup metadata.

#### 🔄 `/relay` - Relay Server
Encrypted packet forwarding fallback for difficult network scenarios.

**Files:**
- `main.rs` - Server entry point
- `server.rs` - Core relay logic
- `forward.rs` - Packet forwarding implementation

**Purpose:** Used when direct P2P or host-as-server fails due to:
- CGNAT (Carrier-Grade NAT)
- Strict firewall rules
- Mobile hotspot restrictions
- Asymmetric routing issues

Forwards end-to-end encrypted packets **without decrypting them**. Acts as a dumb pipe for encrypted data.

#### 🔨 `/target` - Build Artifacts
Rust compiler output (auto-generated, not source code).
- `debug/` - Debug builds
- `release/` - Optimized release builds
- `deps/` - Compiled dependencies

### Component Interaction Flow

```
1. User runs CLIENT
         ↓
2. CLIENT connects to SIGNALING server
         ↓
3. SIGNALING coordinates peers (exchanges public keys & addresses)
         ↓
4. Clients use VC_CORE to perform cryptographic handshake
         ↓
5. Connection established (P2P → host-as-server → relay fallback)
         ↓
6. AUDIO crate captures voice from mic
         ↓
7. VC_CORE encrypts audio packets
         ↓
8. Encrypted packets sent over network
         ↓
9. VC_CORE decrypts received packets
         ↓
10. AUDIO crate plays voice through speakers
```

### Security Model

- **Zero-trust servers:** Signaling and relay servers never see plaintext data
- **End-to-end encryption:** All voice and chat encrypted before leaving client
- **Perfect forward secrecy:** Session keys derived via ECDH, not reusable
- **No persistent identity:** Public keys generated per-session (currently)
- **Minimal attack surface:** CLI-only, no web interface, no plugins

🛠 Tech Stack

Language: Rust

Audio Codec: Opus

Transport: UDP (voice), reliable channel for chat

Encryption: End-to-End Encryption (session-based)

Runtime: Async (later phases)

UI: CLI (intentional)

🧪 Development Philosophy

Correctness over features

Finish an MVP before expanding scope

Avoid unnecessary abstractions

Learn networking by building real systems

Accept real-world constraints (NAT, CGNAT, firewalls)


## 🚧 Project Status

This project is currently **feature-complete and closed for active development**.

The following components are fully implemented and stable:

- End-to-end encrypted peer-to-peer text chat
- Secure identity and handshake (Ed25519, X25519, HKDF, AEAD)
- Room-based signaling and peer discovery
- Local voice streaming pipeline (tested and functional)

### ⚠️ Known Limitations

Peer-to-peer encrypted voice over the public internet requires:

- Dedicated relay / TURN infrastructure, or
- Advanced NAT traversal and echo cancellation

Due to limited infrastructure resources, a public voice relay server is not hosted.

## 🤝 Contributions

Contributions are welcome specifically for:

- Improving the peer-to-peer encrypted voice pipeline
- Echo cancellation and jitter handling
- NAT traversal (STUN/TURN–like mechanisms)
- Alternative deployment strategies for voice channels

If you have experience in real-time audio or P2P networking, feel free to open an issue or submit a pull request.

## 🧠 Motivation

This project was built as a learning-focused systems and networking experiment.
The cryptographic core and messaging system are complete; further work requires infrastructure and long-term maintenance beyond the current scope.

---

⚠️ Disclaimer

This is an educational and experimental project.
It is not intended for large-scale production use.
