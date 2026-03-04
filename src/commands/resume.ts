import { stat } from "fs/promises";
import { ProjectIndex } from "../lib/index.ts";
import { fuzzyMatch, AUTO_JUMP_THRESHOLD } from "../lib/fuzzy.ts";
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

function resolveName(
  name: string,
  idx: ProjectIndex,
  config: Config,
): { resolvedName: string; path: string } | null {
  const path = idx.resolve(name);
  if (path) return { resolvedName: name, path };

  const entries = idx.list().map((e) => ({ name: e.name, path: e.path }));
  const matches = fuzzyMatch(name, entries, config.similarityThreshold);

  if (matches.length === 0) return null;

  const first = matches[0];
  if (matches.length === 1 && first && first.score >= AUTO_JUMP_THRESHOLD) {
    console.error(
      `Fuzzy match: '${name}' -> '${first.name}' (${(first.score * 100).toFixed(0)}%)`,
    );
    return { resolvedName: first.name, path: first.path };
  }

  console.error(`No exact match for '${name}'. Did you mean:`);
  for (const [i, m] of matches.entries()) {
    console.error(`  ${i + 1}. ${m.name} (${(m.score * 100).toFixed(0)}%)`);
  }
  return null;
}

export async function resume(
  name: string,
  indexPath: string,
  config: Config,
): Promise<void> {
  const idx = await ProjectIndex.load(indexPath);

  const resolved = resolveName(name, idx, config);
  if (!resolved) {
    console.error(`Project '${name}' not found in index`);
    process.exit(1);
  }

  const ctx = await getResumeContext(resolved.path);
  if (!ctx) {
    console.error(
      `Project '${resolved.resolvedName}' directory not found: ${resolved.path}`,
    );
    process.exit(1);
  }

  idx.touch(resolved.resolvedName);
  await idx.save(indexPath);

  const dirty =
    ctx.dirtyCount > 0
      ? ` — ${ctx.dirtyCount} dirty ${ctx.dirtyCount === 1 ? "file" : "files"}`
      : "";
  console.error(`${resolved.resolvedName} (${ctx.branch})${dirty}`);
  if (ctx.lastCommit) {
    console.error(`  Last commit: ${ctx.lastCommit}`);
  }

  console.log(resolved.path);
}
