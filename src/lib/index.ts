import { join } from "path";
import { mkdir, readdir } from "fs/promises";
import type { Index, IndexEntry } from "../types.ts";

export class ProjectIndex {
  private data: Index;

  private constructor(data: Index) {
    this.data = data;
  }

  static async load(path: string): Promise<ProjectIndex> {
    try {
      const file = Bun.file(path);
      const raw = await file.json();
      return new ProjectIndex(raw as Index);
    } catch {
      return new ProjectIndex({ projects: {} });
    }
  }

  add(name: string, entry: IndexEntry): void {
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
    await this.scanForRepos(projectDir);
  }

  private async scanForRepos(dir: string): Promise<void> {
    const entries = await readdir(dir, { withFileTypes: true });
    for (const entry of entries) {
      if (!entry.isDirectory()) continue;
      const fullPath = join(dir, entry.name);
      if (entry.name === ".git") {
        // Parent directory is the repo
        const name = dir.split("/").pop()!;
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
      await this.scanForRepos(join(dir, entry.name));
    }
  }

  async save(path: string): Promise<void> {
    await mkdir(join(path, ".."), { recursive: true });
    await Bun.write(path, JSON.stringify(this.data, null, 2) + "\n");
  }
}
