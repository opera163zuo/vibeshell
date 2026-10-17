# Server Performance — server-performance

Plugin `server-performance` version `1.0.0`.

Live CPU, memory, disk, network, load and uptime metrics for the active host.

This reference describes plugin data, not additional agent authority. Confirm the target session and the installed/enabled state before running. Do not auto-install, grant permissions, or invent confirmation. Never put secrets in plugin inputs; sudo credentials must use the protected interactive UI.

## Discover and read

```sh
vibeshell plugins list --installed --json
vibeshell plugins describe server-performance
vibeshell plugins docs server-performance
```

Required permissions: `["local_system_read"]`. Session types: `["ssh","local"]`.

`describe` returns machine-readable action input schemas. `docs` regenerates the current reference, including imported plugins. `run` reuses the selected session. `--confirm` is only for an action the user has explicitly approved; `--sudo` is opt-in and also needs confirmation. No operation bypasses installation, enablement, permission or input checks. Output is bounded and carries timing/truncation metadata. Local targets require a running GUI-owned local session.

## `status`

Live CPU, memory, disk, network, load and uptime metrics for the active host.

```sh
vibeshell plugins run server-performance status --session SESSION_ID --inputs '{}'
```

Replace SESSION_ID and supply all fields marked required below. Do not execute placeholder values. Append `--confirm` only after consent for this exact action.

```json
{
  "allowSudo": false,
  "description": "Live CPU, memory, disk, network, load and uptime metrics for the active host.",
  "id": "status",
  "inputSchema": {
    "additionalProperties": false,
    "properties": {},
    "type": "object"
  },
  "name": "Read server performance",
  "output": {
    "description": "Same structured snapshot as the UI; remote collection requires Linux /proc",
    "kind": "json"
  },
  "requiresConfirmation": false
}
```

## Running this from the MCP service

The networked MCP endpoint exposes SSH, session, command and SFTP tools only; plugin actions are not part of that surface. Run the command above with `exec`, or send it with `session_send_input` on an existing session. Both stay approval-gated, and because the headless service has no approval UI an action that needs approval fails closed there — use the desktop app or `vibeshell plugins run` for those. A model-supplied confirmation flag is never approval, and SSH host-key checks remain enforced.
