<!-- Archived: 2026-02-24 | Reason: v1 build complete — historical record of initial construction -->

# gx Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a TypeScript/Bun CLI tool that clones git repos into organized directories and lets you jump between projects by name.

**Architecture:** Compiled Bun binary handles cloning, URL parsing, index, and config. A thin zsh plugin wraps the binary to provide shell `cd` and tab completion. Binary outputs paths to stdout; plugin does the `cd`.

**Tech Stack:** TypeScript, Bun (runtime + test + compile), zsh

---

### Task 1: Project structure and types

**Files:**
- Create: `src/types.ts`
- Modify: `package.json`
- Move: `index.ts` → `src/index.ts`
- Test: `tests/types.test.ts`

**Step 1: Update package.json with bin entry and scripts**

```json
{
  "name": "gx",
  "version": "0.1.0",
  "module": "src/index.ts",
  "type": "module",
  "private": true,
  "bin": {
    "gx": "src/index.ts"
  },
  "scripts": {
    "dev": "bun run src/index.ts",
    "build": "bun build src/index.ts --compile --outfile gx",
    "test": "bun test"
  },
  "devDependencies": {
    "@types/bun": "latest"
  },
  "peerDependencies": {
    "typescript": "^5"
  }
}
```

**Step 2: Create src/types.ts with all shared types**

```ts
export interface Config {
  projectDir: string;
  defaultHost: string;
  structure: "flat" | "host";
  shallow: boolean;
}

export const DEFAULT_CONFIG: Config = {
  projectDir: "~/Projects/src",
  defaultHost: "github.com",
  structure: "flat",
  shallow: false,
};

export interface ParsedRepo {
  host: string;
  owner: string;
  repo: string;
  originalUrl: string;
}

export interface IndexEntry {
  path: string;
  url: string;
  clonedAt: string;
}

export interface Index {
  projects: Record<string, IndexEntry>;
}
```

**Step 3: Write a simple type validation test**

```ts
// tests/types.test.ts
import { test, expect } from "bun:test";
import { DEFAULT_CONFIG } from "../src/types.ts";

test("DEFAULT_CONFIG has expected values", () => {
  expect(DEFAULT_CONFIG.defaultHost).toBe("github.com");
  expect(DEFAULT_CONFIG.structure).toBe("flat");
  expect(DEFAULT_CONFIG.shallow).toBe(false);
});
```

**Step 4: Move index.ts to src/ and make it a placeholder entry point**

```ts
// src/index.ts
#!/usr/bin/env bun
console.log("gx v0.1.0");
```

**Step 5: Run tests**

Run: `bun test`
Expected: PASS

**Step 6: Commit**

```bash
git add src/ tests/ package.json
git rm index.ts
git commit -m "feat: project structure, types, and build config"
```

---

### Task 2: Config module

**Files:**
- Create: `src/lib/config.ts`
- Test: `tests/lib/config.test.ts`

**Step 1: Write failing tests for config**

```ts
// tests/lib/config.test.ts
import { test, expect, beforeEach, afterEach } from "bun:test";
import { loadConfig, saveConfig, getConfigPath } from "../../src/lib/config.ts";
import { DEFAULT_CONFIG } from "../../src/types.ts";
import { join } from "path";
import { mkdtemp, rm } from "fs/promises";
import { tmpdir } from "os";

let tmpDir: string;

beforeEach(async () => {
  tmpDir = await mkdtemp(join(tmpdir(), "gx-test-"));
});

afterEach(async () => {
  await rm(tmpDir, { recursive: true });
});

test("loadConfig returns defaults when no config file exists", async () => {
  const config = await loadConfig(join(tmpDir, "config.json"));
  expect(config).toEqual(DEFAULT_CONFIG);
});

test("saveConfig writes and loadConfig reads back", async () => {
  const configPath = join(tmpDir, "config.json");
  const custom = { ...DEFAULT_CONFIG, structure: "host" as const };
  await saveConfig(configPath, custom);
  const loaded = await loadConfig(configPath);
  expect(loaded.structure).toBe("host");
});

test("loadConfig merges partial config with defaults", async () => {
  const configPath = join(tmpDir, "config.json");
  await Bun.write(configPath, JSON.stringify({ shallow: true }));
  const config = await loadConfig(configPath);
  expect(config.shallow).toBe(true);
  expect(config.defaultHost).toBe("github.com");
});

test("getConfigPath returns ~/.config/gx/config.json", () => {
  const p = getConfigPath();
  expect(p).toContain(".config/gx/config.json");
});
```

