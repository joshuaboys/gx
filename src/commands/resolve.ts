import { ProjectIndex } from "../lib/index.ts";
import { fuzzyMatch } from "../lib/fuzzy.ts";
import type { Config } from "../types.ts";

const AUTO_JUMP_THRESHOLD = 0.85;

export async function resolve(
  name: string,
  indexPath: string,
  config: Config,
  listAll = false,
): Promise<void> {
  const idx = await ProjectIndex.load(indexPath);

  if (listAll) {
    console.log(idx.names().join("\n"));
    return;
  }

  // Try exact match first
  const path = idx.resolve(name);
  if (path) {
    console.log(path);
    return;
  }

  // Fall back to fuzzy matching
  const entries = idx.list().map((e) => ({ name: e.name, path: e.path }));
  const matches = fuzzyMatch(name, entries, config.similarityThreshold);

  if (matches.length === 0) {
    console.error(`Project '${name}' not found`);
    process.exit(1);
  }

  if (matches.length === 1 && matches[0].score >= AUTO_JUMP_THRESHOLD) {
    // Single high-confidence match: auto-jump
    console.error(`Fuzzy match: '${name}' -> '${matches[0].name}' (${(matches[0].score * 100).toFixed(0)}%)`);
    console.log(matches[0].path);
    return;
  }

  // Multiple candidates or single low-confidence match: show list
  console.error(`No exact match for '${name}'. Did you mean:`);
  for (let i = 0; i < matches.length; i++) {
    console.error(`  ${i + 1}. ${matches[i].name} (${(matches[i].score * 100).toFixed(0)}%)`);
  }
  process.exit(1);
}
