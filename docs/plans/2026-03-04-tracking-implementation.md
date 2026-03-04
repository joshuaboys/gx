# Tracking Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add project visit tracking to gx — `lastVisited` timestamps, `gx recent`, and `gx resume`.

**Architecture:** Extend `IndexEntry` with an optional `lastVisited` field. `resolve` updates it as a side effect (no separate `touch` command). Two new commands (`recent`, `resume`) and a relative-time utility.

**Tech Stack:** TypeScript, Bun, bun:test

---

### Task 1: Add `lastVisited` to IndexEntry type

**Files:**

- Modify: `src/types.ts:26-30`

**Step 1: Write the type change**

In `src/types.ts`, add `lastVisited` to `IndexEntry`:

```typescript
export interface IndexEntry {
  path: string;
  url: string;
  clonedAt: string;
  lastVisited?: string;
}
```

**Step 2: Verify types compile**

Run: `bun x tsc --noEmit`
Expected: PASS (no errors — field is optional, so all existing code is compatible)

**Step 3: Run existing tests to confirm no regressions**

Run: `bun test`
Expected: All existing tests pass unchanged.

**Step 4: Commit**

```bash
git add src/types.ts
git commit -m "feat(tracking): add lastVisited field to IndexEntry"
```

---

### Task 2: Add `touch()` and `recent()` methods to ProjectIndex

**Files:**

- Modify: `src/lib/index.ts:15` (inside `ProjectIndex` class)
- Test: `tests/lib/index.test.ts`

**Step 1: Write the failing tests**

Add to `tests/lib/index.test.ts`:

```typescript
test("touch updates lastVisited on existing entry", async () => {
  const idx = await ProjectIndex.load(indexPath);
  idx.add("gx", {
    path: "/tmp/gx",
    url: "",
    clonedAt: "2026-01-01T00:00:00Z",
  });
  const before = new Date().toISOString();
  const found = idx.touch("gx");
  const after = new Date().toISOString();
  expect(found).toBe(true);
  const entry = idx.list().find((e) => e.name === "gx");
  expect(entry!.lastVisited).toBeDefined();
  expect(entry!.lastVisited! >= before).toBe(true);
  expect(entry!.lastVisited! <= after).toBe(true);
});

test("touch returns false for unknown project", async () => {
  const idx = await ProjectIndex.load(indexPath);
  expect(idx.touch("nope")).toBe(false);
});

test("recent returns entries sorted by lastVisited descending", async () => {
  const idx = await ProjectIndex.load(indexPath);
  idx.add("old", {
    path: "/tmp/old",
    url: "",
    clonedAt: "2026-01-01T00:00:00Z",
    lastVisited: "2026-01-01T00:00:00Z",
  });
  idx.add("new", {
    path: "/tmp/new",
    url: "",
    clonedAt: "2026-01-02T00:00:00Z",
    lastVisited: "2026-03-01T00:00:00Z",
  });
  idx.add("mid", {
    path: "/tmp/mid",
    url: "",
    clonedAt: "2026-01-03T00:00:00Z",
    lastVisited: "2026-02-01T00:00:00Z",
  });
  const result = idx.recent();
  expect(result.map(([name]) => name)).toEqual(["new", "mid", "old"]);
});

test("recent falls back to clonedAt when lastVisited is absent", async () => {
  const idx = await ProjectIndex.load(indexPath);
  idx.add("visited", {
    path: "/tmp/visited",
    url: "",
    clonedAt: "2026-01-01T00:00:00Z",
    lastVisited: "2026-02-01T00:00:00Z",
  });
  idx.add("unvisited", {
    path: "/tmp/unvisited",
    url: "",
    clonedAt: "2026-03-01T00:00:00Z",
  });
  const result = idx.recent();
  // unvisited has clonedAt 2026-03-01 > visited's lastVisited 2026-02-01
  expect(result.map(([name]) => name)).toEqual(["unvisited", "visited"]);
});

test("recent respects limit parameter", async () => {
  const idx = await ProjectIndex.load(indexPath);
  idx.add("a", {
    path: "/tmp/a",
    url: "",
    clonedAt: "",
    lastVisited: "2026-03-03T00:00:00Z",
  });
  idx.add("b", {
    path: "/tmp/b",
    url: "",
    clonedAt: "",
    lastVisited: "2026-03-02T00:00:00Z",
  });
  idx.add("c", {
    path: "/tmp/c",
    url: "",
    clonedAt: "",
    lastVisited: "2026-03-01T00:00:00Z",
  });
  const result = idx.recent(2);
  expect(result).toHaveLength(2);
  expect(result.map(([name]) => name)).toEqual(["a", "b"]);
});
```

