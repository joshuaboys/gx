# Tutorial

| ID | Owner | Status |
|----|-------|--------|
| TUTR | @joshuaboys | Future |

## Purpose

Provide an interactive walkthrough that teaches new users how to use gx. `gx tutorial` guides users step-by-step through core features — cloning, navigating, listing, configuring — by actually running commands and explaining output in real time.

## In Scope

- `gx tutorial` — launch interactive tutorial
- Step-by-step walkthrough covering: clone, jump, ls, rebuild, config, recent, resume, dash
- Each step explains what it does, runs the command, and shows the result
- User confirms each step before proceeding (press Enter to continue)
- Tutorial uses a safe sandbox or the user's real environment with confirmation
- Skip option for individual steps
- Progress indicator (step N of M)

## Out of Scope

- Web-based or GUI tutorial
- Video or animated terminal recordings
- Tutorial content for v4 features (hooks, templates, TUI) — added when those ship
- Localization or multi-language support
- Tutorial state persistence (resume where you left off)

## Interfaces

**Depends on:**

- All stable modules — tutorial exercises each feature, so all referenced commands must be working

**Exposes:**

- `tutorial` subcommand in CLI

## Constraints

- Must not create permanent side effects without user consent (e.g., cloning a repo should ask first)
- Must work in any terminal that supports basic ANSI (no TUI library dependency)
- Must handle interruption gracefully (Ctrl+C exits cleanly)
- Tutorial content should be embedded, not fetched from network
- Should complete in under 5 minutes for a fast reader

## Ready Checklist

Change status to **Ready** when:

- [ ] Tutorial step sequence designed
- [ ] Sandbox vs real-environment strategy decided
- [ ] All referenced commands are stable and shipped
- [ ] Dependencies identified
- [ ] At least one task defined

## Work Items

_None yet -- this module is in Future status._
