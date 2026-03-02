import { resolve as resolvePath, basename } from "path";
import { lstat } from "fs/promises";
import { ProjectIndex } from "../lib/index.ts";
import { expandTilde } from "../lib/config.ts";
import type { Config } from "../types.ts";

export async function indexRepos(
  paths: string[],
  config: Config,
  indexPath: string,
): Promise<void> {
  if (paths.length === 0) {
    await indexScan(config, indexPath);
    return;
  }

  await indexPaths(paths, indexPath);
}

async function indexPaths(paths: string[], indexPath: string): Promise<void> {
  const idx = await ProjectIndex.load(indexPath);

  for (const rawPath of paths) {
    const absPath = resolvePath(rawPath);

    // Verify .git exists
    try {
      await lstat(`${absPath}/.git`);
    } catch {
      throw new Error(`${absPath} is not a git repository`);
    }

    const name = basename(absPath);
    const url = await ProjectIndex.getRemoteUrl(absPath);
    const isNew = idx.merge(name, {
      path: absPath,
      url,
      clonedAt: new Date().toISOString(),
    });

    if (isNew) {
      console.error(`Indexed ${name} (${absPath})`);
    } else {
      console.error(`Already indexed: ${name} (${absPath})`);
    }
  }

  await idx.save(indexPath);
}

async function indexScan(config: Config, indexPath: string): Promise<void> {
  const projectDir = expandTilde(config.projectDir);
  const idx = await ProjectIndex.load(indexPath);
  const before = idx.list().length;
  await idx.additiveScan(projectDir);
  await idx.save(indexPath);
  const after = idx.list().length;
  const added = after - before;
  console.error(`Found ${added} new project${added !== 1 ? "s" : ""} (${after} total)`);
}
