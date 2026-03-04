import { test, expect, beforeEach, afterEach } from "bun:test";
import { ProjectIndex } from "../../src/lib/index.ts";
import { join } from "path";
import { mkdtemp, rm } from "fs/promises";
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

test("recent lists projects sorted by lastVisited", async () => {
  const idx = await ProjectIndex.load(indexPath);
  idx.add("old", {
    path: "/tmp/old",
    url: "",
    clonedAt: "2026-01-01T00:00:00Z",
    lastVisited: "2026-01-01T00:00:00Z",
  });
  idx.add("new", {
    path: "/tmp/new",
    url: "",
    clonedAt: "2026-01-02T00:00:00Z",
    lastVisited: "2026-03-01T00:00:00Z",
  });
  await idx.save(indexPath);

  const result = idx.recent();
  expect(result[0]![0]).toBe("new");
  expect(result[1]![0]).toBe("old");
});

test("recent with empty index produces no output", async () => {
  const idx = await ProjectIndex.load(indexPath);
  const result = idx.recent();
  expect(result).toHaveLength(0);
});
