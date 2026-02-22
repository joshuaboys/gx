import { join } from "path";
import { mkdir, lstat } from "fs/promises";
import { parseUrl, toCloneUrl } from "../lib/url.ts";
import { toPath } from "../lib/path.ts";
import { ProjectIndex } from "../lib/index.ts";
import type { Config } from "../types.ts";

export async function cloneRepo(
  input: string,
  config: Config,
  indexPath: string
): Promise<string> {
  const parsed = parseUrl(input, config.defaultHost);
  const targetDir = toPath(parsed, config);

  // Check if target already exists (handles .git as dir or file for worktrees)
  try {
    const gitPath = join(targetDir, ".git");
    const stat = await lstat(gitPath);
    if (stat.isSymbolicLink()) {
      throw new Error(`Refusing to clone into symlink: ${targetDir}`);
    }
    if (stat.isDirectory() || stat.isFile()) {
      console.error(`already exists: ${targetDir}`);
      return targetDir;
    }
  } catch (err: unknown) {
    if (err instanceof Error && "code" in err && (err as NodeJS.ErrnoException).code !== "ENOENT") {
      throw err;
    }
    // ENOENT = doesn't exist, continue with clone
  }

  // Create parent directory
  await mkdir(join(targetDir, ".."), { recursive: true });

  // Clone
  const cloneUrl = toCloneUrl(parsed);
  const args = ["clone"];
  if (config.shallow) args.push("--depth=1");
  args.push(cloneUrl, targetDir);

  const proc = Bun.spawn(["git", ...args], {
    stdout: "inherit",
    stderr: "inherit",
  });
  const exitCode = await proc.exited;
  if (exitCode !== 0) {
    throw new Error(`git clone failed with exit code ${exitCode}`);
  }

  // Update index
  const idx = await ProjectIndex.load(indexPath);
  idx.add(parsed.repo, {
    path: targetDir,
    url: cloneUrl,
    clonedAt: new Date().toISOString(),
  });
  await idx.save(indexPath);

  return targetDir;
}
