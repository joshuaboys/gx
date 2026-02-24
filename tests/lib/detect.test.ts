import { test, expect, describe, beforeEach, afterEach } from "bun:test";
import { detectProjectType, type ProjectType } from "../../src/lib/detect.ts";
import { join } from "path";
import { mkdtemp, rm } from "fs/promises";
import { tmpdir } from "os";

let tmpDir: string;

beforeEach(async () => {
  tmpDir = await mkdtemp(join(tmpdir(), "gx-detect-"));
});

afterEach(async () => {
  await rm(tmpDir, { recursive: true });
});

describe("detectProjectType", () => {
  test("detects typescript-bun when package.json and bun.lock (text format) exist", async () => {
    await Bun.write(join(tmpDir, "package.json"), "{}");
    await Bun.write(join(tmpDir, "bun.lock"), "{}");
    expect(await detectProjectType(tmpDir)).toBe("typescript-bun");
  });

  test("detects typescript-bun when package.json and bun.lockb (binary format) exist", async () => {
    await Bun.write(join(tmpDir, "package.json"), "{}");
    await Bun.write(join(tmpDir, "bun.lockb"), "");
    expect(await detectProjectType(tmpDir)).toBe("typescript-bun");
  });

  test("detects typescript-node when only package.json exists", async () => {
    await Bun.write(join(tmpDir, "package.json"), "{}");
    expect(await detectProjectType(tmpDir)).toBe("typescript-node");
  });

  test("detects rust when Cargo.toml exists", async () => {
    await Bun.write(join(tmpDir, "Cargo.toml"), "");
    expect(await detectProjectType(tmpDir)).toBe("rust");
  });

  test("detects go when go.mod exists", async () => {
    await Bun.write(join(tmpDir, "go.mod"), "");
    expect(await detectProjectType(tmpDir)).toBe("go");
  });

  test("detects python when pyproject.toml exists", async () => {
    await Bun.write(join(tmpDir, "pyproject.toml"), "");
    expect(await detectProjectType(tmpDir)).toBe("python");
  });

  test("detects python when requirements.txt exists", async () => {
    await Bun.write(join(tmpDir, "requirements.txt"), "");
    expect(await detectProjectType(tmpDir)).toBe("python");
  });

  test("returns generic when no manifest files found", async () => {
    expect(await detectProjectType(tmpDir)).toBe("generic");
  });

  test("prefers typescript-bun over rust when both exist", async () => {
    await Bun.write(join(tmpDir, "package.json"), "{}");
    await Bun.write(join(tmpDir, "bun.lock"), "{}");
    await Bun.write(join(tmpDir, "Cargo.toml"), "");
    expect(await detectProjectType(tmpDir)).toBe("typescript-bun");
  });

  test("prefers rust over go when both exist", async () => {
    await Bun.write(join(tmpDir, "Cargo.toml"), "");
    await Bun.write(join(tmpDir, "go.mod"), "");
    expect(await detectProjectType(tmpDir)).toBe("rust");
  });
});
