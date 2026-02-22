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
