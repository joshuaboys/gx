import { ProjectIndex } from "../lib/index.ts";

export async function ls(indexPath: string): Promise<void> {
  const idx = await ProjectIndex.load(indexPath);
  const entries = idx.list();
  if (entries.length === 0) {
    console.error("No projects indexed. Clone a repo or run 'gx rebuild'.");
    return;
  }
  const maxName = Math.max(...entries.map((e) => e.name.length));
  for (const entry of entries) {
    console.log(`${entry.name.padEnd(maxName)}  ${entry.path}`);
  }
}
