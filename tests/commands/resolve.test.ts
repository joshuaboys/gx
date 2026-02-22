import { test, expect, describe, beforeEach, afterEach, spyOn } from "bun:test";
import { resolve } from "../../src/commands/resolve.ts";
import { ProjectIndex } from "../../src/lib/index.ts";
import { DEFAULT_CONFIG } from "../../src/types.ts";
import { join } from "path";
import { mkdtemp, rm } from "fs/promises";
import { tmpdir } from "os";

let tmpDir: string;
let indexPath: string;

beforeEach(async () => {
  tmpDir = await mkdtemp(join(tmpdir(), "gx-resolve-"));
  indexPath = join(tmpDir, "index.json");
  // Set up index with test entries
  const idx = await ProjectIndex.load(indexPath);
  idx.add("myproject", { path: "/tmp/myproject", url: "", clonedAt: "" });
  idx.add("gx", { path: "/tmp/gx", url: "", clonedAt: "" });
  await idx.save(indexPath);
});

afterEach(async () => {
  await rm(tmpDir, { recursive: true });
});

describe("resolve", () => {
  test("--list mode prints all names joined by newlines", async () => {
    const logSpy = spyOn(console, "log").mockImplementation(() => {});
    await resolve("", indexPath, DEFAULT_CONFIG, true);
    expect(logSpy).toHaveBeenCalledWith("gx\nmyproject");
    logSpy.mockRestore();
  });

  test("exact match prints path to stdout", async () => {
    const logSpy = spyOn(console, "log").mockImplementation(() => {});
    await resolve("myproject", indexPath, DEFAULT_CONFIG);
    expect(logSpy).toHaveBeenCalledWith("/tmp/myproject");
    logSpy.mockRestore();
  });

  test("no match exits with code 1", async () => {
    const mockExit = spyOn(process, "exit").mockImplementation(() => { throw new Error("exit"); });
    try {
      await resolve("nonexistent", indexPath, DEFAULT_CONFIG);
    } catch {}
    expect(mockExit).toHaveBeenCalledWith(1);
    mockExit.mockRestore();
  });
});
