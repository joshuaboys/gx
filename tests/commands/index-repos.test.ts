import { test, expect, beforeEach, afterEach } from "bun:test";
import { indexRepos } from "../../src/commands/index-repos.ts";
import { ProjectIndex } from "../../src/lib/index.ts";
import { join } from "path";
import { mkdtemp, rm, mkdir } from "fs/promises";
import { tmpdir } from "os";
import type { Config } from "../../src/types.ts";
import { DEFAULT_CONFIG } from "../../src/types.ts";

let tmpDir: string;
let indexPath: string;

beforeEach(async () => {
  tmpDir = await mkdtemp(join(tmpdir(), "gx-test-"));
  indexPath = join(tmpDir, "index.json");
});

afterEach(async () => {
  await rm(tmpDir, { recursive: true });
});

test("indexRepos adds a repo by explicit path", async () => {
  // Create a real git repo
  const repoDir = join(tmpDir, "myrepo");
  await mkdir(repoDir, { recursive: true });
  const init = Bun.spawn(["git", "init"], { cwd: repoDir, stdout: "ignore", stderr: "ignore" });
  await init.exited;

  const config: Config = { ...DEFAULT_CONFIG, projectDir: tmpDir };
  await indexRepos([repoDir], config, indexPath);

  const idx = await ProjectIndex.load(indexPath);
  expect(idx.resolve("myrepo")).toBe(repoDir);
});

test("indexRepos with no paths runs additive scan", async () => {
  // Pre-populate index
  const idx = await ProjectIndex.load(indexPath);
  idx.add("existing", { path: "/some/existing", url: "", clonedAt: "" });
  await idx.save(indexPath);

  // Create a repo to discover
  await mkdir(join(tmpDir, "discovered", ".git"), { recursive: true });

  const config: Config = { ...DEFAULT_CONFIG, projectDir: tmpDir };
  await indexRepos([], config, indexPath);

  const idx2 = await ProjectIndex.load(indexPath);
  expect(idx2.resolve("existing")).toBe("/some/existing");
  expect(idx2.resolve("discovered")).toBe(join(tmpDir, "discovered"));
});

test("indexRepos rejects path without .git", async () => {
  const badDir = join(tmpDir, "notarepo");
  await mkdir(badDir, { recursive: true });

  const config: Config = { ...DEFAULT_CONFIG, projectDir: tmpDir };
  await expect(indexRepos([badDir], config, indexPath)).rejects.toThrow("not a git repository");
});
