# VibeShell MCP Service CLI

The `vibeshell` binary runs and administers the Linux MCP service. SSH site management and manual login are provided by the desktop UI; AI Agents connect through MCP HTTP and never receive saved credentials. VibeShell does not provide macOS or Windows desktop/CLI distributions.

## Install

```bash
./install.sh
```

The default destination is `~/.local/bin/vibeshell`. Override it with `VIBESHELL_INSTALL_DIR=/usr/local/bin ./install.sh` when appropriate.

## Run the MCP service

```bash
vibeshell mcp serve
```

The default listener is loopback at `127.0.0.1:8765`. For a same-user desktop deployment, install `deploy/systemd/vibeshell-mcp.user.service`; see `docs/LAN-MCP-UBUNTU-DEBIAN.md`. LAN binding requires an explicit private interface address, `VIBESHELL_MCP_ALLOW_LAN=true`, and an already paired Agent.

Pair and administer Agents with:

```bash
vibeshell mcp pair-code
vibeshell mcp agents
vibeshell mcp revoke <agent-id>
vibeshell mcp rotate <agent-id>
```

SSH passwords and private keys are saved through the desktop UI or provisioned by piping them to `vibeshell mcp credential set`. They are encrypted at rest and consumed by the Rust service; the MCP protocol never returns them. Sudo credentials and exact command allowlists are configured per site in the UI.

## Build from source

```bash
cargo build --release --locked --package vshell --bin vibeshell
```

To prepare the Linux target-suffixed binary consumed by Tauri desktop packaging:

```bash
node scripts/prepare-sidecar.mjs
```

VibeShell 1.1.0 is GPL-3.0-only. Release archives include LICENSE and NOTICE. The software comes without warranty to the extent permitted by law.
