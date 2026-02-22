import { test, expect } from "bun:test";
import { parseUrl } from "../../src/lib/url.ts";

// Shorthand
test("parses shorthand user/repo", () => {
  const result = parseUrl("juev/gclone");
  expect(result.host).toBe("github.com");
  expect(result.owner).toBe("juev");
  expect(result.repo).toBe("gclone");
});

// HTTPS
test("parses HTTPS URL", () => {
  const result = parseUrl("https://github.com/juev/gclone.git");
  expect(result.host).toBe("github.com");
  expect(result.owner).toBe("juev");
  expect(result.repo).toBe("gclone");
});

test("parses HTTPS URL without .git suffix", () => {
  const result = parseUrl("https://github.com/juev/gclone");
  expect(result.host).toBe("github.com");
  expect(result.owner).toBe("juev");
  expect(result.repo).toBe("gclone");
});

test("parses HTTPS URL with trailing slash", () => {
  const result = parseUrl("https://github.com/juev/gclone/");
  expect(result.repo).toBe("gclone");
});

// SSH
test("parses SSH URL (git@host:user/repo)", () => {
  const result = parseUrl("git@github.com:juev/gclone.git");
  expect(result.host).toBe("github.com");
  expect(result.owner).toBe("juev");
  expect(result.repo).toBe("gclone");
});

test("parses SSH URL without .git suffix", () => {
  const result = parseUrl("git@gitlab.com:company/project");
  expect(result.host).toBe("gitlab.com");
  expect(result.owner).toBe("company");
  expect(result.repo).toBe("project");
});

// Git protocol
test("parses git:// URL", () => {
  const result = parseUrl("git://github.com/juev/gclone.git");
  expect(result.host).toBe("github.com");
  expect(result.owner).toBe("juev");
  expect(result.repo).toBe("gclone");
});

// Nested paths
test("parses URL with nested path (gitlab groups)", () => {
  const result = parseUrl("https://gitlab.com/group/subgroup/repo.git");
  expect(result.host).toBe("gitlab.com");
  expect(result.owner).toBe("group/subgroup");
  expect(result.repo).toBe("repo");
});

// Errors
test("throws on empty input", () => {
  expect(() => parseUrl("")).toThrow();
});

test("throws on invalid URL", () => {
  expect(() => parseUrl("not a url at all")).toThrow();
});

test("throws on URL with path traversal", () => {
  expect(() => parseUrl("https://github.com/../etc/passwd")).toThrow();
});