**Step 2: Run tests to verify they fail**

Run: `bun test tests/lib/config.test.ts`
Expected: FAIL — module not found

**Step 3: Implement config module**

```ts
// src/lib/config.ts
import { join } from "path";
import { homedir } from "os";
import { mkdir } from "fs/promises";
import type { Config } from "../types.ts";
import { DEFAULT_CONFIG } from "../types.ts";

export function getConfigPath(): string {
  return join(homedir(), ".config", "gx", "config.json");
}

export function expandTilde(p: string): string {
  if (p.startsWith("~/") || p === "~") {
    return join(homedir(), p.slice(1));
  }
  return p;
}

export async function loadConfig(path?: string): Promise<Config> {
  const configPath = path ?? getConfigPath();
  try {
    const file = Bun.file(configPath);
    const raw = await file.json();
    return { ...DEFAULT_CONFIG, ...raw };
  } catch {
    return { ...DEFAULT_CONFIG };
  }
}

export async function saveConfig(
  path: string,
  config: Config
): Promise<void> {
  await mkdir(join(path, ".."), { recursive: true });
  await Bun.write(path, JSON.stringify(config, null, 2) + "\n");
}
```

**Step 4: Run tests**

Run: `bun test tests/lib/config.test.ts`
Expected: PASS

**Step 5: Commit**

```bash
git add src/lib/config.ts tests/lib/config.test.ts
git commit -m "feat: config module with load, save, and defaults"
```

---

### Task 3: URL parsing module

**Files:**
- Create: `src/lib/url.ts`
- Test: `tests/lib/url.test.ts`

**Step 1: Write failing tests for URL parsing**

```ts
// tests/lib/url.test.ts
import { test, expect } from "bun:test";
import { parseUrl } from "../../src/lib/url.ts";

// Shorthand
test("parses shorthand user/repo", () => {
  const result = parseUrl("juev/gclone");
  expect(result.host).toBe("github.com");
  expect(result.owner).toBe("juev");
  expect(result.repo).toBe("gclone");
});

// HTTPS
test("parses HTTPS URL", () => {
  const result = parseUrl("https://github.com/juev/gclone.git");
  expect(result.host).toBe("github.com");
  expect(result.owner).toBe("juev");
  expect(result.repo).toBe("gclone");
});

test("parses HTTPS URL without .git suffix", () => {
  const result = parseUrl("https://github.com/juev/gclone");
  expect(result.host).toBe("github.com");
  expect(result.owner).toBe("juev");
  expect(result.repo).toBe("gclone");
});

test("parses HTTPS URL with trailing slash", () => {
  const result = parseUrl("https://github.com/juev/gclone/");
  expect(result.repo).toBe("gclone");
});

// SSH
test("parses SSH URL (git@host:user/repo)", () => {
  const result = parseUrl("git@github.com:juev/gclone.git");
  expect(result.host).toBe("github.com");
  expect(result.owner).toBe("juev");
  expect(result.repo).toBe("gclone");
});

test("parses SSH URL without .git suffix", () => {
  const result = parseUrl("git@gitlab.com:company/project");
  expect(result.host).toBe("gitlab.com");
  expect(result.owner).toBe("company");
  expect(result.repo).toBe("project");
});

// Git protocol
test("parses git:// URL", () => {
  const result = parseUrl("git://github.com/juev/gclone.git");
  expect(result.host).toBe("github.com");
  expect(result.owner).toBe("juev");
  expect(result.repo).toBe("gclone");
});

// Nested paths
test("parses URL with nested path (gitlab groups)", () => {
  const result = parseUrl("https://gitlab.com/group/subgroup/repo.git");
  expect(result.host).toBe("gitlab.com");
  expect(result.owner).toBe("group/subgroup");
  expect(result.repo).toBe("repo");
});

// Errors
test("throws on empty input", () => {
  expect(() => parseUrl("")).toThrow();
});

test("throws on invalid URL", () => {
  expect(() => parseUrl("not a url at all")).toThrow();
});

test("throws on URL with path traversal", () => {
  expect(() => parseUrl("https://github.com/../etc/passwd")).toThrow();
});
```

