import { test, expect, describe } from "bun:test";
import { getClaudeMd, getPlanCommand, getReviewCommand } from "../../src/lib/templates.ts";

describe("getClaudeMd", () => {
  test("typescript-bun template references bun", () => {
    expect(getClaudeMd("typescript-bun").toLowerCase()).toContain("bun");
  });

  test("typescript-node template references node", () => {
    expect(getClaudeMd("typescript-node").toLowerCase()).toContain("node");
  });

  test("rust template references cargo", () => {
    expect(getClaudeMd("rust").toLowerCase()).toContain("cargo");
  });

  test("go template references go build", () => {
    expect(getClaudeMd("go")).toContain("go build");
  });

  test("python template references pytest", () => {
    expect(getClaudeMd("python")).toContain("pytest");
  });

  test("generic template contains TODO placeholders", () => {
    expect(getClaudeMd("generic")).toContain("TODO");
  });

  test("all templates are non-empty", () => {
    const types = ["typescript-bun", "typescript-node", "rust", "go", "python", "generic"] as const;
    for (const t of types) {
      expect(getClaudeMd(t).length).toBeGreaterThan(0);
    }
  });
});

describe("getPlanCommand", () => {
  test("contains $ARGUMENTS placeholder", () => {
    expect(getPlanCommand()).toContain("$ARGUMENTS");
  });
});

describe("getReviewCommand", () => {
  test("contains $ARGUMENTS placeholder", () => {
    expect(getReviewCommand()).toContain("$ARGUMENTS");
  });
});
