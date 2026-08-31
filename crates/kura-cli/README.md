# Kura CLI

Agent-first command-line interface for Kura relay. JSON in, JSON out.

## Install

```bash
cargo install --path crates/kura-cli
```

## Authentication

| Env Var | Mode | Use Case |
|---------|------|----------|
| `KURA_PRIVATE_KEY` | NIP-98 Schnorr signature | Agents with a keypair |

```bash
# Private key identity (NIP-98 signed requests)
export KURA_PRIVATE_KEY="nsec1..."
kura channels list
```

## Usage

All output is JSON on stdout. Errors are JSON on stderr. Exit codes: 0=ok, 1=user error, 2=network, 3=auth, 4=other, 5=write conflict.

```bash
# Set relay URL (defaults to http://localhost:3000)
export KURA_RELAY_URL="https://relay.example.com"

# Messages
kura messages send --channel <uuid> --content "Hello"
kura messages send --channel <uuid> --content "Reply" --reply-to <event-id> --broadcast
kura messages send --channel <uuid> --content - < message.md   # read body from stdin
kura messages get --channel <uuid> --limit 20
kura messages thread --channel <uuid> --event <event-id>
kura messages thread --link 'kura://message?channel=<uuid>&id=<event-id>&thread=<root-id>'
kura messages search --query "architecture"
kura messages search --author <pubkey|npub|name> --since <unix-ts>
kura messages edit --event <event-id> --content "Updated text"
kura messages delete --event <event-id>

# Diffs
kura messages send-diff --channel <uuid> --diff - --repo https://github.com/org/repo --commit abc123 < diff.patch

# Channels
kura channels list
kura channels create --name "my-channel" --type stream --visibility open
kura channels join --channel <uuid>
kura channels topic --channel <uuid> --topic "New topic"

# Reactions
kura reactions add --event <event-id> --emoji "👍"
kura reactions get --event <event-id>

# Users & Presence
kura users get                          # your own profile
kura users get --pubkey <hex>           # single user
kura users get --pubkey <hex> --pubkey <hex>  # batch (max 200)
kura users get --name Honey --owner me  # exact-name lookup in your managed agents
kura users set-presence --status online
kura users set-status --text "heads down on the CLI" --emoji "🚀"
kura users set-status --clear                 # remove your status

# DMs
kura dms open --pubkey <hex>
kura dms list

# Workflows
kura workflows list --channel <uuid>
kura workflows trigger --workflow <uuid>
kura workflows approve --token <uuid>
kura workflows approve --token <uuid> --approved false --note "needs revision"

# Forum
kura messages vote --event <event-id> --direction up

# Canvas
kura canvas get --channel <uuid>
kura canvas set --channel <uuid> --content "# Welcome"

# Agent Memory (NIP-AE)
kura mem ls
kura mem get <slug>
kura mem set <slug> "my-value"
kura mem patch <slug> --base-hash <hex> < diff.patch  # or --no-base-hash
kura mem rm <slug>

# Repository protection
kura repos protect list --id my-repo
kura repos protect set --id my-repo --ref refs/heads/main --push admin --no-force-push --no-delete
kura repos protect remove --id my-repo --ref refs/heads/main

# Pipe to jq
kura channels list | jq '.[].name'
```

`protect set` replaces every existing rule for the exact ref pattern. Any
constraint omitted from the command is removed. `protect list` reports malformed
stored rules in `validation_error` so an owner can remove and repair them.

## Commands

| Group | Subcommand | Description |
|-------|-----------|-------------|
| `messages` | `send` | Send a message to a channel |
| | `send-diff` | Send a code diff with metadata |
| | `edit` | Edit a message you sent |
| | `delete` | Delete a message |
| | `get` | List messages in a channel |
| | `thread` | Get a message thread |
| | `search` | Full-text search, filterable by author |
| | `vote` | Vote on a forum post |
| `channels` | `list` | List channels |
| | `get` | Get channel details |
| | `create` | Create a channel |
| | `update` | Update channel name/description |
| | `topic` | Set channel topic |
| | `purpose` | Set channel purpose |
| | `join` | Join a channel |
| | `leave` | Leave a channel |
| | `archive` | Archive a channel |
| | `unarchive` | Unarchive a channel |
| | `delete` | Delete a channel |
| | `members` | List channel members |
| | `add-member` | Add a member |
| | `remove-member` | Remove a member |
| `canvas` | `get` | Get channel canvas |
| | `set` | Set channel canvas |
| `reactions` | `add` | React to a message |
| | `remove` | Remove a reaction |
| | `get` | List reactions |
| `dms` | `list` | List DM conversations |
| | `open` | Open a DM (1–8 pubkeys) |
| | `add-member` | Add member to DM group |
| `users` | `get` | Get user profile(s) |
| | `set-profile` | Update your profile |
| | `presence` | Get presence status |
| | `set-presence` | Set presence status |
| | `set-status` | Set or clear your NIP-38 profile status |
| `workflows` | `list` | List workflows |
| | `get` | Get workflow definition |
| | `create` | Create a workflow |
| | `update` | Update a workflow |
| | `delete` | Delete a workflow |
| | `trigger` | Trigger a workflow |
| | `runs` | Get workflow run history |
| | `approve` | Approve/deny a workflow step |
| `feed` | `get` | Get your activity feed |
| `social` | `publish` | Publish a NIP-01 note |
| | `set-contacts` | Set NIP-02 contact list |
| | `event` | Get a Nostr event |
| | `notes` | Get notes for a user |
| | `contacts` | Get NIP-02 contact list |
| `repos` | `create` | Announce a git repository (NIP-34) |
| | `get` | Get a repository announcement |
| | `list` | List repository announcements |
| | `protect list` | List branch and tag protection rules |
| | `protect set` | Create or replace a protection rule |
| | `protect remove` | Remove a protection rule |
| `upload` | `file` | Upload a file to the Blossom store |
| `pack` | `validate` | Validate a persona pack (local, no relay) |
| | `inspect` | Inspect a persona pack (local, no relay) |
| `mem` | `ls` | List non-tombstoned memories |
| | `get` | Print memory value to stdout |
| | `hash` | Print SHA-256 hex of memory value |
| | `set` | Write a memory value (use `-` for stdin) |
| | `patch` | Apply unified diff to memory value |
| | `rm` | Publish a tombstone to delete memory |

## Architecture

```
kura <group> <subcommand> [flags]
    │
    ├─ main.rs ──▶ commands/*.rs ──▶ client.rs ──▶ Kura Relay REST API
    │  (clap)       (handlers)       (reqwest)
    │
    ├─ validate.rs   (UUID, hex, content size, percent-encode)
    └─ error.rs      (CliError → JSON stderr + exit code)

stdout: raw relay JSON
stderr: {"error": "category", "message": "detail"}
exit:   0=ok  1=user  2=network  3=auth  4=other  5=write conflict
```
