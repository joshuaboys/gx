import { ProjectIndex } from "../lib/index.ts";
import { expandTilde } from "../lib/config.ts";
import type { Config } from "../types.ts";

export async function rebuild(
  config: Config,
  indexPath: string
): Promise<void> {
  const projectDir = expandTilde(config.projectDir);
  const idx = await ProjectIndex.load(indexPath);
  await idx.rebuild(projectDir);
  await idx.save(indexPath);
  const count = idx.list().length;
  console.error(`Indexed ${count} projects`);
}
