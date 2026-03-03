import { test, expect, beforeEach, afterEach } from "bun:test";
import {
  loadConfig,
  saveConfig,
  getConfigPath,
  getAgent,
  effectiveProjectDir,
} from "../../src/lib/config.ts";
import { DEFAULT_CONFIG } from "../../src/types.ts";
import { join } from "path";
import { mkdtemp, rm } from "fs/promises";
import { tmpdir } from "os";

let tmpDir: string;
let savedGxAgent: string | undefined;

beforeEach(async () => {
  tmpDir = await mkdtemp(join(tmpdir(), "gx-test-"));
  savedGxAgent = process.env.GX_AGENT;
  delete process.env.GX_AGENT;
});

afterEach(async () => {
  if (savedGxAgent !== undefined) {
    process.env.GX_AGENT = savedGxAgent;
  } else {
    delete process.env.GX_AGENT;
  }
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

// getAgent() tests

test("getAgent returns null when GX_AGENT is not set", () => {
  expect(getAgent()).toBeNull();
});

test("getAgent returns lowercase agent name", () => {
  process.env.GX_AGENT = "Morgan";
  expect(getAgent()).toBe("morgan");
});

test("getAgent trims whitespace", () => {
  process.env.GX_AGENT = "  morgan  ";
  expect(getAgent()).toBe("morgan");
});

test("getAgent returns null for empty string", () => {
  process.env.GX_AGENT = "";
  expect(getAgent()).toBeNull();
});

test("getAgent returns null for whitespace-only", () => {
  process.env.GX_AGENT = "   ";
  expect(getAgent()).toBeNull();
});

test("getAgent accepts alphanumeric with hyphens", () => {
  process.env.GX_AGENT = "agent-007";
  expect(getAgent()).toBe("agent-007");
});

test("getAgent throws on invalid characters", () => {
  process.env.GX_AGENT = "bad_agent!";
  expect(() => getAgent()).toThrow("Invalid GX_AGENT");
});

test("getAgent throws on leading hyphen", () => {
  process.env.GX_AGENT = "-morgan";
  expect(() => getAgent()).toThrow("Invalid GX_AGENT");
});

test("getAgent throws on trailing hyphen", () => {
  process.env.GX_AGENT = "morgan-";
  expect(() => getAgent()).toThrow("Invalid GX_AGENT");
});

// effectiveProjectDir() tests

test("effectiveProjectDir returns base dir when no agent", () => {
  const config = { ...DEFAULT_CONFIG, projectDir: "/home/user/src" };
  expect(effectiveProjectDir(config)).toBe("/home/user/src");
});

test("effectiveProjectDir appends .agent when GX_AGENT set", () => {
  process.env.GX_AGENT = "morgan";
  const config = { ...DEFAULT_CONFIG, projectDir: "/home/user/src" };
  expect(effectiveProjectDir(config)).toBe("/home/user/src/.morgan");
});

test("effectiveProjectDir expands tilde", () => {
  const config = { ...DEFAULT_CONFIG, projectDir: "~/src" };
  const result = effectiveProjectDir(config);
  expect(result).not.toContain("~");
  expect(result).toMatch(/\/src$/);
});

test("effectiveProjectDir expands tilde with agent", () => {
  process.env.GX_AGENT = "morgan";
  const config = { ...DEFAULT_CONFIG, projectDir: "~/src" };
  const result = effectiveProjectDir(config);
  expect(result).not.toContain("~");
  expect(result).toMatch(/\/src\/\.morgan$/);
});
