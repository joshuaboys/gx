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
 * Detection order: package.json+bun.lockb > package.json > Cargo.toml > go.mod > pyproject.toml/requirements.txt > generic
 */
export async function detectProjectType(dir: string): Promise<ProjectType> {
  const has = async (name: string) =>
    Bun.file(`${dir}/${name}`).exists();

  if (await has("package.json")) {
    return (await has("bun.lockb")) ? "typescript-bun" : "typescript-node";
  }
  if (await has("Cargo.toml")) return "rust";
  if (await has("go.mod")) return "go";
  if ((await has("pyproject.toml")) || (await has("requirements.txt"))) {
    return "python";
  }

  return "generic";
}
