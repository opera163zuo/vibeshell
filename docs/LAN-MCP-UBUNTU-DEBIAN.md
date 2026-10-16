# VibeShell LAN MCP (Ubuntu/Debian)

This optional service is intended to run VibeShell on a Linux host. Hermes
Agents on macOS, Windows, Linux, and other systems will connect over MCP HTTP
once the network transport is implemented.

> **Not production-ready.** The current `vibeshell mcp-serve` command checks
> configuration and then exits with an explicit message that the HTTP listener
> is not wired. Do not expose the service or open firewall ports yet.

## Intended deployment

- Run under a dedicated `vibeshell` account; do not run as root.
- Keep service data under a directory owned by that account, mode `0700`.
- Default bind should be loopback. For LAN access, bind only a private host
  interface, constrain access with the firewall, and use TLS at a trusted proxy.
- Pair each Agent independently; revoke lost credentials promptly.
- Device-key files used for saved SSH credentials live in VibeShell's per-user
  data directory. Back them up securely; do not share across unrelated hosts.
- Keep SSH host-key verification enabled and inspect first-use fingerprints.

## Draft systemd unit

`deploy/systemd/vibeshell-mcp.service` is a draft unit. It assumes the CLI at
`/usr/local/bin/vibeshell` and an environment file at `/etc/vibeshell/mcp.env`.
The current service data path uses the OS default per-user directory; the unit's
`ProtectHome=read-only` requires a configurable state directory before the unit
can be enabled as written.

Only after real authenticated HTTP serving and Agent pairing endpoints exist:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now vibeshell-mcp.service
sudo systemctl status vibeshell-mcp.service
```

Do not enable the unit or open inbound firewall access until that implementation
is complete and security-tested. Risky remote commands must fail closed when no
human approval UI is available.
