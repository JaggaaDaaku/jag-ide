# Jag IDE 🚀

[![CI](https://github.com/JaggaaDaaku/jag-ide/actions/workflows/ci.yml/badge.svg)](https://github.com/JaggaaDaaku/jag-ide/actions/workflows/ci.yml)
[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)](https://github.com/JaggaaDaaku/jag-ide/releases)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

> **Agent-first autonomous development platform.** Build complex software by simply describing it.

Jag IDE is a command center for autonomous software creation. It orchestrates a team of specialized AI agents (Planner, Backend, Frontend, Integration, Git) to turn your ideas into production-ready code within a secure, sandboxed environment.

---

## ✨ Key Features

- 🤖 **Multi-Agent Orchestration**: Specialized agents work in parallel to solve complex tasks.
- 🛡️ **Secure Sandbox**: Tiered execution environment protecting your host system.
- 📊 **Mission Control**: Real-time dashboard for monitoring agent activity and system health.
- 🔗 **Artifact Traceability**: Automatic generation and tracking of PRDs, architecture, and code.
- 🌳 **Model Garden**: Seamless integration with local Ollama models and cloud LLMs (Claude, GPT, Gemini).
- ✅ **Automated Verification**: Integrated testing and coverage gating for guaranteed quality.

---

## 💻 System Requirements

- **Windows 10+** / **macOS 12+** / **Ubuntu 22.04+**
- **Ollama 0.1+** (Highly recommended for local LLM inference)
- **Rust 1.75+** (Required only for building from source)
- **Node.js 18+** (Required only for building from source)

> [!TIP]
> For the best out-of-the-box experience with local models, run:
> `ollama pull qwen2.5:7b`

---

## 🚀 Getting Started

### Installation
1. Download the latest installer for your platform from the [Releases](https://github.com/JaggaaDaaku/jag-ide/releases) page.
2. Run the installer and follow the prompts.
3. Launch Jag IDE and start your first mission!

### Running from Source
```bash
# 1. Clone the repository
git clone https://github.com/JaggaaDaaku/jag-ide.git
cd jag-ide

# 2. Install dependencies & build
scripts/build-release.ps1

# 3. Launch
cd frontend && npm start
```

---

## 🛠️ Architecture

Jag IDE is built with a high-performance Rust backend and a modern React/Electron frontend.
- **Backend**: 14 specialized Rust crates managing agents, workflow, sandbox, and persistence.
- **Frontend**: React 18 + TypeScript + Recharts for a data-rich experience.
- **Database**: Local SQLite + Redis for zero-dependency persistence.

---

## 🔐 Security & Privacy

- **Sandboxing**: All agent commands run in a hardened sandbox with configurable security tiers (Off, Auto, Turbo).
- **Privacy**: Local inference via Ollama ensures your code never leaves your machine unless you choose to use cloud providers.
- **Audit Logs**: Every action taken by an agent is signed and logged for accountability.

---

## 🤝 Contributing

We welcome contributions! Please see our [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to get involved.

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
