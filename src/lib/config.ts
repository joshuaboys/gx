import { join } from "path";
import { homedir } from "os";
import { mkdir } from "fs/promises";
import type { Config } from "../types.ts";
import { DEFAULT_CONFIG } from "../types.ts";

export function getConfigPath(): string {
  return join(homedir(), ".config", "gx", "config.json");
}

export function expandTilde(p: string): string {
  if (p.startsWith("~/") || p === "~") {
    return join(homedir(), p.slice(1));
  }
  return p;
}

export function getAgent(): string | null {
  const agent = process.env.GX_AGENT?.trim().toLowerCase() ?? null;
  if (agent && !/^[a-z0-9]([a-z0-9-]*[a-z0-9])?$/.test(agent)) {
    throw new Error(
      `Invalid GX_AGENT: "${agent}" — use lowercase alphanumeric and hyphens`,
    );
  }
  return agent || null;
}

export function effectiveProjectDir(config: Config): string {
  const base = expandTilde(config.projectDir);
  const agent = getAgent();
  return agent ? join(base, `.${agent}`) : base;
}

function validateConfig(raw: unknown): Config {
  if (typeof raw !== "object" || raw === null) {
    return { ...DEFAULT_CONFIG };
  }
  const obj = raw as Record<string, unknown>;
  return {
    projectDir:
      typeof obj.projectDir === "string" && obj.projectDir
        ? obj.projectDir
        : DEFAULT_CONFIG.projectDir,
    defaultHost:
      typeof obj.defaultHost === "string" && obj.defaultHost
        ? obj.defaultHost
        : DEFAULT_CONFIG.defaultHost,
    structure:
      obj.structure === "flat" ||
      obj.structure === "owner" ||
      obj.structure === "host"
        ? obj.structure
        : DEFAULT_CONFIG.structure,
    shallow:
      typeof obj.shallow === "boolean" ? obj.shallow : DEFAULT_CONFIG.shallow,
    similarityThreshold:
      typeof obj.similarityThreshold === "number" &&
      obj.similarityThreshold >= 0 &&
      obj.similarityThreshold <= 1
        ? obj.similarityThreshold
        : DEFAULT_CONFIG.similarityThreshold,
    editor: typeof obj.editor === "string" ? obj.editor : DEFAULT_CONFIG.editor,
  };
}

export async function loadConfig(path?: string): Promise<Config> {
  const configPath = path ?? getConfigPath();
  try {
    const file = Bun.file(configPath);
    const raw = await file.json();
    return validateConfig(raw);
  } catch {
    return { ...DEFAULT_CONFIG };
  }
}

export async function saveConfig(path: string, config: Config): Promise<void> {
  await mkdir(join(path, ".."), { recursive: true });
  await Bun.write(path, JSON.stringify(config, null, 2) + "\n");
}
