import { test, expect, describe, beforeEach, afterEach } from "bun:test";
import { setConfig } from "../../src/commands/config.ts";
import { loadConfig } from "../../src/lib/config.ts";
import { CommandError } from "../../src/lib/errors.ts";
import { join } from "path";
import { mkdtemp, rm } from "fs/promises";
import { tmpdir } from "os";

let tmpDir: string;

beforeEach(async () => {
  tmpDir = await mkdtemp(join(tmpdir(), "gx-config-"));
});

afterEach(async () => {
  await rm(tmpDir, { recursive: true });
});

describe("setConfig", () => {
  test("stores a string field", async () => {
    const configPath = join(tmpDir, "config.json");
    await setConfig(configPath, "defaultHost", "gitlab.com");
    const config = await loadConfig(configPath);
    expect(config.defaultHost).toBe("gitlab.com");
  });

  test("coerces 'true' to boolean true for boolean fields", async () => {
    const configPath = join(tmpDir, "config.json");
    await setConfig(configPath, "shallow", "true");
    const config = await loadConfig(configPath);
    expect(config.shallow).toBe(true);
  });

  test("coerces 'false' to boolean false for boolean fields", async () => {
    const configPath = join(tmpDir, "config.json");
    await setConfig(configPath, "shallow", "true");
    await setConfig(configPath, "shallow", "false");
    const config = await loadConfig(configPath);
    expect(config.shallow).toBe(false);
  });

  test("stores number field as number not string", async () => {
    const configPath = join(tmpDir, "config.json");
    await setConfig(configPath, "similarityThreshold", "0.85");
    const config = await loadConfig(configPath);
    expect(config.similarityThreshold).toBe(0.85);
    expect(typeof config.similarityThreshold).toBe("number");
  });

  test("rejects unknown config key", async () => {
    const configPath = join(tmpDir, "config.json");
    await expect(setConfig(configPath, "unknownKey", "value")).rejects.toThrow(
      CommandError,
    );
  });

  test("rejects invalid structure value", async () => {
    const configPath = join(tmpDir, "config.json");
    await expect(setConfig(configPath, "structure", "nested")).rejects.toThrow(
      CommandError,
    );
  });

  test("rejects non-numeric values for number fields", async () => {
    const configPath = join(tmpDir, "config.json");
    await expect(
      setConfig(configPath, "similarityThreshold", "abc"),
    ).rejects.toThrow(CommandError);
  });

  test("accepts valid structure value 'flat'", async () => {
    const configPath = join(tmpDir, "config.json");
    await setConfig(configPath, "structure", "flat");
    const config = await loadConfig(configPath);
    expect(config.structure).toBe("flat");
  });

  test("accepts valid structure value 'owner'", async () => {
    const configPath = join(tmpDir, "config.json");
    await setConfig(configPath, "structure", "owner");
    const config = await loadConfig(configPath);
    expect(config.structure).toBe("owner");
  });

  test("accepts valid structure value 'host'", async () => {
    const configPath = join(tmpDir, "config.json");
    await setConfig(configPath, "structure", "host");
    const config = await loadConfig(configPath);
    expect(config.structure).toBe("host");
  });
});