**Step 2: Run tests to verify they fail**

Run: `bun test tests/lib/index.test.ts`
Expected: FAIL — `touch` and `recent` methods don't exist yet.

**Step 3: Implement `touch()` and `recent()`**

Add these methods to the `ProjectIndex` class in `src/lib/index.ts`, after the `resolve` method (around line 70):

```typescript
touch(name: string): boolean {
  const entry = this.data.projects[name];
  if (!entry) return false;
  entry.lastVisited = new Date().toISOString();
  return true;
}

recent(limit?: number): Array<[string, IndexEntry]> {
  const entries = Object.entries(this.data.projects).sort((a, b) => {
    const timeA = a[1].lastVisited || a[1].clonedAt || "";
    const timeB = b[1].lastVisited || b[1].clonedAt || "";
    return timeB.localeCompare(timeA);
  });
  return limit ? entries.slice(0, limit) : entries;
}
```

**Step 4: Run tests to verify they pass**

Run: `bun test tests/lib/index.test.ts`
Expected: All tests pass.

**Step 5: Commit**

```bash
git add src/lib/index.ts tests/lib/index.test.ts
git commit -m "feat(tracking): add touch() and recent() to ProjectIndex"
```

---

### Task 3: Add relative time utility

**Files:**

- Create: `src/lib/time.ts`
- Test: `tests/lib/time.test.ts`

**Step 1: Write the failing test**

Create `tests/lib/time.test.ts`:

```typescript
import { test, expect } from "bun:test";
import { relativeTime } from "../../src/lib/time.ts";

test("returns 'just now' for timestamps within 60 seconds", () => {
  const now = new Date();
  expect(relativeTime(now.toISOString())).toBe("just now");
});

test("returns minutes ago", () => {
  const t = new Date(Date.now() - 5 * 60 * 1000).toISOString();
  expect(relativeTime(t)).toBe("5 minutes ago");
});

test("returns '1 minute ago' for singular", () => {
  const t = new Date(Date.now() - 90 * 1000).toISOString();
  expect(relativeTime(t)).toBe("1 minute ago");
});

test("returns hours ago", () => {
  const t = new Date(Date.now() - 3 * 60 * 60 * 1000).toISOString();
  expect(relativeTime(t)).toBe("3 hours ago");
});

test("returns days ago", () => {
  const t = new Date(Date.now() - 2 * 24 * 60 * 60 * 1000).toISOString();
  expect(relativeTime(t)).toBe("2 days ago");
});

test("returns weeks ago", () => {
  const t = new Date(Date.now() - 14 * 24 * 60 * 60 * 1000).toISOString();
  expect(relativeTime(t)).toBe("2 weeks ago");
});

test("returns months ago", () => {
  const t = new Date(Date.now() - 60 * 24 * 60 * 60 * 1000).toISOString();
  expect(relativeTime(t)).toBe("2 months ago");
});

test("returns empty string for empty input", () => {
  expect(relativeTime("")).toBe("");
});
```

**Step 2: Run tests to verify they fail**

Run: `bun test tests/lib/time.test.ts`
Expected: FAIL — module doesn't exist.

**Step 3: Implement relativeTime**

Create `src/lib/time.ts`:

