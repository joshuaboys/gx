import { test, expect, describe, beforeEach, afterEach, spyOn } from "bun:test";
import { join } from "path";
import { mkdir, mkdtemp, rm } from "fs/promises";
import { tmpdir } from "os";
import { doctor } from "../../src/commands/doctor.ts";
import type { Config } from "../../src/types.ts";

async function captureOutput(fn: () => Promise<void>): Promise<string> {
  const lines: string[] = [];
  const orig = console.log;
  console.log = (...args: unknown[]) => lines.push(args.join(" "));
  try {
    await fn();
  } finally {
    console.log = orig;
  }
  return lines.join("\n");
}

describe("doctor", () => {
  let tmpDir: string;
  let origHome: string | undefined;
  let origShell: string | undefined;
  let whichSpy: ReturnType<typeof spyOn>;

  beforeEach(async () => {
    tmpDir = await mkdtemp(join(tmpdir(), "gx-doctor-"));
    origHome = process.env.HOME;
    origShell = process.env.SHELL;
    process.env.HOME = tmpDir;
    process.env.SHELL = "/bin/zsh";
    whichSpy = spyOn(Bun, "which").mockReturnValue("/usr/local/bin/gx");
  });

  afterEach(async () => {
    whichSpy.mockRestore();
    if (origHome !== undefined) {
      process.env.HOME = origHome;
    } else {
      delete process.env.HOME;
    }
    if (origShell !== undefined) {
      process.env.SHELL = origShell;
    } else {
      delete process.env.SHELL;
    }
    await rm(tmpDir, { recursive: true, force: true });
  });

  test("reports healthy configured paths", async () => {
    const configDir = join(tmpDir, ".config", "gx");
    const projectDir = join(tmpDir, "Projects", "src");
    await mkdir(configDir, { recursive: true });
    await mkdir(projectDir, { recursive: true });
    await Bun.write(join(tmpDir, ".zshrc"), 'eval "$(gx shell-init)"\n');
    const indexPath = join(configDir, "index.json");
    await Bun.write(
      indexPath,
      JSON.stringify({
        projects: {
          gx: { path: projectDir, url: "", clonedAt: "" },
        },
      }),
    );
    const configPath = join(configDir, "config.json");
    await Bun.write(configPath, "{}");

    const config: Config = {
      projectDir,
      defaultHost: "github.com",
      defaultOwner: "",
      structure: "owner",
      shallow: false,
      similarityThreshold: 0.7,
      editor: "",
    };

    const output = await captureOutput(() =>
      doctor(configPath, indexPath, config),
    );
    expect(output).toContain("runtime");
    expect(output).toContain("binary");
    expect(output).toContain("shell");
    expect(output).toContain("doctor: ok");
  });

  test("warns for first-run missing files", async () => {
    whichSpy.mockReturnValue(null as unknown as string);
    const config: Config = {
      projectDir: join(tmpDir, "missing-projects"),
      defaultHost: "github.com",
      defaultOwner: "",
      structure: "owner",
      shallow: false,
      similarityThreshold: 0.7,
      editor: "",
    };

    const output = await captureOutput(() =>
      doctor(
        join(tmpDir, ".config", "gx", "config.json"),
        join(tmpDir, ".config", "gx", "index.json"),
        config,
      ),
    );

    expect(output).toContain("gx is not on PATH");
    expect(output).toContain("missing; defaults are in use");
    expect(output).toContain("integration not found");
    expect(output).toContain("warning(s)");
  });
});
