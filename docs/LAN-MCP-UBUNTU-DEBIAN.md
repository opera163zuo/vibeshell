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
# One-time pairing code is printed locally; valid for five minutes and one use.
```

The Agent exchanges the code for its own bearer token. The token is returned once; only its hash is stored. Pair from a trusted private network and configure HTTPS at a trusted reverse proxy before allowing remote traffic.

Point the Agent at `http://127.0.0.1:8765/mcp` behind the local TLS proxy, using the issued token in the `Authorization: Bearer …` header. Every MCP route requires that header; only `/health` and `/pair` are unauthenticated.

```bash
sudo -u vibeshell XDG_DATA_HOME=/var/lib/vibeshell vibeshell mcp agents
sudo -u vibeshell XDG_DATA_HOME=/var/lib/vibeshell vibeshell mcp revoke <agent-id>
sudo -u vibeshell XDG_DATA_HOME=/var/lib/vibeshell vibeshell mcp rotate <agent-id>
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

## 5. Verify before exposing it

`scripts/test-lan-mcp-gateway.sh` re-checks the gateway against a throwaway state
directory under `target/`. It never reads your saved servers, credentials or
paired Agents, and it contacts no SSH host.

```bash
bash scripts/test-lan-mcp-gateway.sh                  # free port, detected IPv4
bash scripts/test-lan-mcp-gateway.sh 192.168.1.20     # name the interface address
```

It asserts 18 checks in four groups:

1. **Boot guard** — a non-loopback bind with no paired Agent must refuse to start.
2. **Transport and auth** — health, `401` with no token and with a forged one,
   one-time code exchange plus a rejected replay, `initialize`, `202` with an empty
   body for notifications, `tools/list`, a real `tools/call`, `-32601` for an
   unknown method, `413` at the body limit, and a service log containing neither
   the pairing code nor the issued token.
3. **Audit at rest** — every `agent_activity` row written by the run carries the
   `enc:v1:` prefix.
4. **LAN and revocation** — the listener opens on the named address only because an
   Agent is already paired, a second Agent pairs over that address, and a revoked
   token is rejected on its very next request without restarting the service.

Re-run it after an upgrade or a unit change; failing here is cheaper than failing
on a live Agent. The address argument is positional, and the LAN group is skipped
with a notice when the host has no global IPv4 address.

## 6. Reaching beyond loopback

The service binds to the private LAN address configured in `/etc/vibeshell/mcp.env`; the reverse proxy runs on the same host and terminates HTTPS before forwarding to `127.0.0.1:8765`. Do not bind the MCP service itself to an externally reachable cleartext port.

Four rules are enforced at start-up (`validate_listener`), so a default or a typo can never expose the port:

- a loopback bind is always allowed;
- any other address needs `VIBESHELL_MCP_ALLOW_LAN=true` in the environment file;
- `0.0.0.0` and `::` are refused — name one interface address, e.g. `VIBESHELL_MCP_BIND=192.168.1.20:8765`;
- a non-loopback bind with zero active paired Agents is refused, so pairing has to happen first.

`bash scripts/test-lan-mcp-gateway.sh` covers the first and last of these; restrict which hosts may connect with your own firewall rules.

## 7. Backups

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

## 8. What this service does not do

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
