import { test, expect, describe, beforeEach, afterEach } from "bun:test";
import { shellInit } from "../../src/commands/shell-init.ts";

// Capture stdout by temporarily replacing console.log
function captureOutput(fn: () => void): string {
  const lines: string[] = [];
  const orig = console.log;
  console.log = (...args: unknown[]) => lines.push(args.join(" "));
  try {
    fn();
  } finally {
    console.log = orig;
  }
  return lines.join("\n");
}

describe("shellInit", () => {
  let origShell: string | undefined;
  let origOverride: string | undefined;
  let origExit: typeof process.exit;
  let origArgv0: string | undefined;

  beforeEach(() => {
    origShell = process.env.SHELL;
    origOverride = process.env.GX_SHELL_OVERRIDE;
    origExit = process.exit;
    origArgv0 = process.argv[0];
  });

  afterEach(() => {
    if (origShell !== undefined) {
      process.env.SHELL = origShell;
    } else {
      delete process.env.SHELL;
    }
    if (origOverride !== undefined) {
      process.env.GX_SHELL_OVERRIDE = origOverride;
    } else {
      delete process.env.GX_SHELL_OVERRIDE;
    }
    process.exit = origExit;
    process.argv[0] = origArgv0!;
  });

  test("explicit zsh outputs zsh integration", () => {
    const output = captureOutput(() => shellInit("zsh"));
    expect(output).toContain("gx()");
    expect(output).toContain("compdef _gx gx");
    expect(output).toContain("_GX_BIN=");
  });

  test("explicit bash outputs bash integration", () => {
    const output = captureOutput(() => shellInit("bash"));
    expect(output).toContain("gx()");
    expect(output).toContain("complete -F _gx_completions gx");
    expect(output).toContain("COMPREPLY");
  });

  test("explicit fish outputs fish integration", () => {
    const output = captureOutput(() => shellInit("fish"));
    expect(output).toContain("function gx");
    expect(output).toContain("complete -c gx");
    expect(output).toContain("__fish_use_subcommand");
  });

  test("auto-detects zsh from SHELL env", () => {
    process.env.SHELL = "/bin/zsh";
    const output = captureOutput(() => shellInit());
    expect(output).toContain("compdef _gx gx");
  });

  test("auto-detects bash from SHELL env", () => {
    process.env.SHELL = "/usr/bin/bash";
    const output = captureOutput(() => shellInit());
    expect(output).toContain("complete -F _gx_completions gx");
  });

  test("auto-detects fish from SHELL env", () => {
    process.env.SHELL = "/usr/bin/fish";
    const output = captureOutput(() => shellInit());
    expect(output).toContain("complete -c gx");
  });

  test("GX_SHELL_OVERRIDE takes priority over SHELL", () => {
    process.env.SHELL = "/bin/zsh";
    process.env.GX_SHELL_OVERRIDE = "bash";
    const output = captureOutput(() => shellInit());
    expect(output).toContain("complete -F _gx_completions gx");
  });

  test("unsupported shell exits with error", () => {
    let exitCode: number | undefined;
    process.exit = ((code: number) => {
      exitCode = code;
      throw new Error("exit");
    }) as never;
    expect(() => shellInit("powershell")).toThrow("exit");
    expect(exitCode).toBe(1);
  });

  test("unknown SHELL env exits with error", () => {
    process.env.SHELL = "/usr/bin/tcsh";
    let exitCode: number | undefined;
    process.exit = ((code: number) => {
      exitCode = code;
      throw new Error("exit");
    }) as never;
    expect(() => shellInit()).toThrow("exit");
    expect(exitCode).toBe(1);
  });

  test("zsh output includes shell-init in command list", () => {
    const output = captureOutput(() => shellInit("zsh"));
    expect(output).toContain("shell-init");
  });

  test("bash output includes shell-init in command list", () => {
    const output = captureOutput(() => shellInit("bash"));
    expect(output).toContain("shell-init");
  });

  test("fish output includes shell-init in command list", () => {
    const output = captureOutput(() => shellInit("fish"));
    expect(output).toContain("shell-init");
  });

  test("zsh clone handler uses cd", () => {
    const output = captureOutput(() => shellInit("zsh"));
    expect(output).toContain('cd "$output"');
  });

  test("bash clone handler uses cd", () => {
    const output = captureOutput(() => shellInit("bash"));
    expect(output).toContain('cd "$output"');
  });

  test("fish clone handler uses quoted cd", () => {
    const output = captureOutput(() => shellInit("fish"));
    expect(output).toContain('cd "$output"');
  });

  test("fish jump handler uses quoted cd", () => {
    const output = captureOutput(() => shellInit("fish"));
    expect(output).toContain('cd "$target"');
  });

  test("output embeds absolute binary path", () => {
    const output = captureOutput(() => shellInit("zsh"));
    expect(output).toContain("_GX_BIN=");
    // Should be an absolute path
    expect(output).toMatch(/_GX_BIN="\/.*"/);
  });

  test("all shells use $_GX_BIN instead of command gx", () => {
    for (const shell of ["zsh", "bash", "fish"] as const) {
      const output = captureOutput(() => shellInit(shell));
      expect(output).not.toContain("command gx");
    }
  });

  test("dev mode (argv[0]=bun) falls back to gx on PATH", () => {
    process.argv[0] = "bun";
    const output = captureOutput(() => shellInit("zsh"));
    // Should resolve to the compiled gx binary on PATH, not cwd/bun
    expect(output).not.toContain("/bun");
    expect(output).toMatch(/_GX_BIN=".*gx"/);
  });

  test("dev mode with full bun path falls back to gx on PATH", () => {
    process.argv[0] = "/home/user/.bun/bin/bun";
    const output = captureOutput(() => shellInit("zsh"));
    expect(output).not.toContain("/bun");
    expect(output).toMatch(/_GX_BIN=".*gx"/);
  });

  test("compiled binary path is used directly", () => {
    process.argv[0] = "/home/user/.local/bin/gx";
    const output = captureOutput(() => shellInit("zsh"));
    expect(output).toContain('_GX_BIN="/home/user/.local/bin/gx"');
  });
});
