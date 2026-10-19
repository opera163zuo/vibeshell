# Disk Usage — disk-usage

Plugin `disk-usage` version `1.1.0`.

Inspect filesystems and directory usage before a full disk becomes an incident.

This reference describes plugin data, not additional agent authority. Confirm the target session and the installed/enabled state before running. Do not auto-install, grant permissions, or invent confirmation. Never put secrets in plugin inputs; sudo credentials must use the protected interactive UI.

## Discover and read

```sh
zxyssh plugins list --installed --json
zxyssh plugins describe disk-usage
zxyssh plugins docs disk-usage
```

Required permissions: `["remote_exec","local_exec"]`. Session types: `["ssh","local"]`.

`describe` returns machine-readable action input schemas. `docs` regenerates the current reference, including imported plugins. `run` reuses the selected session. `--confirm` is only for an action the user has explicitly approved; `--sudo` is opt-in and also needs confirmation. No operation bypasses installation, enablement, permission or input checks. Output is bounded and carries timing/truncation metadata. Local targets require a running GUI-owned local session.

## `filesystems`

Show mounted filesystem capacity and usage.

```sh
zxyssh plugins run disk-usage filesystems --session SESSION_ID --inputs '{}'
```

Replace SESSION_ID and supply all fields marked required below. Do not execute placeholder values. Append `--confirm` only after consent for this exact action.

```json
{
  "allowSudo": false,
  "description": "Show mounted filesystem capacity and usage.",
  "elevate": false,
  "id": "filesystems",
  "inputSchema": {
    "additionalProperties": false,
    "properties": {},
    "required": [],
    "type": "object"
  },
  "name": "Filesystems",
  "output": {
    "columns": [],
    "delimiter": "\t",
    "kind": "text"
  },
  "requiresConfirmation": false
}
```

## `current-directory`

Summarize first-level disk usage in the current directory.

```sh
zxyssh plugins run disk-usage current-directory --session SESSION_ID --inputs '{}'
```

Replace SESSION_ID and supply all fields marked required below. Do not execute placeholder values. Append `--confirm` only after consent for this exact action.

```json
{
  "allowSudo": false,
  "description": "Summarize first-level disk usage in the current directory.",
  "elevate": false,
  "id": "current-directory",
  "inputSchema": {
    "additionalProperties": false,
    "properties": {},
    "required": [],
    "type": "object"
  },
  "name": "Current directory",
  "output": {
    "columns": [],
    "delimiter": "\t",
    "kind": "text"
  },
  "requiresConfirmation": false
}
```

## Running this from the MCP service

The networked MCP endpoint exposes SSH, session, command and SFTP tools only; plugin actions are not part of that surface. Run the command above with `exec`, or send it with `session_send_input` on an existing session. Both stay approval-gated, and because the headless service has no approval UI an action that needs approval fails closed there — use the desktop app or `zxyssh plugins run` for those. A model-supplied confirmation flag is never approval, and SSH host-key checks remain enforced.
