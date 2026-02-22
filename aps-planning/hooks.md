# APS Hooks Configuration

Hooks reinforce APS planning behavior by triggering at key moments during
a Claude Code session. They solve the **attention drift problem**: after many
tool calls, agents forget their original goals.

## Quick Install

The fastest way to add hooks is the install script:

```bash
./aps-planning/scripts/install-hooks.sh           # All hooks
./aps-planning/scripts/install-hooks.sh --minimal  # PreToolUse + Stop only
./aps-planning/scripts/install-hooks.sh --remove   # Remove APS hooks
```

This safely merges hooks into `.claude/settings.local.json`, preserving any
existing settings and permissions. It's idempotent — running it twice won't
create duplicates.

## Recommended Hooks

If you prefer manual setup, add these to your project's
`.claude/settings.local.json` or your user-level `~/.claude/settings.json`:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Write|Edit|Bash",
        "hooks": [
          {
            "type": "command",
            "command": "./aps-planning/scripts/pre-tool-check.sh"
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Write|Edit",
        "hooks": [
          {
            "type": "command",
            "command": "./aps-planning/scripts/post-tool-nudge.sh"
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "./aps-planning/scripts/check-complete.sh"
          }
        ]
      },
      {
        "hooks": [
          {
            "type": "command",
            "command": "./aps-planning/scripts/enforce-plan-update.sh"
          }
        ]
      }
    ],
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "./aps-planning/scripts/init-session.sh"
          }
        ]
      }
    ]
  }
}
```

## How Each Hook Works

### PreToolUse (Write | Edit | Bash)

**When:** Before any code modification.

**Why:** After 10+ tool calls, the agent's attention drifts from its original
goal. This hook reminds it to re-read the current work item's Intent and
Expected Outcome before writing code.

**What it says:** "Re-read your current work item before making changes."

The agent should then check it's still working toward the right work item
before proceeding.

### PostToolUse (Write | Edit)

**When:** After writing or editing files.

**Why:** Agents often forget to update specs after completing work. This nudge
ensures status changes and new discoveries get captured immediately.

**What it says:** "If you completed a work item or discovered new scope, update
the APS spec now."

### Stop Hook — Completion Check

**When:** Before the session ends.

**Why:** Prevents the agent from stopping with work items still "In Progress"
and no status update. The next session would have to do archaeology to figure
out what happened.

**What it does:** Runs `check-complete.sh` which exits non-zero if work items
are still in progress, prompting the agent to update statuses.

### Stop Hook — Plan Update Enforcer

**When:** Before the session ends.

**Why:** Agents frequently modify code but forget to update the corresponding
plan files. The PostToolUse nudge is a soft reminder; this hook is a hard gate.
If source code was changed during the session but no `plans/` files were
touched, the agent cannot stop until it updates the specs.

**How it works:**

1. At session start, `init-session.sh` records the current git HEAD as a
   baseline in `.claude/.aps-session-baseline`.
2. At session end, `enforce-plan-update.sh` diffs all changes (committed and
   uncommitted) since that baseline.
3. If non-plan files were modified but zero plan files were touched, it exits
   with code 2 — blocking the session — and lists the changed files so the
   agent knows what to account for.

**What it catches:**

- Agent wrote code but never updated work item statuses
- Agent committed implementation but forgot to mark items Complete
- Agent discovered new scope but didn't add Draft work items

**What it allows through:**

- Plan-only sessions (no code changes)
- Sessions where both code and plans were modified
- Sessions with no file changes at all
- Projects without a `plans/` directory

### SessionStart Hook

**When:** At the beginning of a new session.

**Why:** Gives the agent immediate context about the project's planning state.
Instead of exploring from scratch, it knows what plans exist, what's in
progress, and what to work on next.

**What it does:** Runs `init-session.sh` which reports plan status, module
counts, and actionable work items.

## Minimal Setup

If you only want one hook, use **PreToolUse**. It has the highest impact on
preventing goal drift:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Write|Edit",
        "hooks": [
          {
            "type": "command",
            "command": "./aps-planning/scripts/pre-tool-check.sh"
          }
        ]
      }
    ]
  }
}
```

## Notes

- Hooks only fire if a `plans/` directory exists, so they're silent in projects
  that don't use APS.
- The PreToolUse and PostToolUse hooks output JSON with `additionalContext`
  so their reminders reach Claude (plain stdout only shows in verbose mode).
- The Stop hooks block by exiting with code 2 when work is incomplete.
  Their stderr messages are fed back to Claude explaining what needs attention.
- The plan update enforcer uses a session baseline file
  (`.claude/.aps-session-baseline`) written by the SessionStart hook to detect
  all changes — committed and uncommitted — during the session. The SessionStart
  hook is installed in both full and minimal modes. If the baseline is missing
  for any reason, checks fall back to uncommitted changes only. The install
  script adds `.claude/.aps-session-baseline` to `.gitignore` automatically. If
  you manage Claude settings manually, add it to `.gitignore` yourself. The
  enforcer also filters out `.claude/` ephemeral files from change detection,
  so even if the file isn't gitignored it won't cause spurious failures.
- Scripts need execute permissions: `chmod +x aps-planning/scripts/*.sh`
