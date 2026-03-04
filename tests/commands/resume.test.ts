import { test, expect, beforeEach, afterEach } from "bun:test";
import { ProjectIndex } from "../../src/lib/index.ts";
import { getResumeContext } from "../../src/commands/resume.ts";
import { join } from "path";
import { mkdtemp, rm, mkdir } from "fs/promises";
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

test("getResumeContext returns context for a git repo", async () => {
  const repoDir = join(tmpDir, "myrepo");
  await mkdir(repoDir, { recursive: true });
  let proc = Bun.spawn(["git", "init"], {
    cwd: repoDir,
    stdout: "ignore",
    stderr: "ignore",
  });
  await proc.exited;
  proc = Bun.spawn(["git", "commit", "--allow-empty", "-m", "initial"], {
    cwd: repoDir,
    stdout: "ignore",
    stderr: "ignore",
  });
  await proc.exited;

  const ctx = await getResumeContext(repoDir);
  expect(ctx).not.toBeNull();
  expect(ctx!.branch).toBeDefined();
  expect(typeof ctx!.dirtyCount).toBe("number");
  expect(ctx!.lastCommit).toBeDefined();
});

test("getResumeContext returns null for missing directory", async () => {
  const ctx = await getResumeContext("/nonexistent/path");
  expect(ctx).toBeNull();
});

test("resume fails for unknown project name", async () => {
  const idx = await ProjectIndex.load(indexPath);
  await idx.save(indexPath);
  expect(idx.resolve("unknown")).toBeNull();
});
