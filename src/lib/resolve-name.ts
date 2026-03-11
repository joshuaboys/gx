import type { ProjectIndex } from "./index.ts";
import { fuzzyMatch, AUTO_JUMP_THRESHOLD, type FuzzyResult } from "./fuzzy.ts";
import type { Config } from "../types.ts";

export type ResolveResult =
  | { kind: "exact"; name: string; path: string }
  | { kind: "auto"; name: string; path: string; query: string; score: number }
  | { kind: "ambiguous"; query: string; matches: FuzzyResult[] }
  | { kind: "missing"; query: string };

export function resolveProjectName(
  query: string,
  idx: ProjectIndex,
  config: Config,
): ResolveResult {
  const path = idx.resolve(query);
  if (path) return { kind: "exact", name: query, path };

  const entries = idx.list().map((e) => ({ name: e.name, path: e.path }));
  const matches = fuzzyMatch(query, entries, config.similarityThreshold);

  if (matches.length === 0) return { kind: "missing", query };

  const first = matches[0]!;
  if (matches.length === 1 && first.score >= AUTO_JUMP_THRESHOLD) {
    return {
      kind: "auto",
      name: first.name,
      path: first.path,
      query,
      score: first.score,
    };
  }

  return { kind: "ambiguous", query, matches };
}

export function formatAmbiguous(query: string, matches: FuzzyResult[]): string {
  const lines = [`No exact match for '${query}'. Did you mean:`];
  for (const [i, m] of matches.entries()) {
    lines.push(`  ${i + 1}. ${m.name} (${(m.score * 100).toFixed(0)}%)`);
  }
  return lines.join("\n");
}

export function formatAutoMatch(
  query: string,
  name: string,
  score: number,
): string {
  return `Fuzzy match: '${query}' -> '${name}' (${(score * 100).toFixed(0)}%)`;
}
