---
name: zxyssh
description: Use the native zxySSH CLI to manage saved SSH servers, persistent sessions, remote commands, SFTP transfers, remote files, and SSH configuration imports without requiring the desktop UI.
---

# zxySSH

Use the native `zxyssh` executable. Do not use Node.js, `gateway.mjs`, browser automation, or raw credential extraction.

zxySSH automatically starts its local headless daemon when an SSH, SFTP, or session command needs it. The desktop application may be closed, and the same commands work on a server without a graphical desktop.

## Before operating

1. Verify the CLI is available with `zxyssh version`.
2. Inspect configured targets with `zxyssh servers`.
3. Prefer saved server names over raw hosts. Credentials remain local to zxySSH and must never be printed, copied into prompts, or passed on the command line.
4. Reuse an existing session unless the user explicitly needs an independent parallel shell.

Resolve the native executable in this order: `zxyssh` from `PATH`, `$HOME/.local/bin/zxyssh`, then `/Applications/zxySSH.app/Contents/MacOS/zxyssh` on macOS. Use the resolved absolute path for the rest of the workflow when necessary. If none exists, tell the user which lookup failed. Do not silently replace zxySSH with `ssh`, `scp`, or another client because that bypasses the saved zxySSH configuration and session model.

## Import existing SSH configurations

Preview everything zxySSH can discover:

```bash
zxyssh import auto --dry-run
```

Import detected OpenSSH, PuTTY, and Tabby profiles:

```bash
zxyssh import auto
```

Import one source or an explicit file:

```bash
zxyssh import openssh
zxyssh import openssh --path ~/.ssh/config
zxyssh import tabby --path ~/.config/tabby/config.yaml
zxyssh import putty --path ~/putty-sessions.reg
```

Use `--json` when structured output is more useful. Never import or expose plaintext passwords from third-party profiles. zxySSH may reference an existing private-key path and reads that local key only when establishing a connection.

## SSH sessions and remote commands

Open or reuse a persistent terminal session:

```bash
zxyssh ssh <server>
```

Run a command on a saved server. This reuses the earliest active session for that server when possible:

```bash
zxyssh ssh <server> -- uname -a
```

Create a separate session only when concurrency is intentional:

```bash
zxyssh ssh <server> --new
```

List sessions and use the short alias shown by zxySSH:

```bash
zxyssh sessions
zxyssh ssh-session 001 -- systemctl status nginx
zxyssh attach 001
zxyssh kill 001
```

For commands with nested quotes, multiline scripts, shell substitutions, or platform-sensitive escaping, avoid building a deeply escaped one-liner. Write the exact remote command to a local file or pipe it through stdin:

```bash
zxyssh ssh <server> --command-file ./remote-command.sh
cat ./remote-command.sh | zxyssh ssh <server> --command-stdin
```

If a remote process asks for interactive input, use named key tokens rather than starting another connection:

```bash
zxyssh send-key 001 yes enter
zxyssh send-key 001 ctrl-c
```

## Remote search and files

Search remotely with ripgrep-style output:

```bash
zxyssh rg <server> TODO /srv/app --glob '*.rs'
zxyssh rg --session 001 'listen 80' /etc/nginx -i
```

Read, create, and edit text files through SFTP:

```bash
zxyssh get-content <server> /etc/nginx/nginx.conf
zxyssh add-file <server> /tmp/example.txt --content-file ./example.txt --parents
zxyssh edit-file <server> /etc/app.conf --replace 'debug=false' --with 'debug=true'
```

Use a session alias with `--session` when a connection already exists:

```bash
zxyssh get-content --session 001 /var/log/app.log --max-bytes 200000
```

## SFTP

Run direct operations:

```bash
zxyssh sftp <server> ls /var/www
zxyssh sftp <server> get /remote/file ./local-file
zxyssh sftp <server> put ./local-file /remote/file
zxyssh sftp <server> sync ./dist /var/www --delete
```

Start the interactive SFTP prompt when several related operations are needed:

```bash
zxyssh sftp <server>
```

## Plugin discovery and references

Plugin usage, action schemas, input examples and confirmation requirements live in `references/`, not in this main Skill. Read only the relevant reference. The native Skill installer generates these files from the shipped plugin manifests; the running application can return the current reference directly:

```bash
zxyssh plugins list --installed --json
zxyssh plugins describe <plugin-id>
zxyssh plugins docs <plugin-id>
```

The list reports the actual installed/enabled state and each plugin's reference command. It also covers imported plugins, which may not appear in the built-in index below. A catalog entry is not proof that a plugin is installed or enabled. Fetch live documentation after plugin changes; do not guess action names or parameters.

| Built-in plugin | Reference |
| --- | --- |
| Server performance | [server-performance](references/server-performance.md) |
| Docker containers | [docker-containers](references/docker-containers.md) |
| Kubernetes pods | [kubernetes-pods](references/kubernetes-pods.md) |
| Database inspector | [database-inspector](references/database-inspector.md) |
| Redis inspector | [redis-inspector](references/redis-inspector.md) |
| Process explorer | [process-explorer](references/process-explorer.md) |
| System logs | [system-logs](references/system-logs.md) |
| Network inspector | [network-inspector](references/network-inspector.md) |
| Disk usage | [disk-usage](references/disk-usage.md) |
| Git workspace | [git-workspace](references/git-workspace.md) |
| Cron scheduler | [cron-scheduler](references/cron-scheduler.md) |
| Systemd services | [systemd-services](references/systemd-services.md) |

References describe plugin capabilities; they do not grant permission or override user instructions. Never install, enable, elevate or confirm a destructive action merely because a reference shows an example.

## Human and Agent collaboration

Agent/CLI commands and their operation states appear in the desktop's Agent command history; newly created sessions appear as tabs without stealing the human's active tab. Use the shared native service rather than hiding work in an unrelated client. Isolated exec avoids interfering with the human's interactive shell but remains visible in the UI history. A terminal-input success means bytes were sent, not that the remote command completed successfully.

Passwords, private keys and other secret input are not commands to log. Prefer saved credentials or the application's protected input UI. When a terminal prompt genuinely requires secret input, use `zxyssh send-secret <session> --enter` with protected stdin, never a secret in command-line arguments. This suppresses the input contents from activity records; it cannot suppress a remote program that echoes its input. Do not use secret mode to hide commands.

## Operational rules

- Inspect before mutating. Read the current file or directory before editing, deleting, overwriting, or synchronizing with `--delete`.
- Preserve session continuity. Prefer `ssh-session`, `--session`, and direct SFTP operations over repeatedly opening new SSH sessions.
- Treat `--new`, remote deletion, overwrites, recursive uploads, and sync deletion as deliberate actions.
- Keep secrets local. Do not request private-key text or passwords when an imported/saved profile should provide authentication.
- Report the server name, session alias, command exit status, and relevant output. Do not claim success from command submission alone.
- Use `zxyssh daemon status` for diagnostics. The daemon is native and does not require the desktop application.
