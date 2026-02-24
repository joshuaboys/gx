export type ProjectType =
  | "typescript-bun"
  | "typescript-node"
  | "rust"
  | "go"
  | "python"
  | "generic";

export const VALID_TYPES: ReadonlySet<string> = new Set<ProjectType>([
  "typescript-bun",
  "typescript-node",
  "rust",
  "go",
  "python",
  "generic",
]);

/**
 * Detect project type by checking for manifest files in the given directory.
 * Detection order: package.json+bun.lock(b) > package.json > Cargo.toml > go.mod > pyproject.toml/requirements.txt > generic
 *
 * Bun v1.2+ uses a text-format lockfile named `bun.lock`; older versions used
 * the binary `bun.lockb`. Both are treated as indicators of a Bun project.
 */
export async function detectProjectType(dir: string): Promise<ProjectType> {
  const has = async (name: string) =>
    Bun.file(`${dir}/${name}`).exists();

  if (await has("package.json")) {
    const isBun = (await has("bun.lock")) || (await has("bun.lockb"));
    return isBun ? "typescript-bun" : "typescript-node";
  }
  if (await has("Cargo.toml")) return "rust";
  if (await has("go.mod")) return "go";
  if ((await has("pyproject.toml")) || (await has("requirements.txt"))) {
    return "python";
  }

  return "generic";
}
