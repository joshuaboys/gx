import { test, expect, describe } from "bun:test";
import { jaroWinkler, fuzzyMatch } from "../../src/lib/fuzzy.ts";

describe("jaroWinkler", () => {
  test("identical strings return 1.0", () => {
    expect(jaroWinkler("gx", "gx")).toBe(1.0);
  });

  test("completely different strings return low score", () => {
    expect(jaroWinkler("abc", "xyz")).toBeLessThan(0.5);
  });

  test("typo produces high score", () => {
    const score = jaroWinkler("gclone", "gclne");
    expect(score).toBeGreaterThan(0.85);
  });

  test("empty strings return 1.0", () => {
    expect(jaroWinkler("", "")).toBe(1.0);
  });

  test("one empty string returns 0.0", () => {
    expect(jaroWinkler("abc", "")).toBe(0.0);
    expect(jaroWinkler("", "abc")).toBe(0.0);
  });

  test("single character match", () => {
    expect(jaroWinkler("a", "a")).toBe(1.0);
  });

  test("single character mismatch", () => {
    expect(jaroWinkler("a", "b")).toBeLessThan(0.5);
  });

  test("prefix similarity boosts score", () => {
    // Jaro-Winkler gives bonus for shared prefix
    const jaroOnly = jaroWinkler("MARTHA", "MARHTA");
    expect(jaroOnly).toBeGreaterThan(0.9);
  });

  test("case-insensitive comparison", () => {
    // Our implementation should be case-insensitive for project names
    const score = jaroWinkler("GClone", "gclone");
    expect(score).toBe(1.0);
  });

  test("score is between 0 and 1", () => {
    const score = jaroWinkler("hello", "world");
    expect(score).toBeGreaterThanOrEqual(0);
    expect(score).toBeLessThanOrEqual(1);
  });
});

describe("fuzzyMatch", () => {
  const entries = [
    { name: "gclone", path: "/projects/gclone" },
    { name: "cockpit", path: "/projects/cockpit" },
    { name: "gx", path: "/projects/gx" },
    { name: "dotfiles", path: "/projects/dotfiles" },
    { name: "next-app", path: "/projects/next-app" },
  ];

  test("returns matching entries ranked by score", () => {
    const results = fuzzyMatch("gc", entries);
    expect(results.length).toBeGreaterThan(0);
    // "gclone" scores higher than "gx" for "gc" because "gc" is a prefix of "gclone"
    expect(results[0]!.name).toBe("gclone");
  });

  test("returns gclone ranked first for 'gclon' query", () => {
    const results = fuzzyMatch("gclon", entries);
    expect(results.length).toBeGreaterThan(0);
    expect(results[0]!.name).toBe("gclone");
  });

  test("returns empty array for no matches above threshold", () => {
    const results = fuzzyMatch("xxx", entries);
    expect(results).toEqual([]);
  });

  test("returns cockpit for 'cock' query", () => {
    const results = fuzzyMatch("cock", entries);
    expect(results.length).toBeGreaterThan(0);
    expect(results[0]!.name).toBe("cockpit");
  });

  test("respects custom threshold", () => {
    // Very high threshold should filter most results
    const strict = fuzzyMatch("gclon", entries, 0.99);
    expect(strict.length).toBe(0);

    // Very low threshold should include more results
    const loose = fuzzyMatch("gclon", entries, 0.1);
    expect(loose.length).toBeGreaterThan(0);
  });

  test("results are sorted by score descending", () => {
    const results = fuzzyMatch("g", entries, 0.4);
    for (let i = 1; i < results.length; i++) {
      expect(results[i - 1]!.score).toBeGreaterThanOrEqual(results[i]!.score);
    }
  });

  test("each result includes name, score, and path", () => {
    const results = fuzzyMatch("gclone", entries);
    expect(results.length).toBeGreaterThan(0);
    const first = results[0]!;
    expect(first).toHaveProperty("name");
    expect(first).toHaveProperty("score");
    expect(first).toHaveProperty("path");
    expect(typeof first.score).toBe("number");
  });

  test("handles empty entries array", () => {
    const results = fuzzyMatch("test", []);
    expect(results).toEqual([]);
  });

  test("handles empty query", () => {
    const results = fuzzyMatch("", entries);
    expect(results).toEqual([]);
  });
});