```typescript
export function relativeTime(iso: string): string {
  if (!iso) return "";
  const diff = Date.now() - new Date(iso).getTime();
  const seconds = Math.floor(diff / 1000);
  if (seconds < 60) return "just now";
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60)
    return `${minutes} ${minutes === 1 ? "minute" : "minutes"} ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours} ${hours === 1 ? "hour" : "hours"} ago`;
  const days = Math.floor(hours / 24);
  if (days < 14) return `${days} ${days === 1 ? "day" : "days"} ago`;
  const weeks = Math.floor(days / 7);
  if (days < 60) return `${weeks} ${weeks === 1 ? "week" : "weeks"} ago`;
  const months = Math.floor(days / 30);
  return `${months} ${months === 1 ? "month" : "months"} ago`;
}
```

**Step 4: Run tests to verify they pass**

Run: `bun test tests/lib/time.test.ts`
Expected: All pass.

**Step 5: Commit**

```bash
git add src/lib/time.ts tests/lib/time.test.ts
git commit -m "feat(tracking): add relativeTime utility"
```

---

### Task 4: Update `resolve` to record visits

**Files:**

- Modify: `src/commands/resolve.ts`
- Test: `tests/commands/resolve.test.ts` (create if not exists)

**Step 1: Write the failing test**

Create `tests/commands/resolve.test.ts`:

```typescript
import { test, expect, beforeEach, afterEach } from "bun:test";
import { ProjectIndex } from "../../src/lib/index.ts";
import { join } from "path";
import { mkdtemp, rm } from "fs/promises";
import { tmpdir } from "os";

let tmpDir: string;
let indexPath: string;

beforeEach(async () => {
  tmpDir = await mkdtemp(join(tmpdir(), "gx-test-"));
  indexPath = join(tmpDir, "index.json");
});

afterEach(async () => {
  await rm(tmpDir, { recursive: true });
});

test("resolve updates lastVisited on exact match", async () => {
  const idx = await ProjectIndex.load(indexPath);
  idx.add("gx", {
    path: "/tmp/gx",
    url: "https://github.com/joshuaboys/gx",
    clonedAt: "2026-01-01T00:00:00Z",
  });
  await idx.save(indexPath);

  // Call resolve — it should update lastVisited and save
  // We need to capture stdout and suppress process.exit
  // Instead, test the index method integration directly:
  const idx2 = await ProjectIndex.load(indexPath);
  const found = idx2.touch("gx");
  expect(found).toBe(true);
  await idx2.save(indexPath);

  const idx3 = await ProjectIndex.load(indexPath);
  const entry = idx3.list().find((e) => e.name === "gx");
  expect(entry!.lastVisited).toBeDefined();
});
```

Note: Full integration testing of the `resolve` command with stdout/stderr capture is harder in bun:test. The unit test on `touch()` in Task 2 covers the core logic. This test verifies the save-and-reload cycle.

**Step 2: Modify `src/commands/resolve.ts`**

Update `resolve` to call `touch` and save after a successful match (exact or fuzzy auto-jump):

```typescript
import { ProjectIndex } from "../lib/index.ts";
import { fuzzyMatch } from "../lib/fuzzy.ts";
import type { Config } from "../types.ts";

const AUTO_JUMP_THRESHOLD = 0.85;

export async function resolve(
  name: string,
  indexPath: string,
  config: Config,
  listAll = false,
): Promise<void> {
  const idx = await ProjectIndex.load(indexPath);

  if (listAll) {
    console.log(idx.names().join("\n"));
    return;
  }

  // Try exact match first
  const path = idx.resolve(name);
  if (path) {
    idx.touch(name);
    await idx.save(indexPath);
    console.log(path);
    return;
  }

  // Fall back to fuzzy matching
  const entries = idx.list().map((e) => ({ name: e.name, path: e.path }));
  const matches = fuzzyMatch(name, entries, config.similarityThreshold);

  if (matches.length === 0) {
    console.error(`Project '${name}' not found`);
    process.exit(1);
  }

  const first = matches[0];
  if (matches.length === 1 && first && first.score >= AUTO_JUMP_THRESHOLD) {
    console.error(
      `Fuzzy match: '${name}' -> '${first.name}' (${(first.score * 100).toFixed(0)}%)`,
    );
    idx.touch(first.name);
    await idx.save(indexPath);
    console.log(first.path);
    return;
  }

  // Multiple candidates or single low-confidence match: show list
  console.error(`No exact match for '${name}'. Did you mean:`);
  for (const [i, m] of matches.entries()) {
    console.error(`  ${i + 1}. ${m.name} (${(m.score * 100).toFixed(0)}%)`);
  }
  process.exit(1);
}
```

