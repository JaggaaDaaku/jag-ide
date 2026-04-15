//! # Jag Sandbox — Secure Execution Environment
//! 
//! ## Security Guarantees
//! 
//! This crate provides sandboxed execution for AI agent actions:
//! 
//! ### File System
//! - All paths are canonicalized via `std::fs::canonicalize()`
//! - Operations restricted to `workspace_root` via `starts_with()` check
//! - Symlinks outside workspace are rejected
//! - Path traversal attempts (`../`, absolute paths) return `JagError::PathTraversal`
//! 
//! ### Process Execution
//! - Commands validated against allowlist/denylist per `SecurityTier`
//! - Resource limits: memory (512MB), CPU time (60s), process count (10)
//! - Timeout: 120 seconds max per command
//! - Output captured; no direct TTY access
//! 
//! ### Platform-Specific Hardening
//! - Linux: seccomp-bpf profile (planned: Phase 3)
//! - macOS: sandbox-exec profile (planned)
//! - Windows: Job Object limits (implemented)
//! 
//! ## ⚠️ Known Limitations
//! 
//! - Not a substitute for OS-level containerization
//! - Does not prevent all side-channel attacks
//! - Audit logs are append-only but not cryptographically verified (see Task AUD-001)

pub mod security;
