import { test, expect, describe, beforeEach, afterEach, spyOn } from "bun:test";
import { setConfig } from "../../src/commands/config.ts";
import { loadConfig } from "../../src/lib/config.ts";
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
    // First set to true, then set to false
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
    const mockExit = spyOn(process, "exit").mockImplementation(() => { throw new Error("exit"); });
    try {
      await setConfig(configPath, "unknownKey", "value").catch(() => {});
    } catch {}
    expect(mockExit).toHaveBeenCalledWith(1);
    mockExit.mockRestore();
  });

  test("rejects invalid structure value", async () => {
    const configPath = join(tmpDir, "config.json");
    const mockExit = spyOn(process, "exit").mockImplementation(() => { throw new Error("exit"); });
    try {
      await setConfig(configPath, "structure", "nested").catch(() => {});
    } catch {}
    expect(mockExit).toHaveBeenCalledWith(1);
    mockExit.mockRestore();
  });

  test("rejects non-numeric values for number fields", async () => {
    const configPath = join(tmpDir, "config.json");
    const mockExit = spyOn(process, "exit").mockImplementation(() => { throw new Error("exit"); });
    try {
      await setConfig(configPath, "similarityThreshold", "abc").catch(() => {});
    } catch {}
    expect(mockExit).toHaveBeenCalledWith(1);
    mockExit.mockRestore();
  });

  test("accepts valid structure value 'flat'", async () => {
    const configPath = join(tmpDir, "config.json");
    await setConfig(configPath, "structure", "flat");
    const config = await loadConfig(configPath);
    expect(config.structure).toBe("flat");
  });

  test("accepts valid structure value 'host'", async () => {
    const configPath = join(tmpDir, "config.json");
    await setConfig(configPath, "structure", "host");
    const config = await loadConfig(configPath);
    expect(config.structure).toBe("host");
  });
});