**Step 3: Run all tests**

Run: `bun test`
Expected: All pass.

**Step 4: Commit**

```bash
git add src/commands/resolve.ts tests/commands/resolve.test.ts
git commit -m "feat(tracking): update resolve to record lastVisited"
```

---

### Task 5: Update `clone` to set `lastVisited`

**Files:**

- Modify: `src/commands/clone.ts:54-58`

**Step 1: Update clone to include lastVisited**

In `src/commands/clone.ts`, change the `idx.add()` call to include `lastVisited`:

```typescript
const now = new Date().toISOString();
idx.add(parsed.repo, {
  path: targetDir,
  url: cloneUrl,
  clonedAt: now,
  lastVisited: now,
});
```

**Step 2: Run all tests**

Run: `bun test`
Expected: All pass.

**Step 3: Commit**

```bash
git add src/commands/clone.ts
git commit -m "feat(tracking): set lastVisited on clone"
```

---

### Task 6: Add `gx recent` command

**Files:**

- Create: `src/commands/recent.ts`
- Create: `tests/commands/recent.test.ts`
- Modify: `src/index.ts` (add case for "recent")

**Step 1: Write the failing test**

Create `tests/commands/recent.test.ts`:

```typescript
import { test, expect, beforeEach, afterEach } from "bun:test";
import { ProjectIndex } from "../../src/lib/index.ts";
import { join } from "path";
import { mkdtemp, rm } from "fs/promises";
import { tmpdir } from "os";

let tmpDir: string;
let indexPath: string;

beforeEach(async () => {
  tmpDir = await mkdtemp(join(tmpdir(), "gx-test-"));
  indexPath = join(tmpDir, "index.json");
});

afterEach(async () => {
  await rm(tmpDir, { recursive: true });
});

test("recent lists projects sorted by lastVisited", async () => {
  const idx = await ProjectIndex.load(indexPath);
  idx.add("old", {
    path: "/tmp/old",
    url: "",
    clonedAt: "2026-01-01T00:00:00Z",
    lastVisited: "2026-01-01T00:00:00Z",
  });
  idx.add("new", {
    path: "/tmp/new",
    url: "",
    clonedAt: "2026-01-02T00:00:00Z",
    lastVisited: "2026-03-01T00:00:00Z",
  });
  await idx.save(indexPath);

  const result = idx.recent();
  expect(result[0][0]).toBe("new");
  expect(result[1][0]).toBe("old");
});

test("recent with empty index produces no output", async () => {
  const idx = await ProjectIndex.load(indexPath);
  const result = idx.recent();
  expect(result).toHaveLength(0);
});
```

**Step 2: Run tests to verify they pass**

These tests use `idx.recent()` which was implemented in Task 2, so they should already pass:

Run: `bun test tests/commands/recent.test.ts`
Expected: PASS.

**Step 3: Create the recent command handler**

Create `src/commands/recent.ts`:

