import { test, expect, describe, beforeEach, afterEach } from "bun:test";
import { expandTilde, loadConfig } from "../../src/lib/config.ts";
import { DEFAULT_CONFIG } from "../../src/types.ts";
import { homedir } from "os";
import { join } from "path";
import { mkdtemp, rm } from "fs/promises";
import { tmpdir } from "os";

describe("expandTilde", () => {
  test("replaces ~/ prefix with homedir", () => {
    const result = expandTilde("~/Projects/src");
    expect(result).toBe(join(homedir(), "Projects/src"));
  });

  test("replaces bare ~ with homedir", () => {
    expect(expandTilde("~")).toBe(homedir());
  });

  test("leaves absolute paths unchanged", () => {
    expect(expandTilde("/usr/local/bin")).toBe("/usr/local/bin");
  });

  test("leaves relative paths unchanged", () => {
    expect(expandTilde("relative/path")).toBe("relative/path");
  });
});

describe("loadConfig validation", () => {
  let tmpDir: string;

  beforeEach(async () => {
    tmpDir = await mkdtemp(join(tmpdir(), "gx-config-val-"));
  });

  afterEach(async () => {
    await rm(tmpDir, { recursive: true });
  });

  test("returns defaults for invalid JSON", async () => {
    const configPath = join(tmpDir, "bad.json");
    await Bun.write(configPath, "{ not valid json }");
    const config = await loadConfig(configPath);
    expect(config).toEqual(DEFAULT_CONFIG);
  });

  test("rejects invalid structure value and uses default", async () => {
    const configPath = join(tmpDir, "config.json");
    await Bun.write(configPath, JSON.stringify({ structure: "invalid" }));
    const config = await loadConfig(configPath);
    expect(config.structure).toBe("flat");
  });

  test("rejects non-number similarityThreshold and uses default", async () => {
    const configPath = join(tmpDir, "config.json");
    await Bun.write(configPath, JSON.stringify({ similarityThreshold: "not-a-number" }));
    const config = await loadConfig(configPath);
    expect(config.similarityThreshold).toBe(0.7);
  });

  test("rejects out-of-range similarityThreshold and uses default", async () => {
    const configPath = join(tmpDir, "config.json");
    await Bun.write(configPath, JSON.stringify({ similarityThreshold: 5.0 }));
    const config = await loadConfig(configPath);
    expect(config.similarityThreshold).toBe(0.7);
  });

  test("rejects non-boolean shallow and uses default", async () => {
    const configPath = join(tmpDir, "config.json");
    await Bun.write(configPath, JSON.stringify({ shallow: "yes" }));
    const config = await loadConfig(configPath);
    expect(config.shallow).toBe(false);
  });

  test("rejects empty projectDir and uses default", async () => {
    const configPath = join(tmpDir, "config.json");
    await Bun.write(configPath, JSON.stringify({ projectDir: "" }));
    const config = await loadConfig(configPath);
    expect(config.projectDir).toBe("~/Projects/src");
  });

  test("accepts valid config values", async () => {
    const configPath = join(tmpDir, "config.json");
    const custom = {
      projectDir: "/custom/path",
      defaultHost: "gitlab.com",
      structure: "host",
      shallow: true,
      similarityThreshold: 0.8,
      editor: "vim",
    };
    await Bun.write(configPath, JSON.stringify(custom));
    const config = await loadConfig(configPath);
    expect(config).toEqual(custom);
  });
});
