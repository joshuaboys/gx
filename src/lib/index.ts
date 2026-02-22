import { join, basename } from "path";
import { mkdir, readdir, realpath } from "fs/promises";
import type { Index, IndexEntry } from "../types.ts";

const MAX_SCAN_DEPTH = 10;
const SKIP_DIRS = new Set(["node_modules", "vendor", "target", ".build", "dist", "build"]);

export class ProjectIndex {
  private data: Index;

  private constructor(data: Index) {
    this.data = data;
  }

  static async load(path: string): Promise<ProjectIndex> {
    try {
      const file = Bun.file(path);
      const raw = await file.json();
      if (
        typeof raw === "object" &&
        raw !== null &&
        "projects" in raw &&
        typeof (raw as Record<string, unknown>).projects === "object"
      ) {
        return new ProjectIndex(raw as Index);
      }
      return new ProjectIndex({ projects: {} });
    } catch {
      return new ProjectIndex({ projects: {} });
    }
  }

  add(name: string, entry: IndexEntry): void {
    if (this.data.projects[name] && this.data.projects[name].path !== entry.path) {
      console.error(
        `Warning: project name '${name}' collision — overwriting ${this.data.projects[name].path} with ${entry.path}`,
      );
    }
    this.data.projects[name] = entry;
  }

  resolve(name: string): string | null {
    return this.data.projects[name]?.path ?? null;
  }

  list(): Array<{ name: string } & IndexEntry> {
    return Object.entries(this.data.projects)
      .map(([name, entry]) => ({ name, ...entry }))
      .sort((a, b) => a.name.localeCompare(b.name));
  }

  names(): string[] {
    return Object.keys(this.data.projects).sort();
  }

  async rebuild(projectDir: string): Promise<void> {
    this.data.projects = {};
    const visited = new Set<string>();
    await this.scanForRepos(projectDir, 0, visited);
  }

  private async scanForRepos(
    dir: string,
    depth: number,
    visited: Set<string>,
  ): Promise<void> {
    if (depth > MAX_SCAN_DEPTH) return;

    // Resolve symlinks to detect cycles
    let realDir: string;
    try {
      realDir = await realpath(dir);
    } catch {
      return; // Broken symlink or permission denied
    }
    if (visited.has(realDir)) return;
    visited.add(realDir);

    let entries;
    try {
      entries = await readdir(dir, { withFileTypes: true });
    } catch {
      return; // Permission denied or missing directory
    }

    for (const entry of entries) {
      if (!entry.isDirectory()) continue;
      if (entry.name === ".git") {
        const name = basename(dir);
        this.data.projects[name] = {
          path: dir,
          url: "",
          clonedAt: "",
        };
        return; // Don't descend into .git or sibling dirs of a repo
      }
    }

    // No .git found at this level, recurse into subdirectories
    for (const entry of entries) {
      if (!entry.isDirectory() || entry.name.startsWith(".")) continue;
      if (SKIP_DIRS.has(entry.name)) continue;
      await this.scanForRepos(join(dir, entry.name), depth + 1, visited);
    }
  }

  async save(path: string): Promise<void> {
    await mkdir(join(path, ".."), { recursive: true });
    await Bun.write(path, JSON.stringify(this.data, null, 2) + "\n");
  }
}