**Step 2: Run tests to verify they fail**

Run: `bun test tests/lib/url.test.ts`
Expected: FAIL

**Step 3: Implement URL parser**

```ts
// src/lib/url.ts
import type { ParsedRepo } from "../types.ts";

const HTTPS_RE = /^https?:\/\/([^/]+)\/(.+?)(?:\.git)?\/?$/;
const SSH_RE = /^(?:ssh:\/\/)?[^@]+@([^/:]+)(?::\d+)?[:/](.+?)(?:\.git)?\/?$/;
const GIT_RE = /^git:\/\/([^/]+)\/(.+?)(?:\.git)?\/?$/;
const SHORTHAND_RE = /^([a-zA-Z0-9_.-]+)\/([a-zA-Z0-9_.-][a-zA-Z0-9_.\-/]*)$/;

export function parseUrl(input: string, defaultHost = "github.com"): ParsedRepo {
  const trimmed = input.trim();
  if (!trimmed) throw new Error("Empty repository URL");
  if (trimmed.includes("..")) throw new Error("Path traversal detected");

  let host: string;
  let path: string;

  // Try each pattern
  let match = trimmed.match(HTTPS_RE);
  if (match) {
    [, host, path] = match;
    return buildParsed(host!, path!, trimmed);
  }

  match = trimmed.match(GIT_RE);
  if (match) {
    [, host, path] = match;
    return buildParsed(host!, path!, trimmed);
  }

  match = trimmed.match(SSH_RE);
  if (match) {
    [, host, path] = match;
    return buildParsed(host!, path!, trimmed);
  }

  match = trimmed.match(SHORTHAND_RE);
  if (match) {
    const [, owner, repo] = match;
    return {
      host: defaultHost,
      owner: owner!,
      repo: repo!.replace(/\/$/, ""),
      originalUrl: `https://${defaultHost}/${owner}/${repo}`,
    };
  }

  throw new Error(`Cannot parse repository URL: ${trimmed}`);
}

function buildParsed(host: string, path: string, originalUrl: string): ParsedRepo {
  const segments = path.split("/");
  if (segments.length < 2) {
    throw new Error(`Invalid repository path: ${path}`);
  }
  const repo = segments.pop()!;
  const owner = segments.join("/");
  return { host, owner, repo, originalUrl };
}

export function toCloneUrl(parsed: ParsedRepo): string {
  return parsed.originalUrl.startsWith("git@")
    ? parsed.originalUrl
    : `https://${parsed.host}/${parsed.owner}/${parsed.repo}.git`;
}
```

**Step 4: Run tests**

Run: `bun test tests/lib/url.test.ts`
Expected: PASS

**Step 5: Commit**

```bash
git add src/lib/url.ts tests/lib/url.test.ts
git commit -m "feat: URL parsing for HTTPS, SSH, git://, and shorthand formats"
```

---

### Task 4: Path mapping module

**Files:**
- Create: `src/lib/path.ts`
- Test: `tests/lib/path.test.ts`

**Step 1: Write failing tests for path mapping**

```ts
// tests/lib/path.test.ts
import { test, expect } from "bun:test";
import { toPath } from "../../src/lib/path.ts";
import type { ParsedRepo, Config } from "../../src/types.ts";
import { DEFAULT_CONFIG } from "../../src/types.ts";

const repo: ParsedRepo = {
  host: "github.com",
  owner: "juev",
  repo: "gclone",
  originalUrl: "https://github.com/juev/gclone",
};

