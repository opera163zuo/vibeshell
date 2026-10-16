# VibeShell LAN MCP on Ubuntu/Debian

Run VibeShell's MCP endpoint as a headless service on one Linux server. Agents on
any operating system connect to it over standard MCP HTTP and operate the SSH
sessions, files and saved servers that this host owns. The Agent needs no local
VibeShell, no SSH client and no copy of the credential store.

```
Hermes / any MCP client  ──HTTP + Bearer token──▶  vibeshell mcp serve  ──SSH──▶  targets
   (macOS, Windows, Linux)                          (Ubuntu/Debian host)
```

Verified on Debian 13 with an isolated loopback OpenSSH container: pairing,
`initialize`, `tools/list`, `server_add`, key-based `session_create`, `exec`,
`sftp_ls`, fail-closed approval, and immediate revocation.

## 1. Install the binary

```bash
# Build from the repository (needs the Rust stable toolchain and Tauri build deps)
cargo build --release --locked -p vshell           # binary: target/release/vibeshell
sudo install -m 0755 target/release/vibeshell /usr/local/bin/vibeshell

sudo useradd --system --create-home --home-dir /var/lib/vibeshell --shell /usr/sbin/nologin vibeshell
sudo install -d -o vibeshell -g vibeshell -m 0700 /var/lib/vibeshell /etc/vibeshell
```

## 2. Configure

`deploy/systemd/vibeshell-mcp.service` reads `/etc/vibeshell/mcp.env`:

```ini
# Loopback only, for a reverse proxy or same-host agents.
VIBESHELL_MCP_BIND=127.0.0.1:8765
```

There is no dedicated state switch: the unit sets `XDG_DATA_HOME=%d`, so the
database, device keys and session logs live under `/var/lib/vibeshell`. The same
variable is what a manual run must export.

`mcp.env` is readable by the service user only (`0600`) and must not contain
secrets — credentials live in the encrypted state directory, not in the unit.

## 3. Start and pair

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now vibeshell-mcp.service
curl -s http://127.0.0.1:8765/health

sudo -u vibeshell XDG_DATA_HOME=/var/lib/vibeshell vibeshell mcp pair-code
# One-time pairing code: 447772  (valid 300 s, usable once, never logged)
```

The Agent exchanges the code for its own bearer token. The token is returned
exactly once and only its SHA-256 hash is stored:

```bash
curl -s -X POST http://127.0.0.1:8765/pair \
  -H 'Content-Type: application/json' \
  -d '{"name":"hermes-macos","code":"447772"}'
# {"agentId":"8045…","name":"hermes-macos","token":"vsh_…","tokenType":"Bearer"}
```

Point the client at `http://127.0.0.1:8765/mcp` with an
`Authorization: Bearer <token>` header. Every MCP route requires that header;
only `/health` and `/pair` are reachable without it.

```bash
sudo -u vibeshell XDG_DATA_HOME=/var/lib/vibeshell vibeshell mcp agents
sudo -u vibeshell XDG_DATA_HOME=/var/lib/vibeshell vibeshell mcp revoke 8045
sudo -u vibeshell XDG_DATA_HOME=/var/lib/vibeshell vibeshell mcp rotate 8045
```

Revocation and rotation take effect on the Agent's next request — the service
re-reads the registry from storage, it does not rely on a restart.

## 4. Let the Agent reach a real host

Three steps, each on the side that owns the decision:

1. **Server profile** — the Agent can create it with the `server_add` tool, or an
   operator saves it in the desktop app. Profiles hold host, port, username and
   auth type, never a secret.
2. **Credential** — the administrator pipes it into stdin. It is encrypted with
   the device key and is never accepted as a command-line argument.

   ```bash
   sudo -u vibeshell XDG_DATA_HOME=/var/lib/vibeshell vibeshell mcp credential set prod-web \
     --auth-type password < /root/prod-web-password
   # key auth: paste the PEM, or reference a file on this host
   vibeshell mcp credential set prod-web --auth-type key < ~/.ssh/id_ed25519
   vibeshell mcp credential set prod-web --auth-type key --key-path /var/lib/vibeshell/.ssh/id_ed25519
   ```
3. **Host key** — VibeShell refuses to send credentials to an untrusted key. On a
   headless host there is no dialog, so verify the fingerprint out of band and
   pin it by repeating it:

   ```bash
   vibeshell mcp hostkey probe prod.example --port 22
   # prod.example:22 presents ssh-ed25519 SHA256:UCDhSRh5…  — not trusted yet
   vibeshell mcp hostkey trust prod.example --port 22 --fingerprint SHA256:UCDhSRh5…
   ```

   `trust` re-probes and refuses if the presented key differs from the fingerprint
   you typed. A changed key is reported as `HOST_KEY_CHANGED` and never pinned.

Then the Agent calls `session_create` and the SSH/SFTP tools as usual.

## 5. Reaching beyond loopback

Clear text HTTP over a network is not acceptable, and the service does not do TLS
itself. Put a TLS-terminating proxy in front and bind the service to loopback,
or bind a specific private address deliberately:

```ini
VIBESHELL_MCP_BIND=192.168.1.20:8765
VIBESHELL_MCP_ALLOW_LAN=true
```

Both conditions are enforced at start-up, and pairing must already have happened:

- loopback binds are always allowed;
- `0.0.0.0` / `::` is refused — name one interface address;
- a non-loopback address without `VIBESHELL_MCP_ALLOW_LAN=true` is refused;
- a non-loopback address with zero active paired Agents is refused.

Restrict who may connect, for example:

```bash
sudo apt-get install -y nftables
sudo nft add-table ip vibeshell
sudo nft add-chain ip vibeshell input tcp dport 8765 ip saddr 192.168.1.0/24 accept
```

Caddy terminating TLS for the Agent side:

```
mcp.example.internal {
  reverse_proxy 127.0.0.1:8765
}
```

## 6. Backups

`/var/lib/vibeshell` (the `XDG_DATA_HOME` root the unit assigns) holds
`com.vibeshell.desktop/vibeshell.db`, its `ssh_credentials.key`/`.salt` device
key files, `ssh_fingerprints.json` and session recordings. Back the whole
directory up as one secret; the encrypted rows are unreadable without the device
key, and the device key is useless without the database. Keep it `0700`, offline.

```bash
sudo systemctl stop vibeshell-mcp.service
sudo tar --mode='go-rwx' -czf /root/vibeshell-state.tar.gz -C /var/lib vibeshell
sudo systemctl start vibeshell-mcp.service
```

## 7. What this service does not do

- **No self-approval.** Risky commands are gated by the human approval UI, which
  a headless process does not have. Such a call fails closed with
  `Command requires user approval, but no approval UI is available.` Read-only
  and non-flagged operations run normally.
- **No shared isolation between Agents.** They share one database, one session
  manager and one set of saved servers. Two Agents are two audit identities, not
  two tenants. Do not pair an Agent you intend to keep apart from the others.
- **No TLS in the process.** Use a proxy or a private network, as above.
- **No browser-origin checks.** The endpoint is for native MCP clients; requests
  carrying an `Origin` header are not validated, so do not expose it to browsers.
- **No per-Agent server allowlists yet**; scoping is the common permission profile.
