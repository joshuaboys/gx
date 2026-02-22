export interface FuzzyResult {
  name: string;
  score: number;
  path: string;
}

/**
 * Jaro similarity between two strings.
 * Returns a value between 0 (no similarity) and 1 (identical).
 */
function jaro(a: string, b: string): number {
  if (a === b) return 1.0;

  const aLen = a.length;
  const bLen = b.length;

  if (aLen === 0 || bLen === 0) return 0.0;

  // Maximum distance for matching characters
  const matchDist = Math.max(Math.floor(Math.max(aLen, bLen) / 2) - 1, 0);

  const aMatches = new Array(aLen).fill(false);
  const bMatches = new Array(bLen).fill(false);

  let matches = 0;
  let transpositions = 0;

  // Find matching characters
  for (let i = 0; i < aLen; i++) {
    const start = Math.max(0, i - matchDist);
    const end = Math.min(i + matchDist + 1, bLen);

    for (let j = start; j < end; j++) {
      if (bMatches[j] || a[i] !== b[j]) continue;
      aMatches[i] = true;
      bMatches[j] = true;
      matches++;
      break;
    }
  }

  if (matches === 0) return 0.0;

  // Count transpositions
  let k = 0;
  for (let i = 0; i < aLen; i++) {
    if (!aMatches[i]) continue;
    while (!bMatches[k]) k++;
    if (a[i] !== b[k]) transpositions++;
    k++;
  }

  return (
    (matches / aLen + matches / bLen + (matches - transpositions / 2) / matches) /
    3
  );
}

/**
 * Jaro-Winkler similarity between two strings.
 * Applies a prefix bonus to the Jaro score for strings that share
 * a common prefix (up to 4 characters).
 *
 * Comparison is case-insensitive.
 *
 * @returns A value between 0 (no similarity) and 1 (identical).
 */
export function jaroWinkler(a: string, b: string): number {
  // Case-insensitive comparison
  const al = a.toLowerCase();
  const bl = b.toLowerCase();

  const jaroScore = jaro(al, bl);

  // Common prefix length (up to 4 characters)
  let prefixLen = 0;
  const maxPrefix = Math.min(4, al.length, bl.length);
  for (let i = 0; i < maxPrefix; i++) {
    if (al[i] === bl[i]) {
      prefixLen++;
    } else {
      break;
    }
  }

  // Winkler scaling factor (standard value is 0.1)
  const p = 0.1;
  return jaroScore + prefixLen * p * (1 - jaroScore);
}

/**
 * Match a query against a list of project entries using Jaro-Winkler similarity.
 * Returns results above the threshold, sorted by score descending.
 *
 * @param query - The search string
 * @param entries - List of {name, path} to match against
 * @param threshold - Minimum similarity score (default: 0.7)
 * @returns Matching entries sorted by score descending
 */
export function fuzzyMatch(
  query: string,
  entries: Array<{ name: string; path: string }>,
  threshold = 0.7,
): FuzzyResult[] {
  if (!query || entries.length === 0) return [];

  return entries
    .map((entry) => ({
      name: entry.name,
      score: jaroWinkler(query, entry.name),
      path: entry.path,
    }))
    .filter((result) => result.score >= threshold)
    .sort((a, b) => b.score - a.score);
}
