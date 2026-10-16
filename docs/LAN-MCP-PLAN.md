# VibeShell LAN MCP — project plan

## Goal

Build a Linux-hosted VibeShell MCP service, targeting Ubuntu and Debian servers, so Hermes Agents on different operating systems can connect over the local network and operate SSH sessions managed by that VibeShell instance.

The Linux server owns SSH connections, saved server profiles, host-key verification, SFTP, and the MCP endpoint. Remote Agents use the standard MCP HTTP transport and do not need to run VibeShell or SSH locally.

## Current status

This is a work in progress on branch `feature/lan-mcp-gateway` in the `opera163zuo/vibeshell` fork. A draft PR is open: https://github.com/opera163zuo/vibeshell/pull/1

Implemented scaffolding includes an expiring one-time pairing code, random per-Agent bearer tokens stored as hashes, revocation metadata, device-key encryption for the persisted registry, a CLI `mcp-serve` entry point, listener policy checks, and a draft Ubuntu/Debian systemd unit.

**The service is not usable yet.** HTTP MCP and pairing route integration have not passed end-to-end verification. The latest Gateway/router wiring changes timed out during build/test attempts. Do not install the draft unit, expose a port, or treat current pairing scaffolding as a security-reviewed service.

## Functional requirements

1. Run headlessly on supported Ubuntu/Debian server releases, with no GUI, display server, or Node.js runtime required after installation.
2. Provide a standard Streamable HTTP MCP endpoint addressable by Hermes Agents on macOS, Windows, Linux, and other systems with HTTP MCP support.
3. Pair Agents through an explicit, short-lived one-time code. Return a cryptographically random independent token only after successful pairing.
4. Store only token hashes; encrypt persistent pairing records at rest; never put tokens in URLs or logs.
5. Let an administrator list paired Agents and revoke or rotate each Agent credential independently.
6. Bind to loopback by default. LAN binding must be explicit, require at least one valid Agent credential, and support a specific private interface address rather than silently exposing all interfaces.
7. Preserve audit records and existing command-approval checks. If a risky operation needs human approval and no approval UI exists, fail closed.
8. Support a dedicated system user, protected persistent state, systemd startup/restart, graceful shutdown, and documented backup/restore.

## Security requirements

- Use bearer authorization headers, constant-time token-hash comparison, request/body/output limits, and rate limiting for pairing attempts.
- Enforce HTTPS/TLS in the service or require a documented trusted TLS proxy for LAN traffic; reject accidental cleartext exposure where feasible.
- Protect the initial pairing code: show it only to the local administrator on the server console, expire it quickly, limit attempts, and permit one successful use.
- Require host-key verification for SSH and preserve current credential secrecy boundaries.
- Scope each Agent's access to explicit permissions and, where practical, server/session allowlists. Initial version may start with a clearly documented common permission profile, but must not imply identities are isolated if they share the same database/session manager.
- Keep secrets out of process arguments, environment dumps, logs, and Git.
- Systemd service must run unprivileged with a private state directory and least-privilege filesystem/network settings. Do not ship a unit whose hardening blocks the configured state directory.

## Implementation phases

### Phase 0 — Stabilize the branch

- Build and test the currently pushed commit from a clean checkout.
- Fix malformed wiring or revert unsafe/partial route changes.
- Keep an explicit incomplete status until all acceptance tests pass.

### Phase 1 — MCP transport

- Wire `mcp-serve` to a real Axum HTTP server using the existing MCP JSON-RPC handler.
- Support `initialize`, `notifications/initialized`, `ping`, `tools/list`, and `tools/call` over the supported MCP HTTP transport.
- Add integration tests that start the listener on an ephemeral loopback port and exercise those methods.

### Phase 2 — Pairing and authentication

- Add a separate pairing endpoint, unauthenticated only while presenting a valid one-time code.
- Make pairing atomically consume the code and issue a token once.
- Protect every MCP endpoint with per-Agent bearer token authentication; leave only health and pairing endpoints public.
- Test valid token, missing token, invalid token, revoked token, expired code, repeated code, attempt-limit, and persisted restart behavior.
- Never print a bearer token. Display the pairing code locally; redact it from logs after expiry.

### Phase 3 — Linux service lifecycle

- Add configuration for bind address, port, state/database path, logs, and TLS/proxy mode.
- Add a systemd unit with a dedicated user, explicit writable state path, restart policy, restrictive umask, and safe shutdown.
- Document install, configuration, firewall, TLS, pairing, health-check, upgrade, and rollback on Ubuntu/Debian.
- Verify service lifecycle on real Ubuntu and Debian environments (or CI VMs), not just macOS builds.

### Phase 4 — Agent management and audit

- Add admin-only local CLI commands to list/revoke/rename paired Agents and rotate credentials.
- Ensure encrypted at-rest records survive restarts and backup/restore, and fail closed on decryption/corruption instead of silently resetting the registry.
- Record Agent identity with each MCP action in the audit trail.

### Phase 5 — Cross-platform Hermes verification

- Configure a Hermes Agent on Linux, macOS, and Windows to use the same remote MCP URL with separate Agent tokens.
- Verify `tools/list`, harmless SSH inspection, SFTP read, and an explicitly approved mutating command.
- Confirm revoked credentials stop working immediately and one Agent's token cannot be used to impersonate another identity.

## Acceptance criteria

- A clean Ubuntu/Debian host can install and run the service as an unprivileged systemd user.
- A local admin obtains a short-lived pairing code; a remote Hermes Agent pairs from another OS and receives one token exactly once.
- That Agent successfully initializes MCP, lists tools, and performs an authorized test operation through the LAN endpoint.
- Unauthenticated, malformed, expired, over-limit, or revoked requests are rejected; LAN mode never starts without the intended guardrails.
- Pairing records persist encrypted across service restart; corrupt/unreadable records cause an explicit safe failure.
- Logs and command history contain no pairing code or bearer token.
- Integration tests and platform build/test matrix pass; Ubuntu/Debian install and service run are verified before calling the feature complete.

## Out of scope for this phase

- Removing macOS or Windows desktop releases from upstream VibeShell.
- Public Internet exposure or multi-tenant hosting.
- Replacing SSH authentication or bypassing human approval for risky operations.
- Claiming production readiness based solely on compilation or local unit tests.
