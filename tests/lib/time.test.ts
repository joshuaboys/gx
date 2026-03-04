import { test, expect } from "bun:test";
import { relativeTime } from "../../src/lib/time.ts";

test("returns 'just now' for timestamps within 60 seconds", () => {
  const now = new Date();
  expect(relativeTime(now.toISOString())).toBe("just now");
});

test("returns minutes ago", () => {
  const t = new Date(Date.now() - 5 * 60 * 1000).toISOString();
  expect(relativeTime(t)).toBe("5 minutes ago");
});

test("returns '1 minute ago' for singular", () => {
  const t = new Date(Date.now() - 90 * 1000).toISOString();
  expect(relativeTime(t)).toBe("1 minute ago");
});

test("returns hours ago", () => {
  const t = new Date(Date.now() - 3 * 60 * 60 * 1000).toISOString();
  expect(relativeTime(t)).toBe("3 hours ago");
});

test("returns days ago", () => {
  const t = new Date(Date.now() - 2 * 24 * 60 * 60 * 1000).toISOString();
  expect(relativeTime(t)).toBe("2 days ago");
});

test("returns weeks ago", () => {
  const t = new Date(Date.now() - 14 * 24 * 60 * 60 * 1000).toISOString();
  expect(relativeTime(t)).toBe("2 weeks ago");
});

test("returns months ago", () => {
  const t = new Date(Date.now() - 60 * 24 * 60 * 60 * 1000).toISOString();
  expect(relativeTime(t)).toBe("2 months ago");
});

test("returns empty string for empty input", () => {
  expect(relativeTime("")).toBe("");
});