test("flat structure: owner/repo", () => {
  const config: Config = { ...DEFAULT_CONFIG, projectDir: "/home/user/src" };
  expect(toPath(repo, config)).toBe("/home/user/src/juev/gclone");
});

test("host structure: host/owner/repo", () => {
  const config: Config = {
    ...DEFAULT_CONFIG,
    projectDir: "/home/user/src",
    structure: "host",
  };
  expect(toPath(repo, config)).toBe("/home/user/src/github.com/juev/gclone");
});

test("nested owner preserves path", () => {
  const nested: ParsedRepo = {
    ...repo,
    owner: "group/subgroup",
    repo: "project",
  };
  const config: Config = { ...DEFAULT_CONFIG, projectDir: "/home/user/src" };
  expect(toPath(nested, config)).toBe("/home/user/src/group/subgroup/project");
});

test("tilde expansion in projectDir", () => {
  const config: Config = { ...DEFAULT_CONFIG, projectDir: "~/src" };
  const result = toPath(repo, config);
  expect(result).not.toContain("~");
  expect(result).toContain("/juev/gclone");
});
```

**Step 2: Run tests to verify they fail**

Run: `bun test tests/lib/path.test.ts`
Expected: FAIL

**Step 3: Implement path module**

```ts
// src/lib/path.ts
import { join } from "path";
import type { ParsedRepo, Config } from "../types.ts";
import { expandTilde } from "./config.ts";

export function toPath(parsed: ParsedRepo, config: Config): string {
  const base = expandTilde(config.projectDir);
  if (config.structure === "host") {
    return join(base, parsed.host, parsed.owner, parsed.repo);
  }
  return join(base, parsed.owner, parsed.repo);
}
```

**Step 4: Run tests**

Run: `bun test tests/lib/path.test.ts`
Expected: PASS

**Step 5: Commit**

```bash
git add src/lib/path.ts tests/lib/path.test.ts
git commit -m "feat: path mapping with flat and host directory structures"
```

---

### Task 5: Index module

**Files:**
- Create: `src/lib/index.ts`
- Test: `tests/lib/index.test.ts`

**Step 1: Write failing tests for index**

```ts
// tests/lib/index.test.ts
import { test, expect, beforeEach, afterEach } from "bun:test";
import { ProjectIndex } from "../../src/lib/index.ts";
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

test("load returns empty index when file missing", async () => {
  const idx = await ProjectIndex.load(indexPath);
  expect(idx.list()).toEqual([]);
});

test("add and resolve a project", async () => {
  const idx = await ProjectIndex.load(indexPath);
  idx.add("gx", { path: "/home/user/src/joshuaboys/gx", url: "https://github.com/joshuaboys/gx", clonedAt: "2026-02-23T00:00:00Z" });
  expect(idx.resolve("gx")).toBe("/home/user/src/joshuaboys/gx");
});

test("resolve returns null for unknown project", async () => {
  const idx = await ProjectIndex.load(indexPath);
  expect(idx.resolve("nope")).toBeNull();
});

test("save persists and load reads back", async () => {
  const idx = await ProjectIndex.load(indexPath);
  idx.add("gx", { path: "/tmp/gx", url: "https://github.com/joshuaboys/gx", clonedAt: "2026-02-23T00:00:00Z" });
  await idx.save(indexPath);

  const idx2 = await ProjectIndex.load(indexPath);
  expect(idx2.resolve("gx")).toBe("/tmp/gx");
});

test("list returns all entries sorted", async () => {
  const idx = await ProjectIndex.load(indexPath);
  idx.add("bravo", { path: "/tmp/bravo", url: "", clonedAt: "" });
  idx.add("alpha", { path: "/tmp/alpha", url: "", clonedAt: "" });
  const names = idx.list().map((e) => e.name);
  expect(names).toEqual(["alpha", "bravo"]);
});

test("names returns project names for completion", async () => {
  const idx = await ProjectIndex.load(indexPath);
  idx.add("gx", { path: "/tmp/gx", url: "", clonedAt: "" });
  idx.add("gclone", { path: "/tmp/gclone", url: "", clonedAt: "" });
  expect(idx.names()).toContain("gx");
  expect(idx.names()).toContain("gclone");
});

