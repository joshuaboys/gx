import { ProjectIndex } from "../lib/index.ts";
import { effectiveProjectDir } from "../lib/config.ts";
import type { Config } from "../types.ts";

export async function rebuild(
  config: Config,
  indexPath: string,
): Promise<void> {
  const projectDir = effectiveProjectDir(config);
  const idx = await ProjectIndex.load(indexPath);
  await idx.scopedRebuild(projectDir);
  await idx.save(indexPath);
  const count = idx.list().length;
  console.error(`Indexed ${count} projects`);
}