```typescript
import { ProjectIndex } from "../lib/index.ts";
import { relativeTime } from "../lib/time.ts";

export async function recent(indexPath: string, limit?: number): Promise<void> {
  const idx = await ProjectIndex.load(indexPath);
  const entries = idx.recent(limit);

  if (entries.length === 0) {
    console.error("No projects indexed. Clone a repo or run 'gx rebuild'.");
    return;
  }

  const maxName = Math.max(...entries.map(([name]) => name.length));
  for (const [name, entry] of entries) {
    const time = relativeTime(entry.lastVisited || entry.clonedAt);
    const timeStr = time ? `  ${time}` : "";
    console.log(`${name.padEnd(maxName)}  ${entry.path}${timeStr}`);
  }
}
```

**Step 4: Wire into CLI**

In `src/index.ts`, add the import at the top:

```typescript
import { recent } from "./commands/recent.ts";
```

Add this case in the switch statement, before `default`:

```typescript
    case "recent": {
      const nFlag = args.indexOf("-n");
      let limit: number | undefined;
      if (nFlag >= 0 && args[nFlag + 1]) {
        limit = parseInt(args[nFlag + 1], 10);
        if (isNaN(limit) || limit < 1) {
          console.error("Usage: gx recent [-n <count>]");
          process.exit(1);
        }
      }
      await recent(indexPath, limit);
      break;
    }
```

Also update the help text in the `--help` block to include:

```
  gx recent              List recently visited projects
  gx recent -n <N>       Show last N projects
```

**Step 5: Run all tests**

Run: `bun test`
Expected: All pass.

**Step 6: Commit**

```bash
git add src/commands/recent.ts tests/commands/recent.test.ts src/index.ts
git commit -m "feat(tracking): add gx recent command"
```

---

### Task 7: Add `gx resume` command

**Files:**

- Create: `src/commands/resume.ts`
- Create: `tests/commands/resume.test.ts`
- Modify: `src/index.ts` (add case for "resume")

**Step 1: Write the failing test**

Create `tests/commands/resume.test.ts`:

```typescript
import { test, expect, beforeEach, afterEach } from "bun:test";
import { ProjectIndex } from "../../src/lib/index.ts";
import { getResumeContext } from "../../src/commands/resume.ts";
import { join } from "path";
import { mkdtemp, rm, mkdir } from "fs/promises";
import { tmpdir } from "os";

let tmpDir: string;
let indexPath: string;

beforeEach(async () => {
  tmpDir = await mkdtemp(join(tmpdir(), "gx-test-"));
  indexPath = join(tmpDir, "index.json");
});

afterEach(async () => {
  await rm(tmpDir, { recursive: true });
});

test("getResumeContext returns context for a git repo", async () => {
  // Create a real git repo
  const repoDir = join(tmpDir, "myrepo");
  await mkdir(repoDir, { recursive: true });
  let proc = Bun.spawn(["git", "init"], {
    cwd: repoDir,
    stdout: "ignore",
    stderr: "ignore",
  });
  await proc.exited;
  proc = Bun.spawn(["git", "commit", "--allow-empty", "-m", "initial"], {
    cwd: repoDir,
    stdout: "ignore",
    stderr: "ignore",
  });
  await proc.exited;

  const ctx = await getResumeContext(repoDir);
  expect(ctx.branch).toBeDefined();
  expect(typeof ctx.dirtyCount).toBe("number");
  expect(ctx.lastCommit).toBeDefined();
});

test("getResumeContext returns null for missing directory", async () => {
  const ctx = await getResumeContext("/nonexistent/path");
  expect(ctx).toBeNull();
});

test("resume fails for unknown project name", async () => {
  const idx = await ProjectIndex.load(indexPath);
  await idx.save(indexPath);
  // The resolve would return null for unknown name
  expect(idx.resolve("unknown")).toBeNull();
});
```

**Step 2: Run tests to verify they fail**

Run: `bun test tests/commands/resume.test.ts`
Expected: FAIL — `getResumeContext` doesn't exist.

**Step 3: Implement the resume command**

Create `src/commands/resume.ts`:

