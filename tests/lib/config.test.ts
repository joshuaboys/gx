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
