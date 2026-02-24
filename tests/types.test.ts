import { test, expect } from "bun:test";
import { DEFAULT_CONFIG } from "../src/types.ts";

test("DEFAULT_CONFIG has expected values", () => {
  expect(DEFAULT_CONFIG.defaultHost).toBe("github.com");
  expect(DEFAULT_CONFIG.structure).toBe("owner");
  expect(DEFAULT_CONFIG.shallow).toBe(false);
  expect(DEFAULT_CONFIG.projectDir).toBe("~/Projects/src");
  expect(DEFAULT_CONFIG.similarityThreshold).toBe(0.7);
});
