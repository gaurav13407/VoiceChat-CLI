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
VoiceChat-CLI/
├── client/      # CLI client (thin layer)
├── vc_core/     # Core logic (state machine, protocol, crypto)
├── audio/       # Audio capture, playback, codec
├── signaling/   # Signaling server
├── relay/       # Relay server (fallback)
└── Cargo.toml   # Workspace config


The project is library-first to keep logic testable and reusable.

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
