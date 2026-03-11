import { stat } from "fs/promises";
import { ProjectIndex } from "../lib/index.ts";
import {
  resolveProjectName,
  formatAmbiguous,
  formatAutoMatch,
} from "../lib/resolve-name.ts";
import { CommandError } from "../lib/errors.ts";
import type { Config } from "../types.ts";

export interface ResumeContext {
  branch: string;
  dirtyCount: number;
  lastCommit: string;
}

async function runGit(cwd: string, args: string[]): Promise<string> {
  try {
    const proc = Bun.spawn(["git", ...args], {
      cwd,
      stdout: "pipe",
      stderr: "ignore",
    });
    const exitCode = await proc.exited;
    if (exitCode !== 0) return "";
    return (await new Response(proc.stdout).text()).trim();
  } catch {
    return "";
  }
}

export async function getResumeContext(
  dir: string,
): Promise<ResumeContext | null> {
  try {
    const s = await stat(dir);
    if (!s.isDirectory()) return null;
  } catch {
    return null;
  }

  const branch = await runGit(dir, ["branch", "--show-current"]);
  const statusOutput = await runGit(dir, ["status", "--porcelain"]);
  const dirtyCount = statusOutput
    ? statusOutput.split("\n").filter((l) => l.length > 0).length
    : 0;
  const lastCommit = await runGit(dir, ["log", "-1", "--format=%h %s (%cr)"]);

  return { branch: branch || "HEAD (detached)", dirtyCount, lastCommit };
}

export async function resume(
  name: string,
  indexPath: string,
  config: Config,
): Promise<void> {
  const idx = await ProjectIndex.load(indexPath);
  const result = resolveProjectName(name, idx, config);

  let resolvedName: string;
  let resolvedPath: string;

  switch (result.kind) {
    case "exact":
      resolvedName = result.name;
      resolvedPath = result.path;
      break;

    case "auto":
      console.error(formatAutoMatch(result.query, result.name, result.score));
      resolvedName = result.name;
      resolvedPath = result.path;
      break;

    case "ambiguous":
      throw new CommandError(formatAmbiguous(result.query, result.matches));

    case "missing":
      throw new CommandError(`Project '${name}' not found in index`);
  }

  const ctx = await getResumeContext(resolvedPath);
  if (!ctx) {
    throw new CommandError(
      `Project '${resolvedName}' directory not found: ${resolvedPath}`,
    );
  }

  idx.touch(resolvedName);
  await idx.save(indexPath);

  const dirty =
    ctx.dirtyCount > 0
      ? ` — ${ctx.dirtyCount} dirty ${ctx.dirtyCount === 1 ? "file" : "files"}`
      : "";
  console.error(`${resolvedName} (${ctx.branch})${dirty}`);
  if (ctx.lastCommit) {
    console.error(`  Last commit: ${ctx.lastCommit}`);
  }

  console.log(resolvedPath);
}
