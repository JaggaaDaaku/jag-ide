# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-04-22
### Added
- **Multi-Agent System**: Complete autonomous development loop with Planner, Backend, Frontend, Integration, and Git agents.
- **Mission Control**: Unified dashboard for real-time mission monitoring and agent coordination.
- **Security Sandbox**: Tiered execution environment with path traversal protection and command denylisting.
- **Artifact Traceability**: Automatic generation and tracking of PRDs, architecture diagrams, and code artifacts.
- **E2E Testing**: Integrated Playwright suite for automated frontend verification.
- **CLI Validation**: Added `--validate-config` flag to server for quick environment checks.
- **Build Pipeline**: Robust PowerShell build scripts with checksum generation and dry-run support.

### Changed
- Migrated all crates to version 1.0.0 for stable release.
- Enhanced README with production status and system requirements.
- Hardened CI workflow to include full workspace testing and frontend verification.

### Fixed
- Resolved 26 compiler warnings and clippy lints.
- Fixed Windows path canonicalization bug in workspace manager.
- Corrected TypeScript type errors in coverage reporting components.
- Fixed unused imports in test modules causing CI failures.
