import { test, expect, describe } from "bun:test";
import { parseUrl, toCloneUrl } from "../../src/lib/url.ts";

describe("toCloneUrl", () => {
  test("HTTPS-parsed repo returns https clone URL", () => {
    const parsed = parseUrl("https://github.com/juev/gclone.git");
    expect(toCloneUrl(parsed)).toBe("https://github.com/juev/gclone.git");
  });

  test("SSH git@ URL is preserved", () => {
    const parsed = parseUrl("git@github.com:juev/gclone.git");
    expect(toCloneUrl(parsed)).toBe("git@github.com:juev/gclone.git");
  });

  test("SSH ssh:// URL is preserved", () => {
    const parsed = parseUrl("ssh://git@github.com/juev/gclone.git");
    expect(toCloneUrl(parsed)).toBe("ssh://git@github.com/juev/gclone.git");
  });

  test("shorthand appends .git to https URL", () => {
    const parsed = parseUrl("juev/gclone");
    expect(toCloneUrl(parsed)).toBe("https://github.com/juev/gclone.git");
  });
});

describe("parseUrl edge cases", () => {
  test("uses custom defaultHost for shorthand", () => {
    const result = parseUrl("juev/gclone", "gitlab.com");
    expect(result.host).toBe("gitlab.com");
  });

  test("trims whitespace from input", () => {
    const result = parseUrl("  juev/gclone  ");
    expect(result.repo).toBe("gclone");
  });

  test("rejects path traversal in owner segment", () => {
    expect(() => parseUrl("https://github.com/../etc")).toThrow("traversal");
  });

  test("allows legitimate dots in repo names", () => {
    const result = parseUrl("user/my.cool.repo");
    expect(result.repo).toBe("my.cool.repo");
  });
});