test("rebuild scans directory for git repos", async () => {
  // Create fake repos
  const repoA = join(tmpDir, "user", "repoA", ".git");
  const repoB = join(tmpDir, "user", "repoB", ".git");
  await mkdir(repoA, { recursive: true });
  await mkdir(repoB, { recursive: true });

  const idx = await ProjectIndex.load(indexPath);
  await idx.rebuild(tmpDir);
  expect(idx.resolve("repoA")).toBe(join(tmpDir, "user", "repoA"));
  expect(idx.resolve("repoB")).toBe(join(tmpDir, "user", "repoB"));
});
```

**Step 2: Run tests to verify they fail**

Run: `bun test tests/lib/index.test.ts`
Expected: FAIL

**Step 3: Implement index module**

```ts
// src/lib/index.ts
import { join } from "path";
import { mkdir } from "fs/promises";
import { Glob } from "bun";
import type { Index, IndexEntry } from "../types.ts";

export class ProjectIndex {
  private data: Index;

  private constructor(data: Index) {
    this.data = data;
  }

  static async load(path: string): Promise<ProjectIndex> {
    try {
      const file = Bun.file(path);
      const raw = await file.json();
      return new ProjectIndex(raw as Index);
    } catch {
      return new ProjectIndex({ projects: {} });
    }
  }

  add(name: string, entry: IndexEntry): void {
    this.data.projects[name] = entry;
  }

  resolve(name: string): string | null {
    return this.data.projects[name]?.path ?? null;
  }

  list(): Array<{ name: string } & IndexEntry> {
    return Object.entries(this.data.projects)
      .map(([name, entry]) => ({ name, ...entry }))
      .sort((a, b) => a.name.localeCompare(b.name));
  }

  names(): string[] {
    return Object.keys(this.data.projects).sort();
  }

  async rebuild(projectDir: string): Promise<void> {
    this.data.projects = {};
    const glob = new Glob("**/.git");
    for await (const match of glob.scan({ cwd: projectDir, onlyFiles: false })) {
      const repoPath = join(projectDir, match.replace(/\/.git$/, ""));
      const name = repoPath.split("/").pop()!;
      this.data.projects[name] = {
        path: repoPath,
        url: "",
        clonedAt: "",
      };
    }
  }

  async save(path: string): Promise<void> {
    await mkdir(join(path, ".."), { recursive: true });
    await Bun.write(path, JSON.stringify(this.data, null, 2) + "\n");
  }
}
```

**Step 4: Run tests**

Run: `bun test tests/lib/index.test.ts`
Expected: PASS

**Step 5: Commit**

```bash
git add src/lib/index.ts tests/lib/index.test.ts
git commit -m "feat: project index with add, resolve, list, rebuild"
```

---

### Task 6: Clone command

**Files:**
- Create: `src/commands/clone.ts`
- Test: `tests/commands/clone.test.ts`

**Step 1: Write failing tests for clone**

```ts
// tests/commands/clone.test.ts
import { test, expect, beforeEach, afterEach } from "bun:test";
import { cloneRepo } from "../../src/commands/clone.ts";
import { join } from "path";
import { mkdtemp, rm } from "fs/promises";
import { tmpdir } from "os";
import type { Config } from "../../src/types.ts";
import { DEFAULT_CONFIG } from "../../src/types.ts";

let tmpDir: string;

beforeEach(async () => {
  tmpDir = await mkdtemp(join(tmpdir(), "gx-test-"));
});

afterEach(async () => {
  await rm(tmpDir, { recursive: true });
});

test("cloneRepo clones a real repo and returns path", async () => {
  const config: Config = { ...DEFAULT_CONFIG, projectDir: tmpDir };
  const indexPath = join(tmpDir, "index.json");
  const result = await cloneRepo("joshuaboys/gx", config, indexPath);
  expect(result).toBe(join(tmpDir, "joshuaboys", "gx"));
  // Verify .git exists
  const gitDir = Bun.file(join(result, ".git", "HEAD"));
  expect(await gitDir.exists()).toBe(true);
}, 30_000);

