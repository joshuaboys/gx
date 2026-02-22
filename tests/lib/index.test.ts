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
