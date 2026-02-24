import { test, expect, describe, beforeEach, afterEach } from "bun:test";
import { initAgent } from "../../src/commands/init.ts";
import { join } from "path";
import { mkdtemp, rm } from "fs/promises";
import { tmpdir } from "os";

let tmpDir: string;

beforeEach(async () => {
  tmpDir = await mkdtemp(join(tmpdir(), "gx-init-"));
});

afterEach(async () => {
  await rm(tmpDir, { recursive: true });
});

describe("initAgent", () => {
  test("creates .claude/CLAUDE.md", async () => {
    await initAgent(tmpDir, {});
    const file = Bun.file(join(tmpDir, ".claude", "CLAUDE.md"));
    expect(await file.exists()).toBe(true);
    const content = await file.text();
    expect(content.length).toBeGreaterThan(0);
  });

  test("creates .claude/commands/plan.md", async () => {
    await initAgent(tmpDir, {});
    const file = Bun.file(join(tmpDir, ".claude", "commands", "plan.md"));
    expect(await file.exists()).toBe(true);
    const content = await file.text();
    expect(content).toContain("$ARGUMENTS");
  });

  test("creates .claude/commands/review.md", async () => {
    await initAgent(tmpDir, {});
    const file = Bun.file(join(tmpDir, ".claude", "commands", "review.md"));
    expect(await file.exists()).toBe(true);
    const content = await file.text();
    expect(content).toContain("$ARGUMENTS");
  });

  test("detects typescript-bun project type", async () => {
    await Bun.write(join(tmpDir, "package.json"), "{}");
    await Bun.write(join(tmpDir, "bun.lock"), "{}");
    await initAgent(tmpDir, {});
    const content = await Bun.file(
      join(tmpDir, ".claude", "CLAUDE.md"),
    ).text();
    expect(content.toLowerCase()).toContain("bun");
  });

  test("respects explicit --type override", async () => {
    await initAgent(tmpDir, { type: "rust" });
    const content = await Bun.file(
      join(tmpDir, ".claude", "CLAUDE.md"),
    ).text();
    expect(content.toLowerCase()).toContain("cargo");
  });

  test("refuses to overwrite without --force", async () => {
    await initAgent(tmpDir, {});
    await expect(initAgent(tmpDir, {})).rejects.toThrow(
      "already exists",
    );
  });

  test("overwrites with --force flag", async () => {
    await initAgent(tmpDir, {});
    // Should not throw
    await initAgent(tmpDir, { force: true });
    const file = Bun.file(join(tmpDir, ".claude", "CLAUDE.md"));
    expect(await file.exists()).toBe(true);
  });

  test("uses generic template for unknown directory", async () => {
    await initAgent(tmpDir, {});
    const content = await Bun.file(
      join(tmpDir, ".claude", "CLAUDE.md"),
    ).text();
    expect(content.length).toBeGreaterThan(0);
  });

  test("rejects invalid --type value", async () => {
    await expect(initAgent(tmpDir, { type: "invalid-type" })).rejects.toThrow(
      "Unknown project type",
    );
  });
});