test("cloneRepo skips already cloned repo", async () => {
  const config: Config = { ...DEFAULT_CONFIG, projectDir: tmpDir };
  const indexPath = join(tmpDir, "index.json");
  const first = await cloneRepo("joshuaboys/gx", config, indexPath);
  const second = await cloneRepo("joshuaboys/gx", config, indexPath);
  expect(second).toBe(first);
}, 30_000);
```

**Step 2: Run tests to verify they fail**

Run: `bun test tests/commands/clone.test.ts`
Expected: FAIL

**Step 3: Implement clone command**

```ts
// src/commands/clone.ts
import { join } from "path";
import { mkdir } from "fs/promises";
import { parseUrl, toCloneUrl } from "../lib/url.ts";
import { toPath } from "../lib/path.ts";
import { ProjectIndex } from "../lib/index.ts";
import type { Config } from "../types.ts";

export async function cloneRepo(
  input: string,
  config: Config,
  indexPath: string
): Promise<string> {
  const parsed = parseUrl(input, config.defaultHost);
  const targetDir = toPath(parsed, config);

  // Skip if already exists
  const gitHead = Bun.file(join(targetDir, ".git", "HEAD"));
  if (await gitHead.exists()) {
    console.error(`already exists: ${targetDir}`);
    return targetDir;
  }

  // Create parent directory
  await mkdir(join(targetDir, ".."), { recursive: true });

  // Clone
  const cloneUrl = toCloneUrl(parsed);
  const args = ["clone"];
  if (config.shallow) args.push("--depth=1");
  args.push(cloneUrl, targetDir);

  const proc = Bun.spawn(["git", ...args], {
    stdout: "inherit",
    stderr: "inherit",
  });
  const exitCode = await proc.exited;
  if (exitCode !== 0) {
    throw new Error(`git clone failed with exit code ${exitCode}`);
  }

  // Update index
  const idx = await ProjectIndex.load(indexPath);
  idx.add(parsed.repo, {
    path: targetDir,
    url: cloneUrl,
    clonedAt: new Date().toISOString(),
  });
  await idx.save(indexPath);

  return targetDir;
}
```

**Step 4: Run tests**

Run: `bun test tests/commands/clone.test.ts`
Expected: PASS (clones real repo — needs network)

**Step 5: Commit**

```bash
git add src/commands/clone.ts tests/commands/clone.test.ts
git commit -m "feat: clone command with index auto-update"
```

---

### Task 7: CLI entry point and remaining commands

**Files:**
- Modify: `src/index.ts`
- Create: `src/commands/ls.ts`
- Create: `src/commands/resolve.ts`
- Create: `src/commands/rebuild.ts`
- Create: `src/commands/config.ts`

**Step 1: Implement ls command**

```ts
// src/commands/ls.ts
import { ProjectIndex } from "../lib/index.ts";

export async function ls(indexPath: string): Promise<void> {
  const idx = await ProjectIndex.load(indexPath);
  const entries = idx.list();
  if (entries.length === 0) {
    console.error("No projects indexed. Clone a repo or run 'gx rebuild'.");
    return;
  }
  const maxName = Math.max(...entries.map((e) => e.name.length));
  for (const entry of entries) {
    console.log(`${entry.name.padEnd(maxName)}  ${entry.path}`);
  }
}
```

**Step 2: Implement resolve command**

```ts
// src/commands/resolve.ts
import { ProjectIndex } from "../lib/index.ts";

export async function resolve(
  name: string,
  indexPath: string,
  listAll = false
): Promise<void> {
  const idx = await ProjectIndex.load(indexPath);
  if (listAll) {
    console.log(idx.names().join("\n"));
    return;
  }
  const path = idx.resolve(name);
  if (path) {
    console.log(path);
  } else {
    console.error(`Project '${name}' not found`);
    process.exit(1);
  }
}
```

**Step 3: Implement rebuild command**

```ts
// src/commands/rebuild.ts
import { ProjectIndex } from "../lib/index.ts";
import { expandTilde } from "../lib/config.ts";
import type { Config } from "../types.ts";

