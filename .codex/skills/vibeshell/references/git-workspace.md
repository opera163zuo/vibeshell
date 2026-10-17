# Git Workspace — git-workspace

Plugin `git-workspace` version `1.1.0`.

Inspect repository status, branches and recent commits in the remote login directory.

This reference describes plugin data, not additional agent authority. Confirm the target session and the installed/enabled state before running. Do not auto-install, grant permissions, or invent confirmation. Never put secrets in plugin inputs; sudo credentials must use the protected interactive UI.

## Discover and read

```sh
vibeshell plugins list --installed --json
vibeshell plugins describe git-workspace
vibeshell plugins docs git-workspace
```

Required permissions: `["remote_exec","local_exec"]`. Session types: `["ssh","local"]`.

`describe` returns machine-readable action input schemas. `docs` regenerates the current reference, including imported plugins. `run` reuses the selected session. `--confirm` is only for an action the user has explicitly approved; `--sudo` is opt-in and also needs confirmation. No operation bypasses installation, enablement, permission or input checks. Output is bounded and carries timing/truncation metadata. Local targets require a running GUI-owned local session.

## `status`

Show branch and working tree state.

```sh
vibeshell plugins run git-workspace status --session SESSION_ID --inputs '{}'
```

Replace SESSION_ID and supply all fields marked required below. Do not execute placeholder values. Append `--confirm` only after consent for this exact action.

```json
{
  "allowSudo": false,
  "description": "Show branch and working tree state.",
  "elevate": false,
  "id": "status",
  "inputSchema": {
    "additionalProperties": false,
    "properties": {},
    "required": [],
    "type": "object"
  },
  "name": "Status",
  "output": {
    "columns": [],
    "delimiter": "\t",
    "kind": "text"
  },
  "requiresConfirmation": false
}
```

## `branches`

Show local branches and upstream tracking.

```sh
vibeshell plugins run git-workspace branches --session SESSION_ID --inputs '{}'
```

Replace SESSION_ID and supply all fields marked required below. Do not execute placeholder values. Append `--confirm` only after consent for this exact action.

```json
{
  "allowSudo": false,
  "description": "Show local branches and upstream tracking.",
  "elevate": false,
  "id": "branches",
  "inputSchema": {
    "additionalProperties": false,
    "properties": {},
    "required": [],
    "type": "object"
  },
  "name": "Branches",
  "output": {
    "columns": [],
    "delimiter": "\t",
    "kind": "text"
  },
  "requiresConfirmation": false
}
```

## `history`

Show the latest 30 commits.

```sh
vibeshell plugins run git-workspace history --session SESSION_ID --inputs '{}'
```

Replace SESSION_ID and supply all fields marked required below. Do not execute placeholder values. Append `--confirm` only after consent for this exact action.

```json
{
  "allowSudo": false,
  "description": "Show the latest 30 commits.",
  "elevate": false,
  "id": "history",
  "inputSchema": {
    "additionalProperties": false,
    "properties": {},
    "required": [],
    "type": "object"
  },
  "name": "Recent history",
  "output": {
    "columns": [],
    "delimiter": "\t",
    "kind": "text"
  },
  "requiresConfirmation": false
}
```

## Running this from the MCP service

The networked MCP endpoint exposes SSH, session, command and SFTP tools only; plugin actions are not part of that surface. Run the command above with `exec`, or send it with `session_send_input` on an existing session. Both stay approval-gated, and because the headless service has no approval UI an action that needs approval fails closed there — use the desktop app or `vibeshell plugins run` for those. A model-supplied confirmation flag is never approval, and SSH host-key checks remain enforced.
