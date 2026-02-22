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

export async function loadConfig(path?: string): Promise<Config> {
  const configPath = path ?? getConfigPath();
  try {
    const file = Bun.file(configPath);
    const raw = await file.json();
    return { ...DEFAULT_CONFIG, ...raw };
  } catch {
    return { ...DEFAULT_CONFIG };
  }
}

export async function saveConfig(
  path: string,
  config: Config
): Promise<void> {
  await mkdir(join(path, ".."), { recursive: true });
  await Bun.write(path, JSON.stringify(config, null, 2) + "\n");
}
