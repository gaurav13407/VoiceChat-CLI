# VoiceChat-CLI

A lightweight, private, end-to-end encrypted voice + chat application built in Rust, designed for low latency, low resource usage, and small private groups.

This project is intentionally CLI-based and minimal.
The goal is correctness, performance, and learning, not UI polish.

🎯 Motivation

Modern voice apps are heavy, centralized, and resource-hungry.
For gaming and private communication, this often means:

High RAM/CPU usage

Unnecessary latency

No true end-to-end privacy

Dependence on always-on third-party servers

This project explores a different approach:

Peer-to-peer first

Host-based when possible

Relay only when necessary

End-to-end encrypted by default

The result is a tool that works well for friends who just want to talk while playing games, without slowing their system down.

✨ Key Features (MVP)

🔐 End-to-End Encrypted voice

💬 Encrypted text chat

👥 Small private rooms (invite code based)

🔄 Hybrid networking:

Direct P2P when possible

Host-as-server fallback

Relay fallback for strict networks (CGNAT, mobile hotspot)

⚡ Low latency (UDP-based)

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

🚧 Project Status

Active development (MVP phase)

Current focus:

CLI commands

Room code generation

Signaling logic

Secure connection setup



⚠️ Disclaimer

This is an educational and experimental project.
It is not intended for large-scale production use.