export async function rebuild(
  config: Config,
  indexPath: string
): Promise<void> {
  const projectDir = expandTilde(config.projectDir);
  const idx = await ProjectIndex.load(indexPath);
  await idx.rebuild(projectDir);
  await idx.save(indexPath);
  const count = idx.list().length;
  console.error(`Indexed ${count} projects`);
}
```

**Step 4: Implement config command**

```ts
// src/commands/config.ts
import { loadConfig, saveConfig } from "../lib/config.ts";
import type { Config } from "../types.ts";

export async function showConfig(configPath: string): Promise<void> {
  const config = await loadConfig(configPath);
  console.log(JSON.stringify(config, null, 2));
}

export async function setConfig(
  configPath: string,
  key: string,
  value: string
): Promise<void> {
  const config = await loadConfig(configPath);
  if (!(key in config)) {
    console.error(`Unknown config key: ${key}`);
    console.error(`Valid keys: ${Object.keys(config).join(", ")}`);
    process.exit(1);
  }
  const k = key as keyof Config;
  if (typeof config[k] === "boolean") {
    (config as Record<string, unknown>)[key] = value === "true";
  } else {
    (config as Record<string, unknown>)[key] = value;
  }
  await saveConfig(configPath, config);
  console.error(`Set ${key} = ${value}`);
}
```

**Step 5: Wire up CLI entry point**

```ts
// src/index.ts
#!/usr/bin/env bun
import { join } from "path";
import { homedir } from "os";
import { loadConfig, getConfigPath } from "./lib/config.ts";
import { cloneRepo } from "./commands/clone.ts";
import { ls } from "./commands/ls.ts";
import { resolve } from "./commands/resolve.ts";
import { rebuild } from "./commands/rebuild.ts";
import { showConfig, setConfig } from "./commands/config.ts";

const VERSION = "0.1.0";

function getIndexPath(): string {
  return join(homedir(), ".config", "gx", "index.json");
}

async function main() {
  const args = process.argv.slice(2);
  const command = args[0];

  if (!command || command === "--help" || command === "-h") {
    console.log(`gx v${VERSION} — git project manager

Usage:
  gx <name>                Jump to project
  gx clone <repo>          Clone and jump to repo
  gx ls                    List indexed projects
  gx rebuild               Rescan and rebuild index
  gx config                Show config
  gx config set <key> <v>  Set config value
  gx resolve <name>        Resolve project name to path
  gx resolve --list        List all project names`);
    return;
  }

  if (command === "--version" || command === "-v") {
    console.log(`gx v${VERSION}`);
    return;
  }

  const configPath = getConfigPath();
  const indexPath = getIndexPath();
  const config = await loadConfig(configPath);

  switch (command) {
    case "clone": {
      const repo = args[1];
      if (!repo) {
        console.error("Usage: gx clone <repo>");
        process.exit(1);
      }
      const path = await cloneRepo(repo, config, indexPath);
      console.log(path);
      break;
    }
    case "ls":
      await ls(indexPath);
      break;
    case "rebuild":
      await rebuild(config, indexPath);
      break;
    case "config":
      if (args[1] === "set" && args[2] && args[3]) {
        await setConfig(configPath, args[2], args[3]);
      } else {
        await showConfig(configPath);
      }
      break;
    case "resolve":
      if (args[1] === "--list") {
        await resolve("", indexPath, true);
      } else if (args[1]) {
        await resolve(args[1], indexPath);
      } else {
        console.error("Usage: gx resolve <name> | gx resolve --list");
        process.exit(1);
      }
      break;
    default:
      // Default: treat as project name to resolve
      await resolve(command, indexPath);
      break;
  }
}

