export interface Config {
  projectDir: string;
  defaultHost: string;
  structure: "flat" | "host";
  shallow: boolean;
  similarityThreshold: number;
  editor: string;
}

export const DEFAULT_CONFIG: Config = {
  projectDir: "~/Projects/src",
  defaultHost: "github.com",
  structure: "flat",
  shallow: false,
  similarityThreshold: 0.7,
  editor: "",
};

export interface ParsedRepo {
  host: string;
  owner: string;
  repo: string;
  originalUrl: string;
}

export interface IndexEntry {
  path: string;
  url: string;
  clonedAt: string;
}

export interface Index {
  projects: Record<string, IndexEntry>;
}
