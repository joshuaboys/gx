import {
  loadConfig,
  saveConfig,
  getAgent,
  effectiveProjectDir,
} from "../lib/config.ts";
import type { Config } from "../types.ts";

const VALID_STRUCTURES = new Set(["flat", "owner", "host"]);

export async function showConfig(configPath: string): Promise<void> {
  const config = await loadConfig(configPath);
  const agent = getAgent();
  const output: Record<string, unknown> = { ...config };
  if (agent) {
    output.agent = agent;
    output.effectiveProjectDir = effectiveProjectDir(config);
  }
  console.log(JSON.stringify(output, null, 2));
}

export async function setConfig(
  configPath: string,
  key: string,
  value: string,
): Promise<void> {
  const config = await loadConfig(configPath);
  if (!(key in config)) {
    console.error(`Unknown config key: ${key}`);
    console.error(`Valid keys: ${Object.keys(config).join(", ")}`);
    process.exit(1);
  }

  const k = key as keyof Config;
  const record = config as unknown as Record<string, unknown>;
  if (key === "structure") {
    if (!VALID_STRUCTURES.has(value)) {
      console.error(`Invalid structure value: ${value}`);
      console.error(`Valid values: flat, owner, host`);
      process.exit(1);
    }
    record[key] = value;
  } else if (typeof config[k] === "boolean") {
    record[key] = value === "true";
  } else if (typeof config[k] === "number") {
    const num = Number(value);
    if (Number.isNaN(num)) {
      console.error(`Value for '${key}' must be a number`);
      process.exit(1);
    }
    record[key] = num;
  } else {
    record[key] = value;
  }

  await saveConfig(configPath, config);
  console.error(`Set ${key} = ${value}`);
}