main().catch((err) => {
  console.error(err.message);
  process.exit(1);
});
```

**Step 6: Run full test suite**

Run: `bun test`
Expected: All tests PASS

**Step 7: Commit**

```bash
git add src/index.ts src/commands/
git commit -m "feat: CLI entry point with clone, ls, resolve, rebuild, config commands"
```

---

### Task 8: Zsh plugin

**Files:**
- Create: `plugin/gx.plugin.zsh`

**Step 1: Write the zsh plugin**

```zsh
# plugin/gx.plugin.zsh
# gx — git project manager shell integration
# Install: symlink or copy to ~/.oh-my-zsh/custom/plugins/gx/

# Find the gx binary — compiled binary or bun dev
_gx_bin() {
    if command -v gx &>/dev/null; then
        echo "gx"
    elif [ -x "${0:A:h}/../../gx" ]; then
        echo "${0:A:h}/../../gx"
    else
        echo "bun run ${0:A:h}/../../src/index.ts"
    fi
}

gx() {
    local bin
    bin=$(_gx_bin)

    case "$1" in
        clone)
            local output
            output=$(eval "$bin" clone "${@:2}" 2>/dev/null)
            if [ -n "$output" ] && [ -d "$output" ]; then
                cd "$output"
            else
                eval "$bin" clone "${@:2}"
            fi
            ;;
        ls|rebuild|config|--help|-h|--version|-v)
            eval "$bin" "$@"
            ;;
        resolve)
            eval "$bin" "$@"
            ;;
        "")
            eval "$bin" --help
            ;;
        *)
            # Default: jump to project
            local target
            target=$(eval "$bin" resolve "$1" 2>/dev/null)
            if [ -n "$target" ] && [ -d "$target" ]; then
                cd "$target"
            else
                echo "Project '$1' not found. Run 'gx rebuild' to update index."
                return 1
            fi
            ;;
    esac
}

# Tab completion
_gx() {
    local bin
    bin=$(_gx_bin)

    local -a commands projects
    commands=(clone ls rebuild config resolve --help --version)

    if (( CURRENT == 2 )); then
        # First arg: commands + project names
        projects=($(eval "$bin" resolve --list 2>/dev/null))
        compadd "${commands[@]}" "${projects[@]}"
    elif (( CURRENT == 3 )) && [[ "${words[2]}" == "config" ]]; then
        compadd set
    elif (( CURRENT == 4 )) && [[ "${words[2]}" == "config" ]] && [[ "${words[3]}" == "set" ]]; then
        compadd projectDir defaultHost structure shallow
    fi
}
compdef _gx gx
```

**Step 2: Test manually**

Run: `source plugin/gx.plugin.zsh && gx --help`
Expected: Shows help text

**Step 3: Commit**

```bash
git add plugin/
git commit -m "feat: zsh plugin with cd integration and tab completion"
```

---

### Task 9: Build, test end-to-end, and install

**Step 1: Run full test suite**

Run: `bun test`
Expected: All PASS

**Step 2: Build compiled binary**

Run: `bun build src/index.ts --compile --outfile gx`
Expected: Produces `./gx` binary

**Step 3: Test compiled binary**

Run: `./gx --version`
Expected: `gx v0.1.0`

Run: `./gx --help`
Expected: Shows usage

**Step 4: Add gx binary to .gitignore**

Append `/gx` to `.gitignore` (the compiled binary shouldn't be committed).

**Step 5: Install the zsh plugin**

```bash
# Symlink plugin into oh-my-zsh custom plugins
ln -sf "$(pwd)/plugin" ~/.oh-my-zsh/custom/plugins/gx
```

**Step 6: Update .zshrc — replace old proj plugin with gx**

In `~/.zshrc`, change `plugins=(git proj)` to `plugins=(git gx)`. Remove the old proj-related entries (PROJ_INDEX, GIT_PROJECT_DIR export can stay since gx uses config file).

**Step 7: Test in new shell**

```bash
source ~/.zshrc
gx rebuild
gx ls
gx gx       # should cd into the gx project
```

**Step 8: Commit**

```bash
git add .gitignore
git commit -m "chore: add compiled binary to gitignore"
```

**Step 9: Push**

```bash
git push
```
