import { test, expect, beforeEach, afterEach } from "bun:test";
import { toPath } from "../../src/lib/path.ts";
import type { ParsedRepo, Config } from "../../src/types.ts";
import { DEFAULT_CONFIG } from "../../src/types.ts";

let savedGxAgent: string | undefined;

beforeEach(() => {
  savedGxAgent = process.env.GX_AGENT;
  delete process.env.GX_AGENT;
});

afterEach(() => {
  if (savedGxAgent !== undefined) {
    process.env.GX_AGENT = savedGxAgent;
  } else {
    delete process.env.GX_AGENT;
  }
});

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

test("owner structure with GX_AGENT routes to dotdir", () => {
  process.env.GX_AGENT = "morgan";
  const config: Config = { ...DEFAULT_CONFIG, projectDir: "/home/user/src" };
  expect(toPath(repo, config)).toBe("/home/user/src/.morgan/juev/gclone");
});

test("flat structure with GX_AGENT routes to dotdir", () => {
  process.env.GX_AGENT = "morgan";
  const config: Config = {
    ...DEFAULT_CONFIG,
    projectDir: "/home/user/src",
    structure: "flat",
  };
  expect(toPath(repo, config)).toBe("/home/user/src/.morgan/gclone");
});

test("host structure with GX_AGENT routes to dotdir", () => {
  process.env.GX_AGENT = "morgan";
  const config: Config = {
    ...DEFAULT_CONFIG,
    projectDir: "/home/user/src",
    structure: "host",
  };
  expect(toPath(repo, config)).toBe(
    "/home/user/src/.morgan/github.com/juev/gclone",
  );
});

test("no GX_AGENT produces normal path", () => {
  const config: Config = { ...DEFAULT_CONFIG, projectDir: "/home/user/src" };
  expect(toPath(repo, config)).toBe("/home/user/src/juev/gclone");
});
