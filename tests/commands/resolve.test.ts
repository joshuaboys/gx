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

test("touch updates lastVisited on exact match", async () => {
  const idx = await ProjectIndex.load(indexPath);
  idx.add("gx", {
    path: "/tmp/gx",
    url: "https://github.com/joshuaboys/gx",
    clonedAt: "2026-01-01T00:00:00Z",
  });
  await idx.save(indexPath);

  const idx2 = await ProjectIndex.load(indexPath);
  const found = idx2.touch("gx");
  expect(found).toBe(true);
  await idx2.save(indexPath);

  const idx3 = await ProjectIndex.load(indexPath);
  const entry = idx3.list().find((e) => e.name === "gx");
  expect(entry!.lastVisited).toBeDefined();
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
    const mockExit = spyOn(process, "exit").mockImplementation(() => {
      throw new Error("exit");
    });
    try {
      await resolve("nonexistent", indexPath, DEFAULT_CONFIG);
    } catch {}
    expect(mockExit).toHaveBeenCalledWith(1);
    mockExit.mockRestore();
  });
});
