# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.4.x   | ✅ Current |
| 0.3.x   | ⚠️ Deprecated — upgrade recommended |
| < 0.3   | ❌ Not supported |

## Reporting a Vulnerability

If you discover a security vulnerability in wiki-mem, please report it responsibly:

1. **Do NOT open a public GitHub issue.**
2. Email the maintainer at: **security@something-cabinet.dev**
3. Or use [GitHub's private vulnerability reporting](https://github.com/something-cabinet/wiki-mem/security/advisories/new).

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

## Response Timeline

- **Acknowledgment:** within 48 hours
- **Initial assessment:** within 7 days
- **Fix and disclosure:** coordinated with the reporter, typically within 30 days

## Scope

The following are in scope:
- `wm-cli` and `wm-server` binaries
- The MCP tool surface (`wm mcp`)
- The HTTP API (`wm web`)
- ONNX model download and verification
- Wiki page parsing and filesystem operations

Out of scope:
- The Angular frontend in isolation (no server-side rendering)
- Third-party dependencies (report upstream, but notify us if exploitation is possible through wm)

## Security Design

As of v0.4.0:
- The HTTP API requires a per-launch token and rejects cross-origin requests
- All filesystem operations from request input pass through a confinement chokepoint
- Model downloads verify SHA-256 against pinned hashes
- The web UI is read-only; mutations require CLI or MCP access
- CI requires manual approval before publishing to npm
