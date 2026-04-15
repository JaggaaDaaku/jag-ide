# Jag IDE

> Agent-first autonomous development platform

## Status
- [x] Phase 1: Foundation (Tasks 1.1.1–1.2.2)
- [ ] Phase 2: Multi-Agent System (Weeks 13-24)

## Quick Start

### Prerequisites
- Rust 1.75+ (`rustup install stable`)
- Node.js 18+ (`nvm install 18`)
- Ollama (optional, for local LLM): https://ollama.ai

### Build
```bash
# Check all Rust crates
cargo check --workspace

# Run core tests
cargo test -p jag-core

# Build frontend
cd frontend && npm install && npm run build
```

### Configuration
1. Copy `.env.example` to `.env`
2. Add at least one LLM API key or ensure Ollama is running
3. Run the application

## Architecture
- 12 Rust crates in workspace (see `Cargo.toml`)
- Electron 28 + React 18 + TypeScript 5.3 frontend
- SQLite + Redis persistence (local-only; PostgreSQL planned for Phase 3)
- Ollama + cloud LLM integration

## Security
- See [`crates/jag-sandbox/SECURITY.md`](crates/jag-sandbox/SECURITY.md) for threat model
- API keys managed via `.env` (never commit real values)
- Sandbox enforces security tiers: Off / Auto / Turbo

## Contributing
1. Run `cargo clippy -- -D warnings` before PR
2. Add tests for new functionality
3. Update this README if adding new crates

## License
MIT
