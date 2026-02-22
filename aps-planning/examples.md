# APS Examples

Real-world examples of APS specs at different scales.

> **Canonical examples:** For full worked examples with complete file trees, see
> [examples/user-auth/](../examples/user-auth/) and
> [examples/opencode-companion/](../examples/opencode-companion/). The examples
> below are compact summaries for quick context-window access.

## Example 1: Quick Feature (Simple Spec)

A small, self-contained dark mode toggle:

```markdown
# Dark Mode Toggle

| ID | Owner | Status |
|----|-------|--------|
| DARK | @sarah | Ready |

## Purpose

Users need to switch between light and dark themes to reduce eye strain.

## Success Criteria

- [ ] Toggle persists across page reloads
- [ ] All components respect theme setting

## Work Items

### DARK-001: Add theme toggle component

- **Intent:** Users can switch between light and dark themes
- **Expected Outcome:** Toggle in settings page switches theme instantly
- **Validation:** `npm test -- theme-toggle.test.ts`
- **Confidence:** high

### DARK-002: Persist theme preference

- **Intent:** Theme choice survives browser refresh
- **Expected Outcome:** localStorage stores preference, applies on load
- **Validation:** `npm test -- theme-persistence.test.ts`
- **Dependencies:** DARK-001
```

## Example 2: Module with Interfaces

An authentication module in a larger system:

```markdown
# User Authentication

| ID | Owner | Priority | Status |
|----|-------|----------|--------|
| AUTH | @josh | high | Ready |

## Purpose

Users need to register, log in, and maintain authenticated sessions.

## In Scope

- User registration with email/password
- Login/logout
- Password hashing and verification

## Out of Scope

- OAuth/SSO (separate module)
- Session token management (SESSION module)

## Interfaces

**Depends on:**
- DATABASE — user table schema

**Exposes:**
- `registerUser(email, password)` → User
- `verifyCredentials(email, password)` → boolean

## Constraints

- AUTH must not import from SESSION
- AUTH must not issue or validate tokens

## Work Items

### AUTH-001: Create user registration

- **Intent:** New users can create accounts
- **Expected Outcome:** POST /api/register creates user, returns 201
- **Validation:** `curl -X POST localhost:3000/api/register -d '{"email":"test@test.com","password":"secret"}' | jq .status`
- **Confidence:** high

### AUTH-002: Create credential verification

- **Intent:** Existing users can prove identity
- **Expected Outcome:** Function returns true for valid credentials, false otherwise
- **Validation:** `npm test -- auth.verify.test.ts`
- **Dependencies:** AUTH-001
- **Confidence:** high
```

## Example 3: Multi-Module Index

An initiative spanning multiple bounded areas:

```markdown
# Add User Authentication

## Overview

Add authentication and session management to the application so users
can register, log in, and maintain sessions.

## Problem & Success Criteria

**Problem:** All data is currently public. Users have no identity.

**Success Criteria:**
- [ ] Users can register and log in
- [ ] Sessions persist across browser refresh
- [ ] Unauthorized access returns 401

## Modules

| Module | Purpose | Status |
|--------|---------|--------|
| [auth](./modules/01-auth.aps.md) | Registration and login | Ready |
| [session](./modules/02-session.aps.md) | Token management | Draft |

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Password storage vulnerability | Use bcrypt, never store plaintext |
```

## Example 4: Action Plan

Breaking down a complex work item into checkpoints:

```markdown
# Action Plan: AUTH-001

| Field | Value |
|-------|-------|
| Source | [./modules/01-auth.aps.md](./modules/01-auth.aps.md) |
| Work Item | AUTH-001 — Create user registration |
| Status | In Progress |

## Prerequisites

- [ ] Database migration for users table is applied

## Actions

### Action 1 — Create users table migration

**Purpose:** Registration needs somewhere to store user data
**Produces:** Migration file, users table in database
**Checkpoint:** Users table exists with email and password_hash columns
**Validate:** `psql -c '\d users'`

### Action 2 — Implement registration endpoint

**Purpose:** Users need an API to create accounts
**Produces:** POST /api/register endpoint
**Checkpoint:** Endpoint accepts email/password, returns 201 with user ID
**Validate:** `npm test -- auth.register.test.ts`

### Action 3 — Add input validation

**Purpose:** Prevent invalid registrations
**Produces:** Validation middleware for registration
**Checkpoint:** Invalid inputs return 400 with descriptive errors
**Validate:** `npm test -- auth.validation.test.ts`

## Completion

- [ ] All checkpoints validated
- [ ] Work item marked complete in source module
```
