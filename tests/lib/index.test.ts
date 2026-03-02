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

test("getRemoteUrl returns origin URL for a local repo with origin configured", async () => {
  const repoDir = join(tmpDir, "fixture-repo");
  await mkdir(repoDir, { recursive: true });

  let proc = Bun.spawn(["git", "init"], { cwd: repoDir, stdout: "ignore", stderr: "ignore" });
  await proc.exited;
  proc = Bun.spawn(["git", "remote", "add", "origin", "https://github.com/joshuaboys/gx.git"], {
    cwd: repoDir,
    stdout: "ignore",
    stderr: "ignore",
  });
  await proc.exited;

  const url = await ProjectIndex.getRemoteUrl(repoDir);
  expect(url).toBe("https://github.com/joshuaboys/gx.git");
});

test("getRemoteUrl returns empty string for non-repo", async () => {
  const url = await ProjectIndex.getRemoteUrl(tmpDir);
  expect(url).toBe("");
});

test("merge adds a new project and returns true", async () => {
  const idx = await ProjectIndex.load(indexPath);
  const isNew = idx.merge("newrepo", { path: "/tmp/newrepo", url: "https://github.com/user/newrepo", clonedAt: "2026-03-02T00:00:00Z" });
  expect(isNew).toBe(true);
  expect(idx.resolve("newrepo")).toBe("/tmp/newrepo");
});

test("merge skips when name and path already match", async () => {
  const idx = await ProjectIndex.load(indexPath);
  idx.add("myrepo", { path: "/tmp/myrepo", url: "", clonedAt: "" });
  const isNew = idx.merge("myrepo", { path: "/tmp/myrepo", url: "https://github.com/user/myrepo", clonedAt: "2026-03-02T00:00:00Z" });
  expect(isNew).toBe(false);
});

test("merge overwrites when name exists but path differs", async () => {
  const idx = await ProjectIndex.load(indexPath);
  idx.add("myrepo", { path: "/old/myrepo", url: "", clonedAt: "" });
  const isNew = idx.merge("myrepo", { path: "/new/myrepo", url: "", clonedAt: "" });
  expect(isNew).toBe(true);
  expect(idx.resolve("myrepo")).toBe("/new/myrepo");
});

test("scanForRepos populates remote URL when available", async () => {
  const repoDir = join(tmpDir, "testorg", "gx");
  await mkdir(repoDir, { recursive: true });

  let proc = Bun.spawn(["git", "init"], { cwd: repoDir, stdout: "ignore", stderr: "ignore" });
  await proc.exited;
  proc = Bun.spawn(["git", "remote", "add", "origin", "https://github.com/joshuaboys/gx.git"], {
    cwd: repoDir,
    stdout: "ignore",
    stderr: "ignore",
  });
  await proc.exited;
  await mkdir(join(repoDir, ".git"), { recursive: true });

  const idx = await ProjectIndex.load(indexPath);
  await idx.rebuild(tmpDir);
  const entries = idx.list();
  const gxEntry = entries.find((e) => e.name === "gx");
  expect(gxEntry).toBeDefined();
  expect(gxEntry!.url).toContain("joshuaboys/gx");
});

test("additiveScan adds new repos without removing existing", async () => {
  const idx = await ProjectIndex.load(indexPath);
  // Pre-populate with an entry that won't be found by scan
  idx.add("external", { path: "/external/repo", url: "", clonedAt: "" });

  // Create fake repos to discover
  await mkdir(join(tmpDir, "org", "repoA", ".git"), { recursive: true });
  await mkdir(join(tmpDir, "org", "repoB", ".git"), { recursive: true });

  await idx.additiveScan(tmpDir);

  // External entry is preserved
  expect(idx.resolve("external")).toBe("/external/repo");
  // New repos are added
  expect(idx.resolve("repoA")).toBe(join(tmpDir, "org", "repoA"));
  expect(idx.resolve("repoB")).toBe(join(tmpDir, "org", "repoB"));
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