```typescript
import { ProjectIndex } from "../lib/index.ts";
import { fuzzyMatch } from "../lib/fuzzy.ts";
import { relativeTime } from "../lib/time.ts";
import type { Config } from "../types.ts";

const AUTO_JUMP_THRESHOLD = 0.85;

export interface ResumeContext {
  branch: string;
  dirtyCount: number;
  lastCommit: string;
}

async function runGit(cwd: string, args: string[]): Promise<string> {
  try {
    const proc = Bun.spawn(["git", ...args], {
      cwd,
      stdout: "pipe",
      stderr: "ignore",
    });
    const exitCode = await proc.exited;
    if (exitCode !== 0) return "";
    return (await new Response(proc.stdout).text()).trim();
  } catch {
    return "";
  }
}

export async function getResumeContext(
  dir: string,
): Promise<ResumeContext | null> {
  try {
    const { statSync } = await import("fs");
    if (!statSync(dir).isDirectory()) return null;
  } catch {
    return null;
  }

  const branch = await runGit(dir, ["branch", "--show-current"]);
  const statusOutput = await runGit(dir, ["status", "--porcelain"]);
  const dirtyCount = statusOutput
    ? statusOutput.split("\n").filter((l) => l.length > 0).length
    : 0;
  const lastCommit = await runGit(dir, ["log", "-1", "--format=%h %s (%cr)"]);

  return { branch: branch || "HEAD (detached)", dirtyCount, lastCommit };
}

function resolveName(
  name: string,
  idx: ProjectIndex,
  config: Config,
): { resolvedName: string; path: string } | null {
  // Exact match
  const path = idx.resolve(name);
  if (path) return { resolvedName: name, path };

  // Fuzzy match
  const entries = idx.list().map((e) => ({ name: e.name, path: e.path }));
  const matches = fuzzyMatch(name, entries, config.similarityThreshold);

  if (matches.length === 0) return null;

  const first = matches[0];
  if (matches.length === 1 && first && first.score >= AUTO_JUMP_THRESHOLD) {
    console.error(
      `Fuzzy match: '${name}' -> '${first.name}' (${(first.score * 100).toFixed(0)}%)`,
    );
    return { resolvedName: first.name, path: first.path };
  }

  // Multiple candidates: show list and bail
  console.error(`No exact match for '${name}'. Did you mean:`);
  for (const [i, m] of matches.entries()) {
    console.error(`  ${i + 1}. ${m.name} (${(m.score * 100).toFixed(0)}%)`);
  }
  return null;
}

export async function resume(
  name: string,
  indexPath: string,
  config: Config,
): Promise<void> {
  const idx = await ProjectIndex.load(indexPath);

  const resolved = resolveName(name, idx, config);
  if (!resolved) {
    console.error(`Project '${name}' not found in index`);
    process.exit(1);
  }

  const ctx = await getResumeContext(resolved.path);
  if (!ctx) {
    console.error(
      `Project '${resolved.resolvedName}' directory not found: ${resolved.path}`,
    );
    process.exit(1);
  }

  // Record visit
  idx.touch(resolved.resolvedName);
  await idx.save(indexPath);

  // Print context to stderr (so stdout is just the path for shell cd)
  const dirty =
    ctx.dirtyCount > 0
      ? ` — ${ctx.dirtyCount} dirty ${ctx.dirtyCount === 1 ? "file" : "files"}`
      : "";
  console.error(`${resolved.resolvedName} (${ctx.branch})${dirty}`);
  if (ctx.lastCommit) {
    console.error(`  Last commit: ${ctx.lastCommit}`);
  }

  // Print path to stdout for shell cd
  console.log(resolved.path);
}
```

**Step 4: Wire into CLI**

In `src/index.ts`, add the import:

```typescript
import { resume } from "./commands/resume.ts";
```

Add this case in the switch, before `default`:

```typescript
    case "resume": {
      const name = args[1];
      if (!name) {
        console.error("Usage: gx resume <name>");
        process.exit(1);
      }
      await resume(name, indexPath, config);
      break;
    }
```

Update the help text to include:

```
  gx resume <name>       Jump to project with git context
```

