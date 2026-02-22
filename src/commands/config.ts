import { loadConfig, saveConfig } from "../lib/config.ts";
import type { Config } from "../types.ts";

export async function showConfig(configPath: string): Promise<void> {
  const config = await loadConfig(configPath);
  console.log(JSON.stringify(config, null, 2));
}

export async function setConfig(
  configPath: string,
  key: string,
  value: string
): Promise<void> {
  const config = await loadConfig(configPath);
  if (!(key in config)) {
    console.error(`Unknown config key: ${key}`);
    console.error(`Valid keys: ${Object.keys(config).join(", ")}`);
    process.exit(1);
  }
  const k = key as keyof Config;
  if (typeof config[k] === "boolean") {
    (config as Record<string, unknown>)[key] = value === "true";
  } else {
    (config as Record<string, unknown>)[key] = value;
  }
  await saveConfig(configPath, config);
  console.error(`Set ${key} = ${value}`);
}
