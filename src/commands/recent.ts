import { ProjectIndex } from "../lib/index.ts";
import { relativeTime } from "../lib/time.ts";

export async function recent(indexPath: string, limit?: number): Promise<void> {
  const idx = await ProjectIndex.load(indexPath);
  const entries = idx.recent(limit);

  if (entries.length === 0) {
    console.error("No projects indexed. Clone a repo or run 'gx rebuild'.");
    return;
  }

  const maxName = Math.max(...entries.map(([name]) => name.length));
  for (const [name, entry] of entries) {
    const time = relativeTime(entry.lastVisited || entry.clonedAt);
    const timeStr = time ? `  ${time}` : "";
    console.log(`${name.padEnd(maxName)}  ${entry.path}${timeStr}`);
  }
}