**Step 5: Run tests to verify they pass**

Run: `bun test`
Expected: All pass.

**Step 6: Commit**

```bash
git add src/commands/resume.ts tests/commands/resume.test.ts src/index.ts
git commit -m "feat(tracking): add gx resume command"
```

---

### Task 8: Update shell integration

**Files:**

- Modify: `src/commands/shell-init.ts`
- Test: `tests/commands/shell-init.test.ts` (if exists, update; otherwise spot-check via `bun run dev -- shell-init`)

**Step 1: Update zsh shell function**

In the `zshInit()` function in `src/commands/shell-init.ts`:

1. Add `resume` to the passthrough case alongside `recent`:

Change the passthrough line:

```
ls|rebuild|config|open|init|shell-init|--help|-h|--version|-v)
```

to:

```
ls|recent|rebuild|config|open|init|shell-init|--help|-h|--version|-v)
```

2. Add a `resume` case before the `*` catch-all that works like `clone` (captures path from stdout, displays stderr context, cd's):

```bash
        resume)
            local output
            output=$("$_GX_BIN" resume "${@:2}" 2>&1 1>&3)
            local target
            target=$("$_GX_BIN" resume "${@:2}" 3>&1 1>/dev/null 2>&1)
            ;;
```

Actually, a simpler approach — since `resume` prints context to stderr and path to stdout (same pattern as `clone`):

Add `resume` case right after the `clone` case:

```bash
        resume)
            local output
            output=$("$_GX_BIN" resume "${@:2}")
            if [ -n "$output" ] && [ -d "$output" ]; then
                cd "$output"
            fi
            ;;
```

This works because `resume` prints the path to stdout (captured by `output`) and context to stderr (which passes through to the terminal).

3. Add `recent` and `resume` to the tab completion `commands` array.

Apply the same three changes to `bashInit()` and `fishInit()`.

**For bash** — same pattern: add `resume` case after `clone`, add `recent` to passthrough, update completion.

**For fish** — add `resume` case in the `switch`, add `recent` to passthrough, update completion:

```fish
        case resume
            set -l output ($_GX_BIN resume $argv[2..])
            if test -n "$output" -a -d "$output"
                cd "$output"
            end
```

**Step 2: Run existing shell-init tests**

Run: `bun test tests/commands/shell-init.test.ts` (if exists)
Otherwise: `bun run dev -- shell-init zsh` and visually verify `resume` and `recent` appear.

**Step 3: Run all tests**

Run: `bun test`
Expected: All pass.

**Step 4: Commit**

```bash
git add src/commands/shell-init.ts
git commit -m "feat(tracking): add recent and resume to shell integration"
```

---

### Task 9: Build and verify end-to-end

**Files:** None (verification only)

**Step 1: Type check**

Run: `bun x tsc --noEmit`
Expected: No errors.

**Step 2: Run full test suite**

Run: `bun test`
Expected: All pass.

**Step 3: Build binary**

Run: `bun run build`
Expected: Compiles successfully.

**Step 4: Smoke test**

```bash
./gx recent
./gx shell-init zsh | grep -E "recent|resume"
```

Expected: `recent` shows project list (or empty message), shell-init output contains both `recent` and `resume`.

**Step 5: Commit any fixes, then final commit if needed**

If everything is clean, no commit needed. If fixes were applied, commit them.

---

### Task 10: Update module spec status

**Files:**

- Modify: `plans/modules/09-tracking.aps.md`
- Modify: `plans/index.aps.md`

**Step 1: Update tracking module to Ready/Complete**

In `plans/modules/09-tracking.aps.md`, update the status table and check the ready checklist items.

In `plans/index.aps.md`, update the Tracking row status from "Draft" to "Complete" and check the v3 success criteria items for `gx recent` and `gx resume`.

**Step 2: Commit**

```bash
git add plans/modules/09-tracking.aps.md plans/index.aps.md
git commit -m "docs: update tracking module status to complete"
```
