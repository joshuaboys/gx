import { test, expect } from "bun:test";
import { toPath } from "../../src/lib/path.ts";
import type { ParsedRepo, Config } from "../../src/types.ts";
import { DEFAULT_CONFIG } from "../../src/types.ts";

const repo: ParsedRepo = {
  host: "github.com",
  owner: "juev",
  repo: "gclone",
  originalUrl: "https://github.com/juev/gclone",
};

test("flat structure: repo only", () => {
  const config: Config = {
    ...DEFAULT_CONFIG,
    projectDir: "/home/user/src",
    structure: "flat",
  };
  expect(toPath(repo, config)).toBe("/home/user/src/gclone");
});

test("owner structure: owner/repo", () => {
  const config: Config = { ...DEFAULT_CONFIG, projectDir: "/home/user/src" };
  expect(toPath(repo, config)).toBe("/home/user/src/juev/gclone");
});

test("host structure: host/owner/repo", () => {
  const config: Config = {
    ...DEFAULT_CONFIG,
    projectDir: "/home/user/src",
    structure: "host",
  };
  expect(toPath(repo, config)).toBe("/home/user/src/github.com/juev/gclone");
});

test("nested owner preserves path", () => {
  const nested: ParsedRepo = {
    ...repo,
    owner: "group/subgroup",
    repo: "project",
  };
  const config: Config = { ...DEFAULT_CONFIG, projectDir: "/home/user/src" };
  expect(toPath(nested, config)).toBe("/home/user/src/group/subgroup/project");
});

test("tilde expansion in projectDir", () => {
  const config: Config = { ...DEFAULT_CONFIG, projectDir: "~/src" };
  const result = toPath(repo, config);
  expect(result).not.toContain("~");
  expect(result).toContain("/juev/gclone");
});
