export interface Config {
  projectDir: string;
  defaultHost: string;
  defaultOwner: string;
  structure: "flat" | "owner" | "host";
  shallow: boolean;
  similarityThreshold: number;
  editor: string;
}

export const DEFAULT_CONFIG: Config = {
  projectDir: "~/Projects/src",
  defaultHost: "github.com",
  defaultOwner: "",
  structure: "owner",
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
  lastVisited?: string;
}

export interface Index {
  projects: Record<string, IndexEntry>;
}
