# Jag Sandbox Threat Model

## Trust Boundaries
- User → Agent: Prompts are sanitized before LLM processing
- Agent → Sandbox: Commands validated against security tier policy
- Sandbox → OS: Resource limits enforced; path traversal blocked

## Attack Vectors & Mitigations
| Vector | Mitigation |
|--------|------------|
| Prompt Injection | `jag-validation` sanitization layer |
| Path Traversal | `canonicalize()` + `starts_with(workspace_root)` |
| Resource Exhaustion | Memory/CPU/process limits + timeouts |
| Dependency Confusion | Workspace pinning + `vendored` git2/libgit2 |

## Audit Requirements
- All privileged operations logged to `audit_log` table
- Security tier changes require explicit user approval
- API keys never stored in plaintext; loaded via `.env` at runtime
